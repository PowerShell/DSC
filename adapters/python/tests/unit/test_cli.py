# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Unit tests for ms_dsc.cli — dsc-gen manifest command.
"""
from __future__ import annotations

import json
from collections.abc import Iterator
from dataclasses import dataclass, field
from pathlib import Path

import pytest

from ms_dsc import DscResource, dsc_resource
from ms_dsc.metadata import SetReturn, TestReturn
from ms_dsc.results import SetResult, TestResult
from ms_dsc.schema import DataclassSchemaProvider


# ---------------------------------------------------------------------------
# Module-level resource classes (must be importable for dsc-gen)
# ---------------------------------------------------------------------------

@dataclass
class _CliTestSchema:
    name: str = field(metadata={"description": "Name field."})
    _exist: bool = field(default=True, metadata={"description": "Existence flag."})


@dsc_resource(type="Cli/GetOnly", version="1.2.3", description="CLI test get-only resource")
class _CliGetOnlyResource(DscResource[_CliTestSchema]):
    schema_provider = DataclassSchemaProvider(_CliTestSchema)
    def get(self, instance): return instance


@dsc_resource(type="Cli/Full", version="0.9.0",
              set_return=SetReturn.STATE_AND_DIFF, test_return=TestReturn.STATE_AND_DIFF)
class _CliFullResource(DscResource[_CliTestSchema]):
    schema_provider = DataclassSchemaProvider(_CliTestSchema)
    def get(self, i): return i
    def set(self, i): return SetResult(actual_state=i, changed_properties=[])
    def test(self, i): return TestResult(actual_state=i, differing_properties=[])
    def delete(self, i): pass
    def export(self, i): return iter([])


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _write_pyproject(tmp_path: Path, entry_points: dict[str, str]) -> Path:
    ep_lines = "\n".join(f'"{k}" = "{v}"' for k, v in entry_points.items())
    content = f"""\
[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[project]
name = "test-pkg"
version = "0.1.0"

[project.entry-points."microsoft.dsc.resources"]
{ep_lines}
"""
    p = tmp_path / "pyproject.toml"
    p.write_text(content, encoding="utf-8")
    return p


# ---------------------------------------------------------------------------
# Tests using _generate_manifest directly (no filesystem import needed)
# ---------------------------------------------------------------------------

class TestDscGenManifest:
    def test_manifest_type(self):
        from ms_dsc.cli import _generate_manifest
        manifest = _generate_manifest("Cli/GetOnly", _CliGetOnlyResource)
        assert manifest["type"] == "Cli/GetOnly"

    def test_manifest_version(self):
        from ms_dsc.cli import _generate_manifest
        manifest = _generate_manifest("Cli/GetOnly", _CliGetOnlyResource)
        assert manifest["version"] == "1.2.3"

    def test_manifest_description(self):
        from ms_dsc.cli import _generate_manifest
        manifest = _generate_manifest("Cli/GetOnly", _CliGetOnlyResource)
        assert manifest["description"] == "CLI test get-only resource"

    def test_manifest_kind(self):
        from ms_dsc.cli import _generate_manifest
        manifest = _generate_manifest("Cli/GetOnly", _CliGetOnlyResource)
        assert manifest["kind"] == "resource"

    def test_manifest_require_adapter(self):
        from ms_dsc.cli import _generate_manifest
        manifest = _generate_manifest("Cli/GetOnly", _CliGetOnlyResource)
        assert manifest["requireAdapter"] == "Microsoft.Adapter/Python"

    def test_manifest_schema_embedded(self):
        from ms_dsc.cli import _generate_manifest
        manifest = _generate_manifest("Cli/GetOnly", _CliGetOnlyResource)
        assert "schema" in manifest
        assert manifest["schema"]["embedded"]["type"] == "object"

    def test_manifest_content_has_module_and_class(self):
        from ms_dsc.cli import _generate_manifest
        manifest = _generate_manifest("Cli/GetOnly", _CliGetOnlyResource)
        assert "content" in manifest
        assert isinstance(manifest["content"]["module"], str)
        assert isinstance(manifest["content"]["class"], str)
        assert len(manifest["content"]["module"]) > 0
        assert len(manifest["content"]["class"]) > 0

    def test_get_only_capabilities(self):
        from ms_dsc.cli import _generate_manifest
        manifest = _generate_manifest("Cli/GetOnly", _CliGetOnlyResource)
        assert manifest["capabilities"] == ["get"]

    def test_full_capabilities(self):
        from ms_dsc.cli import _generate_manifest
        manifest = _generate_manifest("Cli/Full", _CliFullResource)
        for cap in ("get", "set", "test", "delete", "export"):
            assert cap in manifest["capabilities"]

    def test_schema_url(self):
        from ms_dsc.cli import _generate_manifest
        manifest = _generate_manifest("Cli/GetOnly", _CliGetOnlyResource)
        assert "adaptedresource" in manifest["$schema"]

    def test_set_return_not_in_manifest_top_level(self):
        """set_return/test_return are NOT top-level keys in the manifest."""
        from ms_dsc.cli import _generate_manifest
        manifest = _generate_manifest("Cli/Full", _CliFullResource)
        assert "set_return" not in manifest
        assert "test_return" not in manifest


class TestDscGenMainCommand:
    def test_returns_1_when_pyproject_missing(self, tmp_path):
        from ms_dsc.cli import main
        rc = main(["manifest", "--pyproject", str(tmp_path / "does_not_exist.toml")])
        assert rc == 1

    def test_returns_1_when_no_entry_points(self, tmp_path):
        from ms_dsc.cli import main
        p = tmp_path / "pyproject.toml"
        p.write_text("[project]\nname = 'empty'\nversion = '0.0.0'\n")
        rc = main(["manifest", "--out", str(tmp_path / "out"), "--pyproject", str(p)])
        assert rc == 1

    def test_writes_file_for_valid_entry_point(self, tmp_path):
        from ms_dsc.cli import main

        ep = f"unit.test_cli:_CliGetOnlyResource"
        pyproject = _write_pyproject(tmp_path, {"Cli/GetOnly": ep})
        out_dir = tmp_path / "out"
        rc = main(["manifest", "--out", str(out_dir), "--pyproject", str(pyproject)])
        assert rc == 0
        assert (out_dir / "Cli.GetOnly.dsc.adaptedResource.json").exists()

    def test_filename_uses_dot_separator(self, tmp_path):
        from ms_dsc.cli import main

        ep = f"unit.test_cli:_CliGetOnlyResource"
        pyproject = _write_pyproject(tmp_path, {"Cli/GetOnly": ep})
        out_dir = tmp_path / "out"
        main(["manifest", "--out", str(out_dir), "--pyproject", str(pyproject)])
        assert (out_dir / "Cli.GetOnly.dsc.adaptedResource.json").exists()


# ---------------------------------------------------------------------------
# Tests for pyadapter CLI --content argument
# ---------------------------------------------------------------------------

class TestPyadapterCliContent:
    """Verify that --content JSON is parsed and forwarded to dispatch."""

    def _run(self, argv: list[str], stdin: str = "{}") -> int:
        import io
        from unittest.mock import patch
        from pyadapter.cli import main

        with patch("sys.stdin", io.StringIO(stdin)):
            with patch("sys.stdout", io.StringIO()):
                with patch("sys.stderr", io.StringIO()):
                    return main(argv)

    def test_content_forwarded_to_dispatch(self):
        """--content JSON is parsed and passed to router.dispatch as adapted_content."""
        import json
        from unittest.mock import patch, call
        from pyadapter.cli import main
        import io

        content = {"module": "unit.test_router", "class": "StateDiffResource"}
        content_json = json.dumps(content)

        captured = {}

        def fake_dispatch(operation, resource_type, stdin_json, adapted_content=None):
            captured["adapted_content"] = adapted_content
            return 0

        with patch("pyadapter.router.dispatch", fake_dispatch):
            with patch("sys.stdin", io.StringIO("{}")):
                with patch("sys.stdout", io.StringIO()):
                    main(["get", "--resource", "Any/Type", "--content", content_json])

        assert captured["adapted_content"] == content

    def test_invalid_content_json_returns_2(self):
        """Malformed --content JSON causes exit code 2."""
        rc = self._run(["get", "--resource", "Any/Type", "--content", "{not valid json}"])
        assert rc == 2

    def test_no_content_arg_passes_none(self):
        """When --content is omitted, adapted_content=None is passed to dispatch."""
        from unittest.mock import patch
        import io

        captured = {}

        def fake_dispatch(operation, resource_type, stdin_json, adapted_content=None):
            captured["adapted_content"] = adapted_content
            return 0

        with patch("pyadapter.router.dispatch", fake_dispatch):
            with patch("sys.stdin", io.StringIO("{}")):
                with patch("sys.stdout", io.StringIO()):
                    from pyadapter.cli import main
                    main(["get", "--resource", "Any/Type"])

        assert captured["adapted_content"] is None


# ---------------------------------------------------------------------------
# Additional CLI error handling and edge cases
# ---------------------------------------------------------------------------

class TestCliValidateCommand:
    """Test the validate command."""

    def test_validate_returns_zero(self, capsys):
        """validate verb should exit with 0."""
        from pyadapter.cli import main
        result = main(["validate"])
        assert result == 0
        captured = capsys.readouterr()
        data = json.loads(captured.out)
        assert data.get("valid") is True

    def test_validate_output_is_json(self, capsys):
        """validate output should be valid JSON."""
        from pyadapter.cli import main
        main(["validate"])
        captured = capsys.readouterr()
        data = json.loads(captured.out)
        assert isinstance(data, dict)
        assert "valid" in data


class TestCliListCommand:
    """Test the list command."""

    def test_list_verb_returns_code(self):
        """list command should return exit code."""
        from pyadapter.cli import main
        result = main(["list"])
        # list may return 0 or 1 depending on available resources
        assert result in (0, 1)


class TestCliHelpCommand:
    """Test help functionality."""

    def test_help_exits(self):
        """--help should trigger SystemExit with code 0."""
        from pyadapter.cli import main
        with pytest.raises(SystemExit) as exc_info:
            main(["--help"])
        assert exc_info.value.code == 0


class TestCliErrorHandling:
    """Test CLI error handling."""

    def test_no_arguments_raises_exit(self):
        """No arguments should raise SystemExit."""
        from pyadapter.cli import main
        with pytest.raises(SystemExit):
            main([])

    def test_invalid_verb_raises_exit(self):
        """Unknown verb should raise SystemExit."""
        from pyadapter.cli import main
        with pytest.raises(SystemExit):
            main(["invalid_verb_xyz"])

    def test_help_is_available(self):
        """Help should be available and exit cleanly."""
        from pyadapter.cli import main
        with pytest.raises(SystemExit) as exc_info:
            main(["-h"])
        assert exc_info.value.code == 0

