# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
pytest configuration for the Python DSC adapter unit tests.

Adds pyadapter to sys.path so tests can import it without a pip install.
This is test-only setup: in production, DSC invokes the adapter via
``python -m pyadapter.cli``, which adds '' (CWD) to sys.path[0] automatically.
ms-dsc is expected to be pip-installed in the test environment.
"""
from __future__ import annotations

import sys
from pathlib import Path

import pytest

# adapters/python is not a pip-installed package — add it to sys.path so that
# `import pyadapter` works from any working directory.
_ADAPTER_ROOT = Path(__file__).parent.parent.resolve()
if str(_ADAPTER_ROOT) not in sys.path:
    sys.path.insert(0, str(_ADAPTER_ROOT))

# Also ensure the fixture resource package path is available for integration tests.
_FIXTURE_ROOT = Path(__file__).parent / "fixture"
if str(_FIXTURE_ROOT) not in sys.path:
    sys.path.insert(0, str(_FIXTURE_ROOT))


@pytest.fixture(scope="session", autouse=True)
def regenerate_fixture_manifests() -> None:
    """
    Regenerate the dsc_test_resource manifests before the test session.

    The pre-built manifests in tests/fixture/dsc_test_resource/dsc/ contain an
    absolute ``path`` field that is machine-specific.  This session fixture
    regenerates them so they point to the correct path on the current machine,
    making the test suite portable across CI environments and developer machines.
    """
    try:
        from ms_dsc.cli import main as dsc_gen

        out_dir = str(_FIXTURE_ROOT / "dsc_test_resource" / "dsc")
        pyproject = str(_FIXTURE_ROOT / "pyproject.toml")
        dsc_gen(["manifest", "--out", out_dir, "--pyproject", pyproject])
    except Exception:
        # Non-fatal: unit tests mock entry points and do not read manifest files.
        # Only integration tests that test discovery need correct manifests.
        pass

