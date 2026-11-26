# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""Unit tests for the bundled ms_dsc SDK scenario.

These tests verify that when ms_dsc/ is present alongside pyadapter/ (as in a
bundled DSC package), importing it works without any pip install, and that the
pyadapter itself can be invoked via ``python -m pyadapter.cli``.
"""
from __future__ import annotations

import importlib
import subprocess
import sys
from pathlib import Path

import pytest

_ADAPTER_ROOT = Path(__file__).parent.parent.parent.resolve()
_BUNDLED_MSDSC = _ADAPTER_ROOT / "ms_dsc"


# ---------------------------------------------------------------------------
# Tests for bundled ms_dsc presence and importability
# ---------------------------------------------------------------------------

class TestBundledSdkDiscovery:
    """
    These tests are conditional: they run only when ms_dsc/ has been built
    (i.e. Copy-PythonAdapterSdk has run).  In a regular development environment
    ms_dsc is pip-installed, so these tests serve as smoke tests for the
    packaged/CI scenario.
    """

    @pytest.mark.skipif(
        not _BUNDLED_MSDSC.exists(),
        reason="Bundled ms_dsc/ not present; run build to generate it",
    )
    def test_bundled_msdsc_directory_structure(self):
        assert (_BUNDLED_MSDSC / "__init__.py").exists()
        assert (_BUNDLED_MSDSC / "resource.py").exists()
        assert (_BUNDLED_MSDSC / "metadata.py").exists()
        assert (_BUNDLED_MSDSC / "protocols.py").exists()
        assert (_BUNDLED_MSDSC / "schema" / "__init__.py").exists()

    @pytest.mark.skipif(
        not _BUNDLED_MSDSC.exists(),
        reason="Bundled ms_dsc/ not present; run build to generate it",
    )
    def test_bundled_msdsc_excludes_build_hook(self):
        build_dir = _BUNDLED_MSDSC / "build"
        assert not build_dir.exists(), (
            "bundled ms_dsc/ must not include the build/ hook (hatchling dependency)"
        )

    @pytest.mark.skipif(
        not _BUNDLED_MSDSC.exists(),
        reason="Bundled ms_dsc/ not present; run build to generate it",
    )
    def test_bundled_msdsc_includes_logging(self):
        assert (_BUNDLED_MSDSC / "logging.py").exists(), (
            "bundled ms_dsc/ must include logging.py"
        )


# ---------------------------------------------------------------------------
# Module invocation tests
# ---------------------------------------------------------------------------

class TestModuleInvocation:
    """Test that the adapter is invocable as ``python -m pyadapter.cli``."""

    def test_cli_module_help(self):
        """``python -m pyadapter.cli --help`` must exit 0 and print usage."""
        result = subprocess.run(
            [sys.executable, "-m", "pyadapter.cli", "--help"],
            capture_output=True,
            text=True,
            cwd=str(_ADAPTER_ROOT),
        )
        assert result.returncode == 0
        assert "pyadapter" in result.stdout or "usage" in result.stdout.lower()

    def test_cli_module_validate(self):
        """``python -m pyadapter.cli validate`` must return JSON with valid=true."""
        import json as _json

        result = subprocess.run(
            [sys.executable, "-m", "pyadapter.cli", "validate"],
            capture_output=True,
            text=True,
            cwd=str(_ADAPTER_ROOT),
        )
        assert result.returncode == 0
        data = _json.loads(result.stdout.strip())
        assert data.get("valid") is True

    def test_cli_module_list_runs(self):
        """``python -m pyadapter.cli list`` must exit cleanly (0 or 1)."""
        result = subprocess.run(
            [sys.executable, "-m", "pyadapter.cli", "list"],
            capture_output=True,
            text=True,
            cwd=str(_ADAPTER_ROOT),
        )
        assert result.returncode in (0, 1), (
            f"pyadapter list exited with unexpected code {result.returncode}"
        )

    def test_pyadapter_package_invocation(self):
        """``python -m pyadapter validate`` (via __main__.py) must also work."""
        import json as _json

        result = subprocess.run(
            [sys.executable, "-m", "pyadapter", "validate"],
            capture_output=True,
            text=True,
            cwd=str(_ADAPTER_ROOT),
        )
        assert result.returncode == 0
        data = _json.loads(result.stdout.strip())
        assert data.get("valid") is True


# ---------------------------------------------------------------------------
# CWD-based importability tests
# ---------------------------------------------------------------------------

class TestCwdBasedImportability:
    """
    Verify that the bundled ms_dsc/ directory is importable when python is
    invoked from the adapter root (simulating DSC's CWD = manifest directory
    behaviour with ``python -m pyadapter.cli``).
    """

    def test_ms_dsc_importable_from_cwd(self):
        """
        When CWD contains ms_dsc/, ``import ms_dsc`` works inside a -m invocation.
        The adapter root already contains ms-dsc/ (source) or ms_dsc/ (bundled).
        """
        # Check either the pip-installed version or the bundled copy is importable
        result = subprocess.run(
            [sys.executable, "-c", "import ms_dsc; print(ms_dsc.__file__)"],
            capture_output=True,
            text=True,
            cwd=str(_ADAPTER_ROOT),
        )
        assert result.returncode == 0, (
            f"ms_dsc is not importable from adapter root: {result.stderr}"
        )
