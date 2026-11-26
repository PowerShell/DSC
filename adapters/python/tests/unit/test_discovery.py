# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Unit tests for pyadapter.discovery — list entry generation and cache integration.
"""
from __future__ import annotations

import importlib.metadata
import importlib.resources
import json
from collections.abc import Iterator
from dataclasses import dataclass
from unittest.mock import MagicMock, patch

import pytest

from ms_dsc import DscResource, dsc_resource
from ms_dsc.metadata import SetReturn, TestReturn
from ms_dsc.schema import DataclassSchemaProvider
from ms_dsc.results import SetResult, TestResult


# ---------------------------------------------------------------------------
# Test resource classes
# ---------------------------------------------------------------------------

@dataclass
class Schema:
    name: str
    _exist: bool = True


@dsc_resource(type="Disc/Full", version="1.2.3", description="All caps",
              tags=["a", "b"], set_return=SetReturn.STATE_AND_DIFF)
class FullResource(DscResource[Schema]):
    schema_provider = DataclassSchemaProvider(Schema)
    def get(self, i): return i
    def set(self, i): return SetResult(actual_state=i, changed_properties=[])
    def test(self, i): return TestResult(actual_state=i, differing_properties=[])
    def delete(self, i): pass
    def export(self, i): return iter([])


@dsc_resource(type="Disc/GetOnly", version="0.1.0")
class GetOnlyResource(DscResource[Schema]):
    schema_provider = DataclassSchemaProvider(Schema)
    def get(self, i): return i


# ---------------------------------------------------------------------------
# Tests for _build_list_entry
# ---------------------------------------------------------------------------

class TestBuildListEntry:
    def test_type_name(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert entry["type"] == "Disc/Full"

    def test_version_from_metadata(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert entry["version"] == "1.2.3"

    def test_description_from_metadata(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert entry["description"] == "All caps"

    def test_no_tags_in_entry(self):
        """Tags are excluded from list entries (not supported in DSC adapted resource schema)."""
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert "tags" not in entry

    def test_require_adapter_field(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert entry["requireAdapter"] == "Microsoft.Adapter/Python"

    def test_capabilities_all_five(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        for cap in ("get", "set", "test", "delete", "export"):
            assert cap in entry["capabilities"]

    def test_capabilities_get_only(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/GetOnly", GetOnlyResource)
        assert entry["capabilities"] == ["get"]
        for cap in ("set", "test", "delete", "export"):
            assert cap not in entry["capabilities"]

    def test_schema_embedded(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert "schema" in entry
        assert "embedded" in entry["schema"]
        schema = entry["schema"]["embedded"]
        assert schema["type"] == "object"

    def test_kind_is_resource(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert entry["kind"] == "resource"

    def test_no_description_key_when_empty(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/GetOnly", GetOnlyResource)
        assert "description" not in entry or entry.get("description") == ""

    def test_no_tags_key_when_empty(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/GetOnly", GetOnlyResource)
        assert "tags" not in entry or entry.get("tags") == []

    def test_no_content_field_in_list_entry(self):
        """List entries must NOT include 'content' — it is not a valid DscResource field."""
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert "content" not in entry

    def test_no_path_field_in_entry(self):
        """List entries should not include a 'path' field."""
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert "path" not in entry


# ---------------------------------------------------------------------------
# Tests for _manifests_for_dist (importlib.resources-based discovery)
# ---------------------------------------------------------------------------

class TestManifestsForDist:
    def _make_dist(self, name: str) -> MagicMock:
        d = MagicMock()
        d.name = name
        return d

    def test_returns_empty_for_non_dsc_package(self):
        """Distributions without a 'dsc' subpackage return no manifests."""
        from pyadapter.discovery import _manifests_for_dist
        dist = self._make_dist("some-regular-package")
        with patch("importlib.resources.files", side_effect=ModuleNotFoundError):
            result = _manifests_for_dist(dist)
        assert result == []

    def test_normalises_hyphenated_name(self, tmp_path):
        """Distribution names with hyphens are normalised to underscores."""
        from pyadapter.discovery import _manifests_for_dist
        dist = self._make_dist("my-dsc-resource")
        # Should try 'my_dsc_resource.dsc', not 'my-dsc-resource.dsc'.
        with patch("importlib.resources.files", side_effect=ModuleNotFoundError) as mock_files:
            _manifests_for_dist(dist)
        mock_files.assert_called_once_with("my_dsc_resource.dsc")

    def test_finds_manifest_via_resources(self, tmp_path):
        """Returns manifest path entries when importlib.resources finds manifests."""
        from pyadapter.discovery import _manifests_for_dist

        # Create a real manifest file on disk.
        manifest = tmp_path / "Test.Resource.dsc.adaptedResource.json"
        manifest.write_text('{"type": "Test/Resource"}', encoding="utf-8")

        dist = self._make_dist("test-resource")

        mock_resource = MagicMock()
        mock_resource.name = "Test.Resource.dsc.adaptedResource.json"
        mock_resource.__str__ = lambda self: str(manifest)

        mock_ref = MagicMock()
        mock_ref.iterdir.return_value = [mock_resource]

        with patch("importlib.resources.files", return_value=mock_ref):
            result = _manifests_for_dist(dist)

        assert len(result) == 1
        assert "manifestPath" in result[0]

    def test_skips_non_manifest_files(self, tmp_path):
        """Non-manifest files in the dsc package are ignored."""
        from pyadapter.discovery import _manifests_for_dist

        dist = self._make_dist("test-resource")

        mock_resource = MagicMock()
        mock_resource.name = "README.md"
        mock_ref = MagicMock()
        mock_ref.iterdir.return_value = [mock_resource]

        with patch("importlib.resources.files", return_value=mock_ref):
            result = _manifests_for_dist(dist)

        assert result == []


# ---------------------------------------------------------------------------
# Tests for cmd_list
# ---------------------------------------------------------------------------

class TestCmdList:
    def _make_ep(self, type_name, cls):
        return type("EP", (), {"name": type_name, "value": "x:Y", "load": lambda self, c=cls: c})()

    def test_returns_zero(self, tmp_path):
        """cmd_list exits with 0 when at least one entry point exists."""
        import io
        from pyadapter.discovery import cmd_list, LIST_CACHE

        LIST_CACHE._path = tmp_path / "list.json"

        eps = [self._make_ep("Disc/Full", FullResource)]
        with patch.object(importlib.metadata, "entry_points", return_value=eps):
            with patch("pyadapter.discovery._covered_types", return_value=set()):
                buf = io.StringIO()
                with patch("sys.stdout", buf):
                    rc = cmd_list()
        assert rc == 0

    def test_emits_ndjson(self, tmp_path):
        """Each line of list output is a valid JSON object."""
        import io
        from pyadapter.discovery import cmd_list, LIST_CACHE

        LIST_CACHE._path = tmp_path / "list.json"

        eps = [self._make_ep("Disc/Full", FullResource), self._make_ep("Disc/GetOnly", GetOnlyResource)]
        with patch.object(importlib.metadata, "entry_points", return_value=eps):
            with patch("pyadapter.discovery._covered_types", return_value=set()):
                buf = io.StringIO()
                with patch("sys.stdout", buf):
                    cmd_list()

        lines = [l for l in buf.getvalue().splitlines() if l.strip()]
        assert len(lines) == 2
        for line in lines:
            obj = json.loads(line)
            assert "type" in obj
            assert "capabilities" in obj

    def test_list_entries_have_no_content_field(self, tmp_path):
        """List entries must not contain 'content' (not a valid DscResource field)."""
        import io
        from pyadapter.discovery import cmd_list, LIST_CACHE

        LIST_CACHE._path = tmp_path / "list.json"

        eps = [self._make_ep("Disc/Full", FullResource)]
        with patch.object(importlib.metadata, "entry_points", return_value=eps):
            with patch("pyadapter.discovery._covered_types", return_value=set()):
                buf = io.StringIO()
                with patch("sys.stdout", buf):
                    cmd_list()

        for line in buf.getvalue().splitlines():
            if line.strip():
                assert "content" not in json.loads(line)

    def test_skips_covered_types(self, tmp_path):
        """Resources already covered by pre-built manifests are excluded from list output."""
        import io
        from pyadapter.discovery import cmd_list, LIST_CACHE

        LIST_CACHE._path = tmp_path / "list.json"

        eps = [self._make_ep("Disc/Full", FullResource), self._make_ep("Disc/GetOnly", GetOnlyResource)]
        covered = {"disc/full"}
        with patch.object(importlib.metadata, "entry_points", return_value=eps):
            with patch("pyadapter.discovery._covered_types", return_value=covered):
                buf = io.StringIO()
                with patch("sys.stdout", buf):
                    cmd_list()

        lines = [l for l in buf.getvalue().splitlines() if l.strip()]
        types = [json.loads(l)["type"] for l in lines]
        assert "Disc/Full" not in types
        assert "Disc/GetOnly" in types


# ---------------------------------------------------------------------------
# Tests for cmd_clear_cache
# ---------------------------------------------------------------------------

class TestCmdClearCache:
    def test_clear_removes_list_cache(self, tmp_path):
        from pyadapter.discovery import cmd_clear_cache, LIST_CACHE

        LIST_CACHE._path = tmp_path / "list.json"
        LIST_CACHE.save("fp", [])
        assert LIST_CACHE._path.exists()

        rc = cmd_clear_cache()
        assert rc == 0
        assert not LIST_CACHE._path.exists()

    def test_clear_is_idempotent(self, tmp_path):
        from pyadapter.discovery import cmd_clear_cache, LIST_CACHE

        LIST_CACHE._path = tmp_path / "list.json"
        rc = cmd_clear_cache()
        assert rc == 0


# ---------------------------------------------------------------------------
# Additional tests for edge cases and error handling
# ---------------------------------------------------------------------------

class TestCapabilitiesFor:
    """Test _capabilities_for protocol detection."""

    def test_gettable_only(self):
        """Detect Gettable capability."""
        from ms_dsc.protocols import Gettable

        class GetableResource(Gettable):
            def get(self, instance):
                return instance

        from pyadapter.discovery import _capabilities_for
        caps = _capabilities_for(GetableResource)
        assert "get" in caps

    def test_settable_only(self):
        """Detect Settable capability."""
        from ms_dsc.protocols import Settable

        class SettableResource(Settable):
            def set(self, instance):
                pass

        from pyadapter.discovery import _capabilities_for
        caps = _capabilities_for(SettableResource)
        assert "set" in caps

    def test_testable_only(self):
        """Detect Testable capability."""
        from ms_dsc.protocols import Testable

        class TestableResource(Testable):
            def test(self, instance):
                return True

        from pyadapter.discovery import _capabilities_for
        caps = _capabilities_for(TestableResource)
        assert "test" in caps

    def test_deletable_only(self):
        """Detect Deletable capability."""
        from ms_dsc.protocols import Deletable

        class DeletableResource(Deletable):
            def delete(self, instance):
                pass

        from pyadapter.discovery import _capabilities_for
        caps = _capabilities_for(DeletableResource)
        assert "delete" in caps

    def test_exportable_only(self):
        """Detect Exportable capability."""
        from ms_dsc.protocols import Exportable

        class ExportableResource(Exportable):
            def export(self):
                return []

        from pyadapter.discovery import _capabilities_for
        caps = _capabilities_for(ExportableResource)
        assert "export" in caps

    def test_all_capabilities(self):
        """Detect all capabilities when class implements all protocols."""
        from ms_dsc.protocols import Gettable, Settable, Testable, Deletable, Exportable

        class FullResource(Gettable, Settable, Testable, Deletable, Exportable):
            def get(self, instance):
                return instance

            def set(self, instance):
                pass

            def test(self, instance):
                return True

            def delete(self, instance):
                pass

            def export(self):
                return []

        from pyadapter.discovery import _capabilities_for
        caps = _capabilities_for(FullResource)
        assert set(caps) == {"get", "set", "test", "delete", "export"}

    def test_no_capabilities(self):
        """Class with no protocol implementations returns empty list."""
        from pyadapter.discovery import _capabilities_for

        class NoCapabilities:
            pass

        caps = _capabilities_for(NoCapabilities)
        assert caps == []


class TestSourcePathFor:
    """Test _source_path_for path resolution."""

    def test_valid_module_resolution(self):
        """Resolve source path for a valid module."""
        from ms_dsc.protocols import Gettable
        from pyadapter.discovery import _source_path_for

        source = _source_path_for(Gettable)
        assert "ms_dsc" in source or "protocols" in source

    def test_fallback_to_module_qualname(self):
        """Fall back to module:qualname format on error."""
        from pyadapter.discovery import _source_path_for

        class FakeResource:
            __module__ = "fake.module"
            __qualname__ = "FakeResource"

        with patch("importlib.util.find_spec") as mock_find:
            mock_find.return_value = None
            source = _source_path_for(FakeResource)
            assert source == "fake.module:FakeResource"

    def test_spec_with_no_origin(self):
        """When spec exists but has no origin, use fallback."""
        from pyadapter.discovery import _source_path_for

        class FakeResource:
            __module__ = "test.module"
            __qualname__ = "TestResource"

        mock_spec = MagicMock()
        mock_spec.origin = None

        with patch("importlib.util.find_spec") as mock_find:
            mock_find.return_value = mock_spec
            source = _source_path_for(FakeResource)
            assert source == "test.module:TestResource"

    def test_exception_in_find_spec(self):
        """Exception in find_spec falls back gracefully."""
        from pyadapter.discovery import _source_path_for

        class FakeResource:
            __module__ = "error.module"
            __qualname__ = "ErrorResource"

        with patch("importlib.util.find_spec") as mock_find:
            mock_find.side_effect = RuntimeError("Import error")
            source = _source_path_for(FakeResource)
            assert source == "error.module:ErrorResource"


class TestManifestsParsing:
    """Test manifest path discovery and parsing."""

    def test_iter_manifest_paths_returns_list(self):
        """_iter_manifest_paths returns list even with no distributions."""
        from pyadapter.discovery import _iter_manifest_paths
        results = _iter_manifest_paths()
        assert isinstance(results, list)

    def test_manifests_for_dist_handles_module_not_found(self):
        """_manifests_for_dist handles ModuleNotFoundError gracefully."""
        from pyadapter.discovery import _manifests_for_dist

        mock_dist = MagicMock()
        mock_dist.name = "some-package"

        with patch("pyadapter.discovery.importlib.resources.files") as mock_files:
            mock_files.side_effect = ModuleNotFoundError("No dsc subpackage")
            results = _manifests_for_dist(mock_dist)
            assert results == []

    def test_manifests_for_dist_handles_type_error(self):
        """_manifests_for_dist handles TypeError from files()."""
        from pyadapter.discovery import _manifests_for_dist

        mock_dist = MagicMock()
        mock_dist.name = "invalid-package"

        with patch("pyadapter.discovery.importlib.resources.files") as mock_files:
            mock_files.side_effect = TypeError("Not a package")
            results = _manifests_for_dist(mock_dist)
            assert results == []

    def test_manifests_for_dist_normalizes_package_name(self):
        """Package name normalization: hyphens and dots become underscores."""
        from pyadapter.discovery import _manifests_for_dist

        mock_dist = MagicMock()
        mock_dist.name = "some-package.with-dots"

        with patch("pyadapter.discovery.importlib.resources.files") as mock_files:
            mock_files.side_effect = ModuleNotFoundError()
            _manifests_for_dist(mock_dist)
            # Should have called with normalized name
            mock_files.assert_called_once()
            call_arg = mock_files.call_args[0][0]
            assert "some_package_with_dots" in call_arg

    def test_covered_types_handles_invalid_json(self):
        """_covered_types handles invalid JSON in manifest gracefully."""
        from pyadapter.discovery import _covered_types, _iter_manifest_paths

        mock_entry = {"manifestPath": "/fake/path.json"}

        with patch("pyadapter.discovery._iter_manifest_paths") as mock_iter:
            mock_iter.return_value = [mock_entry]

            with patch("pathlib.Path.read_text") as mock_read:
                mock_read.return_value = "invalid json {"
                covered = _covered_types()
                assert isinstance(covered, set)

    def test_covered_types_extracts_type_names(self):
        """_covered_types correctly extracts type names from manifests."""
        from pyadapter.discovery import _covered_types

        valid_manifest = {"type": "Microsoft.Example/Resource"}
        mock_entry = {"manifestPath": "/fake/manifest.json"}

        with patch("pyadapter.discovery._iter_manifest_paths") as mock_iter:
            mock_iter.return_value = [mock_entry]

            with patch("pathlib.Path.read_text") as mock_read:
                mock_read.return_value = json.dumps(valid_manifest)
                covered = _covered_types()
                assert "microsoft.example/resource" in covered

    def test_covered_types_case_folding(self):
        """Type names are case-folded for comparison."""
        from pyadapter.discovery import _covered_types

        manifest = {"type": "MyType/Resource"}
        mock_entry = {"manifestPath": "/fake/manifest.json"}

        with patch("pyadapter.discovery._iter_manifest_paths") as mock_iter:
            mock_iter.return_value = [mock_entry]

            with patch("pathlib.Path.read_text") as mock_read:
                mock_read.return_value = json.dumps(manifest)
                covered = _covered_types()
                assert "mytype/resource" in covered


class TestListFromEntryPoints:
    """Test entry point enumeration."""

    def test_empty_entry_points_returns_empty_list(self):
        """No entry points returns empty list."""
        from pyadapter.discovery import _list_from_entry_points

        with patch("pyadapter.discovery.importlib.metadata.entry_points") as mock_ep:
            mock_ep.return_value = []
            results = _list_from_entry_points(set())
            assert results == []

    def test_entry_point_enumeration_failure(self):
        """Exception during entry point enumeration returns empty list."""
        from pyadapter.discovery import _list_from_entry_points

        with patch("pyadapter.discovery.importlib.metadata.entry_points") as mock_ep:
            mock_ep.side_effect = RuntimeError("entry_points failed")
            results = _list_from_entry_points(set())
            assert results == []

    def test_covered_resources_are_skipped(self):
        """Resources already covered by manifests are skipped."""
        from pyadapter.discovery import _list_from_entry_points

        mock_ep = MagicMock()
        mock_ep.name = "CoveredResource"
        mock_ep.load.return_value = MagicMock()

        with patch("pyadapter.discovery.importlib.metadata.entry_points") as mock_eps:
            mock_eps.return_value = [mock_ep]
            covered = {"coveredresource"}  # lowercase
            results = _list_from_entry_points(covered)
            assert results == []
            mock_ep.load.assert_not_called()

    def test_entry_point_load_failure_is_handled(self):
        """Failed entry point loads are handled gracefully."""
        from pyadapter.discovery import _list_from_entry_points

        mock_ep = MagicMock()
        mock_ep.name = "FailedResource"
        mock_ep.load.side_effect = ImportError("Module not found")

        with patch("pyadapter.discovery.importlib.metadata.entry_points") as mock_eps:
            mock_eps.return_value = [mock_ep]
            results = _list_from_entry_points(set())
            assert results == []  # Failed loads produce no entry

    def test_successful_entry_point_included(self):
        """Successfully loaded entry points are included."""
        from ms_dsc.protocols import Gettable
        from pyadapter.discovery import _list_from_entry_points

        class ValidResource(Gettable):
            def get(self, instance):
                return instance

        mock_ep = MagicMock()
        mock_ep.name = "ValidResource"
        mock_ep.load.return_value = ValidResource

        with patch("pyadapter.discovery.importlib.metadata.entry_points") as mock_eps:
            mock_eps.return_value = [mock_ep]
            results = _list_from_entry_points(set())
            assert len(results) == 1
            assert results[0]["type"] == "ValidResource"



# ---------------------------------------------------------------------------
# Test resource classes
# ---------------------------------------------------------------------------

@dataclass
class Schema:
    name: str
    _exist: bool = True


@dsc_resource(type="Disc/Full", version="1.2.3", description="All caps",
              tags=["a", "b"], set_return=SetReturn.STATE_AND_DIFF)
class FullResource(DscResource[Schema]):
    schema_provider = DataclassSchemaProvider(Schema)
    def get(self, i): return i
    def set(self, i): return SetResult(actual_state=i, changed_properties=[])
    def test(self, i): return TestResult(actual_state=i, differing_properties=[])
    def delete(self, i): pass
    def export(self, i): return iter([])


@dsc_resource(type="Disc/GetOnly", version="0.1.0")
class GetOnlyResource(DscResource[Schema]):
    schema_provider = DataclassSchemaProvider(Schema)
    def get(self, i): return i


# ---------------------------------------------------------------------------
# Tests for _build_list_entry
# ---------------------------------------------------------------------------

class TestBuildListEntry:
    def test_type_name(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert entry["type"] == "Disc/Full"

    def test_version_from_metadata(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert entry["version"] == "1.2.3"

    def test_description_from_metadata(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert entry["description"] == "All caps"

    def test_no_tags_in_entry(self):
        """Tags are excluded from list entries (not supported in DSC adapted resource schema)."""
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert "tags" not in entry

    def test_require_adapter_field(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert entry["requireAdapter"] == "Microsoft.Adapter/Python"

    def test_capabilities_all_five(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        for cap in ("get", "set", "test", "delete", "export"):
            assert cap in entry["capabilities"]

    def test_capabilities_get_only(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/GetOnly", GetOnlyResource)
        assert entry["capabilities"] == ["get"]
        for cap in ("set", "test", "delete", "export"):
            assert cap not in entry["capabilities"]

    def test_schema_embedded(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert "schema" in entry
        assert "embedded" in entry["schema"]
        schema = entry["schema"]["embedded"]
        assert schema["type"] == "object"

    def test_kind_is_resource(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/Full", FullResource)
        assert entry["kind"] == "resource"

    def test_no_description_key_when_empty(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/GetOnly", GetOnlyResource)
        # description is empty string → should not appear
        assert "description" not in entry or entry.get("description") == ""

    def test_no_tags_key_when_empty(self):
        from pyadapter.discovery import _build_list_entry
        entry = _build_list_entry("Disc/GetOnly", GetOnlyResource)
        assert "tags" not in entry or entry.get("tags") == []

