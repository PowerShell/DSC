# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Unit tests for pyadapter.__main__ — module entry point handling.
"""
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest


class TestMainModuleEntry:
    """Test that the __main__ module delegates correctly to cli.main()."""

    def test_main_module_invocation_help(self):
        """python -m pyadapter --help should work."""
        adapter_root = Path(__file__).parent.parent.parent.resolve()
        result = subprocess.run(
            [sys.executable, "-m", "pyadapter", "--help"],
            cwd=str(adapter_root),
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        assert "usage" in result.stdout.lower() or "pyadapter" in result.stdout.lower()

    def test_main_module_invocation_validate(self):
        """python -m pyadapter validate should return valid JSON."""
        import json

        adapter_root = Path(__file__).parent.parent.parent.resolve()
        result = subprocess.run(
            [sys.executable, "-m", "pyadapter", "validate"],
            cwd=str(adapter_root),
            capture_output=True,
            text=True,
        )
        assert result.returncode == 0
        data = json.loads(result.stdout.strip())
        assert data.get("valid") is True

    def test_main_module_invocation_list(self):
        """python -m pyadapter list should exit cleanly."""
        adapter_root = Path(__file__).parent.parent.parent.resolve()
        result = subprocess.run(
            [sys.executable, "-m", "pyadapter", "list"],
            cwd=str(adapter_root),
            capture_output=True,
            text=True,
        )
        # Exit 0 or 1 is acceptable (depends on whether resources exist)
        assert result.returncode in (0, 1)
