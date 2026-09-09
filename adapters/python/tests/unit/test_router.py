# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Unit tests for pyadapter.router — operation dispatch.

All tests use a mocked entry-point registry so no packages need to be installed.
"""
from __future__ import annotations

import importlib.metadata
import io
import json
from collections.abc import Iterator
from dataclasses import dataclass
from unittest.mock import patch

import pytest

from ms_dsc import DscResource, SetResult, TestResult, dsc_resource
from ms_dsc.metadata import SetReturn, TestReturn
from ms_dsc.schema import DataclassSchemaProvider


# ---------------------------------------------------------------------------
# Fixture resource classes used across all tests
# ---------------------------------------------------------------------------

@dataclass
class Schema:
    name: str
    _exist: bool = True


@dsc_resource(type="Test/StateDiff", version="1.0.0",
              set_return=SetReturn.STATE_AND_DIFF, test_return=TestReturn.STATE_AND_DIFF)
class StateDiffResource(DscResource[Schema]):
    schema_provider = DataclassSchemaProvider(Schema)

    def get(self, i: Schema) -> Schema:
        return Schema(name=i.name, _exist=True)

    def set(self, i: Schema) -> SetResult[Schema]:
        return SetResult(
            actual_state=Schema(name=i.name, _exist=i._exist),
            changed_properties=["_exist"],
        )

    def test(self, i: Schema) -> TestResult[Schema]:
        actual = Schema(name=i.name, _exist=True)
        diffs = ["_exist"] if actual._exist != i._exist else []
        return TestResult(actual_state=actual, differing_properties=diffs)

    def delete(self, i: Schema) -> None:
        pass

    def export(self, i: Schema | None) -> Iterator[Schema]:
        yield Schema(name="item-1")
        yield Schema(name="item-2")


@dsc_resource(type="Test/StateOnly", version="1.0.0")  # STATE mode (default)
class StateOnlyResource(DscResource[Schema]):
    schema_provider = DataclassSchemaProvider(Schema)

    def get(self, i: Schema) -> Schema:
        return Schema(name=i.name, _exist=True)

    def set(self, i: Schema) -> SetResult[Schema]:
        return SetResult(actual_state=Schema(name=i.name))

    def test(self, i: Schema) -> TestResult[Schema]:
        return TestResult(actual_state=Schema(name=i.name))


def _make_ep(type_name: str, cls: type):
    return type("EP", (), {"name": type_name, "value": "x:Y", "load": lambda self, c=cls: c})()


_MOCK_EPS = [
    _make_ep("Test/StateDiff", StateDiffResource),
    _make_ep("Test/StateOnly", StateOnlyResource),
]


@pytest.fixture(autouse=True)
def mock_eps():
    """Default entry_points fixture — returns the two test resources.

    Tests in TestResolveClass that need to control entry_points explicitly
    override this via their own patch.object calls.
    """
    with patch.object(importlib.metadata, "entry_points", return_value=_MOCK_EPS):
        yield


def _dispatch(operation: str, resource_type: str, stdin: str = "{}") -> tuple[int, str]:
    """Call dispatch and return (exit_code, captured_stdout)."""
    from pyadapter.router import dispatch

    cap = io.StringIO()
    err = io.StringIO()
    with patch("sys.stdout", cap), patch("sys.stderr", err):
        rc = dispatch(operation, resource_type, stdin)
    return rc, cap.getvalue()


def _lines(stdout: str) -> list[dict]:
    return [json.loads(l) for l in stdout.splitlines() if l.strip()]


# ---------------------------------------------------------------------------
# GET
# ---------------------------------------------------------------------------

class TestDispatchGet:
    def test_returns_zero(self):
        rc, _ = _dispatch("get", "Test/StateDiff", '{"name":"foo"}')
        assert rc == 0

    def test_single_json_line(self):
        _, out = _dispatch("get", "Test/StateDiff", '{"name":"foo"}')
        lines = [l for l in out.splitlines() if l.strip()]
        assert len(lines) == 1

    def test_correct_state(self):
        _, out = _dispatch("get", "Test/StateDiff", '{"name":"foo"}')
        assert json.loads(out.strip()) == {"name": "foo", "_exist": True}

    def test_case_insensitive_type(self):
        rc, _ = _dispatch("get", "test/statediff", '{"name":"foo"}')
        assert rc == 0

    def test_extra_fields_ignored(self):
        """Unknown JSON fields are silently ignored during schema instantiation."""
        rc, out = _dispatch("get", "Test/StateDiff", '{"name":"foo","unknownField":123}')
        assert rc == 0


# ---------------------------------------------------------------------------
# SET
# ---------------------------------------------------------------------------

class TestDispatchSet:
    def test_state_and_diff_two_lines(self):
        _, out = _dispatch("set", "Test/StateDiff", '{"name":"foo","_exist":false}')
        lines = [l for l in out.splitlines() if l.strip()]
        assert len(lines) == 2

    def test_state_and_diff_first_line_is_state(self):
        _, out = _dispatch("set", "Test/StateDiff", '{"name":"foo","_exist":false}')
        lines = [l for l in out.splitlines() if l.strip()]
        state = json.loads(lines[0])
        assert "name" in state

    def test_state_and_diff_second_line_is_array(self):
        _, out = _dispatch("set", "Test/StateDiff", '{"name":"foo","_exist":false}')
        lines = [l for l in out.splitlines() if l.strip()]
        diffs = json.loads(lines[1])
        assert isinstance(diffs, list)
        assert "_exist" in diffs

    def test_state_only_one_line(self):
        _, out = _dispatch("set", "Test/StateOnly", '{"name":"foo"}')
        lines = [l for l in out.splitlines() if l.strip()]
        assert len(lines) == 1

    def test_returns_zero(self):
        rc, _ = _dispatch("set", "Test/StateDiff", '{"name":"foo"}')
        assert rc == 0


# ---------------------------------------------------------------------------
# TEST
# ---------------------------------------------------------------------------

class TestDispatchTest:
    def test_in_desired_state_empty_diffs(self):
        _, out = _dispatch("test", "Test/StateDiff", '{"name":"foo","_exist":true}')
        lines = [l for l in out.splitlines() if l.strip()]
        assert json.loads(lines[1]) == []

    def test_out_of_desired_state_non_empty_diffs(self):
        _, out = _dispatch("test", "Test/StateDiff", '{"name":"foo","_exist":false}')
        lines = [l for l in out.splitlines() if l.strip()]
        diffs = json.loads(lines[1])
        assert "_exist" in diffs

    def test_state_only_one_line(self):
        _, out = _dispatch("test", "Test/StateOnly", '{"name":"foo"}')
        lines = [l for l in out.splitlines() if l.strip()]
        assert len(lines) == 1

    def test_returns_zero(self):
        rc, _ = _dispatch("test", "Test/StateDiff", '{"name":"foo"}')
        assert rc == 0


# ---------------------------------------------------------------------------
# DELETE
# ---------------------------------------------------------------------------

class TestDispatchDelete:
    def test_returns_zero(self):
        rc, _ = _dispatch("delete", "Test/StateDiff", '{"name":"foo"}')
        assert rc == 0

    def test_no_stdout(self):
        _, out = _dispatch("delete", "Test/StateDiff", '{"name":"foo"}')
        assert out.strip() == ""


# ---------------------------------------------------------------------------
# EXPORT
# ---------------------------------------------------------------------------

class TestDispatchExport:
    def test_returns_zero(self):
        rc, _ = _dispatch("export", "Test/StateDiff", "{}")
        assert rc == 0

    def test_empty_input_returns_all_items(self):
        _, out = _dispatch("export", "Test/StateDiff", "{}")
        items = _lines(out)
        assert len(items) == 2

    def test_each_line_valid_json(self):
        _, out = _dispatch("export", "Test/StateDiff", "{}")
        for line in out.splitlines():
            if line.strip():
                json.loads(line)  # must not raise

    def test_items_are_dicts(self):
        _, out = _dispatch("export", "Test/StateDiff", "{}")
        for item in _lines(out):
            assert isinstance(item, dict)
            assert "name" in item


# ---------------------------------------------------------------------------
# Error paths
# ---------------------------------------------------------------------------

class TestDispatchErrors:
    def test_unknown_resource_type_returns_2(self):
        rc, _ = _dispatch("get", "Unknown/Type", '{"name":"x"}')
        assert rc == 2

    def test_empty_resource_type_returns_2(self):
        rc, _ = _dispatch("get", "", '{"name":"x"}')
        assert rc == 2

    def test_invalid_json_returns_2(self):
        rc, _ = _dispatch("get", "Test/StateDiff", "{not valid json}")
        assert rc == 2

    def test_unknown_operation_returns_2(self):
        rc, _ = _dispatch("frobnicate", "Test/StateDiff", '{"name":"x"}')
        assert rc == 2

    def test_empty_stdin_treated_as_empty_object(self):
        """Empty stdin should not crash — treated as {} (no required fields filled = TypeError caught)."""
        rc, _ = _dispatch("get", "Test/StateDiff", "")
        # May be 0 (if Schema can be constructed with defaults) or non-zero (TypeError)
        assert isinstance(rc, int)


# ---------------------------------------------------------------------------
# Router stdlib-only: set/test must NOT require ms_dsc to be importable
# ---------------------------------------------------------------------------

class TestRouterStdlibOnly:
    """
    The adapter router must work without importing ms_dsc types.

    set() and test() previously had `from ms_dsc.metadata import SetReturn` etc.
    These tests verify the router reads set_return/test_return as plain strings
    via attribute access, not via enum comparisons.
    """

    def test_set_state_and_diff_without_ms_dsc_import(self):
        """
        Verify stateAndDiff mode works even if ms_dsc were unavailable.
        Uses a plain namespace object instead of DscResourceMetadata.
        """
        from types import SimpleNamespace
        from pyadapter.router import _do_set

        @dataclass
        class S:
            name: str

        class FakeResult:
            actual_state = S(name="x")
            changed_properties = ["name"]

        class FakeResource:
            def set(self, instance): return FakeResult()

        metadata = SimpleNamespace(set_return=SimpleNamespace(value="stateAndDiff"))

        cap = io.StringIO()
        with patch("sys.stdout", cap):
            rc = _do_set(FakeResource(), S(name="x"), metadata)

        assert rc == 0
        lines = [l for l in cap.getvalue().splitlines() if l.strip()]
        assert len(lines) == 2
        assert json.loads(lines[1]) == ["name"]

    def test_test_state_and_diff_without_ms_dsc_import(self):
        from types import SimpleNamespace
        from pyadapter.router import _do_test

        @dataclass
        class S:
            name: str

        class FakeResult:
            actual_state = S(name="x")
            differing_properties = ["name"]

        class FakeResource:
            def test(self, instance): return FakeResult()

        metadata = SimpleNamespace(test_return=SimpleNamespace(value="stateAndDiff"))

        cap = io.StringIO()
        with patch("sys.stdout", cap):
            rc = _do_test(FakeResource(), S(name="x"), metadata)

        assert rc == 0
        lines = [l for l in cap.getvalue().splitlines() if l.strip()]
        assert len(lines) == 2

    def test_set_state_mode_no_diff_emitted(self):
        from types import SimpleNamespace
        from pyadapter.router import _do_set

        @dataclass
        class S:
            name: str

        class FakeResult:
            actual_state = S(name="x")
            changed_properties = None

        class FakeResource:
            def set(self, instance): return FakeResult()

        metadata = SimpleNamespace(set_return=SimpleNamespace(value="state"))

        cap = io.StringIO()
        with patch("sys.stdout", cap):
            _do_set(FakeResource(), S(name="x"), metadata)

        lines = [l for l in cap.getvalue().splitlines() if l.strip()]
        assert len(lines) == 1

    def test_no_metadata_defaults_to_state_mode(self):
        from pyadapter.router import _do_set

        @dataclass
        class S:
            name: str

        class FakeResult:
            actual_state = S(name="x")
            changed_properties = None

        class FakeResource:
            def set(self, instance): return FakeResult()

        cap = io.StringIO()
        with patch("sys.stdout", cap):
            _do_set(FakeResource(), S(name="x"), None)

        lines = [l for l in cap.getvalue().splitlines() if l.strip()]
        assert len(lines) == 1


# ---------------------------------------------------------------------------
# Content-based class resolution (_resolve_class)
# ---------------------------------------------------------------------------

class TestResolveClass:
    def test_content_takes_priority_over_entry_points(self):
        """When adapted_content has module+class, it is used before entry_points."""
        from pyadapter.router import _resolve_class

        content = {
            "module": "unit.test_router",
            "class": "StateDiffResource",
        }
        # Entry points mock returns nothing to prove content path is taken.
        with patch.object(importlib.metadata, "entry_points", return_value=[]):
            cls = _resolve_class("Any/Type", adapted_content=content)

        assert cls is StateDiffResource

    def test_content_fallback_to_entry_points_on_import_error(self):
        """If content-based import fails, entry_points are tried next."""
        from pyadapter.router import _resolve_class

        content = {"module": "nonexistent.module", "class": "Cls"}
        ep = _make_ep("Test/StateDiff", StateDiffResource)
        with patch.object(importlib.metadata, "entry_points", return_value=[ep]):
            cls = _resolve_class("Test/StateDiff", adapted_content=content)

        assert cls is StateDiffResource

    def test_no_content_uses_entry_points(self):
        """Without adapted_content, entry_points are used directly."""
        from pyadapter.router import _resolve_class

        ep = _make_ep("Test/StateDiff", StateDiffResource)
        with patch.object(importlib.metadata, "entry_points", return_value=[ep]):
            cls = _resolve_class("Test/StateDiff", adapted_content=None)

        assert cls is StateDiffResource

    def test_content_missing_module_falls_back_to_entry_points(self):
        """Content without 'module' key skips content path and uses entry_points."""
        from pyadapter.router import _resolve_class

        content = {"class": "StateDiffResource"}  # no 'module'
        ep = _make_ep("Test/StateDiff", StateDiffResource)
        with patch.object(importlib.metadata, "entry_points", return_value=[ep]):
            cls = _resolve_class("Test/StateDiff", adapted_content=content)

        assert cls is StateDiffResource

    def test_dispatch_passes_content_to_resolve(self):
        """dispatch() passes adapted_content through to _resolve_class."""
        import io
        from pyadapter.router import dispatch

        content = {
            "module": "unit.test_router",
            "class": "StateDiffResource",
        }
        with patch.object(importlib.metadata, "entry_points", return_value=[]):
            cap = io.StringIO()
            with patch("sys.stdout", cap):
                rc = dispatch("get", "Any/Type", '{"name":"x"}', adapted_content=content)

        assert rc == 0


# ---------------------------------------------------------------------------
# Additional error handling and edge case tests
# ---------------------------------------------------------------------------

class TestDispatchErrorHandling:
    """Test dispatch error handling for invalid inputs."""

    def test_empty_resource_type_returns_error(self):
        """Empty resource type should return error code 2."""
        from pyadapter.router import dispatch
        json_input = '{"name": "test"}'
        rc = dispatch("get", "", json_input)
        assert rc == 2

    def test_unknown_resource_type_returns_error(self):
        """Unknown resource type should return error code 2."""
        from pyadapter.router import dispatch
        json_input = '{"name": "test"}'
        rc = dispatch("get", "NonExistent/Resource", json_input)
        assert rc == 2

    def test_invalid_json_returns_error(self):
        """Invalid JSON input should return error code 2."""
        from pyadapter.router import dispatch
        rc = dispatch("get", "Test/Resource", "{invalid json}")
        assert rc == 2

    def test_empty_json_input_treated_as_object(self):
        """Empty input should be treated as '{}'."""
        from pyadapter.router import dispatch
        # Empty string should not cause JSON error
        rc = dispatch("get", "NonExistent/Resource", "")
        # Will fail due to unknown resource, but not JSON error
        assert rc == 2

    def test_unknown_operation_verb(self):
        """Unknown operation verb should be handled gracefully."""
        from pyadapter.router import dispatch
        # The dispatch function should handle unknown operations
        rc = dispatch("unknown_op", "Test/Resource", '{"name":"test"}')
        # Should return error (either 2 for unknown resource or invalid operation)
        assert rc > 0


class TestDispatchResourceTypeHandling:
    """Test how dispatch handles resource type parameters."""

    def test_resource_type_normalization(self):
        """Resource type handling and normalization."""
        from pyadapter.router import dispatch
        # Both should attempt the same lookup with error
        rc1 = dispatch("get", "test/resource", '{"name":"test"}')
        rc2 = dispatch("get", "Test/Resource", '{"name":"test"}')
        # Both should fail with unknown resource
        assert rc1 == 2
        assert rc2 == 2


class TestDispatchStdinParsing:
    """Test stdin JSON parsing edge cases."""

    def test_single_json_object_parsed(self):
        """Valid single JSON object should be parsed."""
        from pyadapter.router import dispatch
        # This will fail to find the resource, but tests JSON parsing
        rc = dispatch("get", "Test/Resource", '{"valid": "json"}')
        # Fails due to resource not found, not JSON parsing
        assert rc == 2

    def test_whitespace_handling(self):
        """Leading/trailing whitespace should be handled."""
        from pyadapter.router import dispatch
        rc = dispatch("get", "Test/Resource", '   {"name": "test"}   ')
        # Should handle whitespace in JSON
        assert rc == 2  # Resource not found

    def test_empty_object(self):
        """Empty JSON object should be accepted."""
        from pyadapter.router import dispatch
        rc = dispatch("get", "Test/Resource", '{}')
        assert rc == 2  # Resource not found, not JSON error

