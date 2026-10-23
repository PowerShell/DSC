# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""Unit tests for the Python adapter resource manifests.

Verifies that both the Windows manifest (python.dsc.resource.json)
and the Unix manifest (python3.dsc.resource.json) are structurally
correct and consistent with the bundled-SDK invocation design.
"""
from __future__ import annotations

import json
from pathlib import Path

import pytest

_ADAPTER_ROOT = Path(__file__).parent.parent.parent.resolve()
_MANIFEST_WINDOWS = _ADAPTER_ROOT / "python.dsc.resource.json"
_MANIFEST_UNIX = _ADAPTER_ROOT / "python3.dsc.resource.json"

_OPERATIONS = ("adapter.list", "get", "set", "test", "delete", "export", "validate")


def _load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


@pytest.fixture(scope="module")
def windows_manifest() -> dict:
    return _load(_MANIFEST_WINDOWS)


@pytest.fixture(scope="module")
def unix_manifest() -> dict:
    return _load(_MANIFEST_UNIX)


# ---------------------------------------------------------------------------
# Shared structure tests (parametrised over both manifests)
# ---------------------------------------------------------------------------

class TestManifestStructure:
    @pytest.fixture(params=["windows", "unix"])
    def manifest(self, request, windows_manifest, unix_manifest):
        return windows_manifest if request.param == "windows" else unix_manifest

    def test_schema_field_present(self, manifest):
        assert "$schema" in manifest

    def test_type_is_python_adapter(self, manifest):
        assert manifest["type"] == "Microsoft.Adapter/Python"

    def test_kind_is_adapter(self, manifest):
        assert manifest.get("kind") == "adapter"

    def test_has_condition(self, manifest):
        assert "condition" in manifest, "Manifest must declare a condition for graceful absence of Python"

    def test_condition_uses_trywhich(self, manifest):
        assert "tryWhich" in manifest["condition"]
        assert "null" in manifest["condition"]

    def test_has_exit_codes(self, manifest):
        assert "exitCodes" in manifest
        assert "0" in manifest["exitCodes"]

    def test_adapter_list_uses_module_invocation(self, manifest):
        list_cmd = manifest["adapter"]["list"]
        args = list_cmd["args"]
        assert args[0] == "-m", "adapter.list must use '-m pyadapter.cli' module invocation"
        assert args[1] == "pyadapter.cli"

    def test_adapter_list_does_not_use_script_path(self, manifest):
        list_cmd = manifest["adapter"]["list"]
        args = list_cmd["args"]
        assert not any("__main__" in a for a in args if isinstance(a, str)), (
            "manifest must not reference __main__.py directly"
        )

    @pytest.mark.parametrize("op", ["get", "set", "test", "delete", "export", "validate"])
    def test_operation_uses_module_invocation(self, manifest, op):
        if op not in manifest:
            pytest.skip(f"Operation '{op}' not present in manifest")
        args = manifest[op]["args"]
        assert args[0] == "-m"
        assert args[1] == "pyadapter.cli"

    @pytest.mark.parametrize("op", ["get", "set", "test", "delete", "export", "validate"])
    def test_operation_does_not_reference_main_py(self, manifest, op):
        if op not in manifest:
            pytest.skip(f"Operation '{op}' not present in manifest")
        args = manifest[op]["args"]
        assert not any("__main__" in a for a in args if isinstance(a, str))


# ---------------------------------------------------------------------------
# Platform-specific executable tests
# ---------------------------------------------------------------------------

class TestWindowsManifest:
    def test_executable_is_python(self, windows_manifest):
        assert windows_manifest["adapter"]["list"]["executable"] == "python"

    def test_condition_checks_python(self, windows_manifest):
        assert "python" in windows_manifest["condition"]
        # Must not require python3 (Windows uses 'python')
        cond = windows_manifest["condition"]
        assert "tryWhich('python')" in cond or 'tryWhich("python")' in cond

    def test_all_executables_are_python(self, windows_manifest):
        ops = ["get", "set", "test", "delete", "export", "validate"]
        for op in ops:
            if op in windows_manifest:
                assert windows_manifest[op]["executable"] == "python", f"'{op}' must use 'python'"


class TestUnixManifest:
    def test_executable_is_python3(self, unix_manifest):
        assert unix_manifest["adapter"]["list"]["executable"] == "python3"

    def test_condition_checks_python3(self, unix_manifest):
        cond = unix_manifest["condition"]
        assert "tryWhich('python3')" in cond or 'tryWhich("python3")' in cond

    def test_type_matches_windows_manifest(self, unix_manifest, windows_manifest):
        assert unix_manifest["type"] == windows_manifest["type"], (
            "Both manifests must declare the same resource type"
        )

    def test_all_executables_are_python3(self, unix_manifest):
        ops = ["get", "set", "test", "delete", "export", "validate"]
        for op in ops:
            if op in unix_manifest:
                assert unix_manifest[op]["executable"] == "python3", f"'{op}' must use 'python3'"
