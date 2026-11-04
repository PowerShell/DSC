# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Hatchling build hook for dsc-gen manifest generation.

Add to a resource package's pyproject.toml::

    [build-system]
    requires = ["hatchling", "ms-dsc"]
    build-backend = "hatchling.build"

    [tool.hatch.build.hooks.dsc]
    # output_dir defaults to <package_name>/dsc/
    # output_dir = "my_package/dsc"

The hook reads [project.entry-points."microsoft.dsc.resources"], generates
*.dsc.adaptedResource.json files, and includes them in the built wheel.

The hook is registered as the ``dsc`` build hook plugin under the
``hatch.build.hook`` entry-point group.
"""
from __future__ import annotations

from pathlib import Path
from typing import Any


class DscManifestBuildHook:
    """
    Hatchling build hook that runs ``dsc-gen manifest`` before packaging.

    Registered via the ``hatch.build.hook`` entry-point group so that
    packages can activate it with ``[tool.hatch.build.hooks.dsc]``.
    """

    PLUGIN_NAME = "dsc"

    def __init__(self, root: str, config: dict[str, Any], build_config: Any, version_api: Any) -> None:
        self.root = Path(root)
        self.config = config
        self.build_config = build_config

    def initialize(self, version: str, build_data: dict[str, Any]) -> None:
        import argparse

        from ms_dsc.cli import cmd_manifest

        out_dir: str | None = self.config.get("output_dir")

        class _Args(argparse.Namespace):
            out = out_dir
            pyproject = str(self.root / "pyproject.toml")

        result = cmd_manifest(_Args())
        if result != 0:
            raise RuntimeError("dsc-gen manifest failed; see output above")

        # Resolve actual output directory to include generated files in the wheel.
        import tomllib

        pyproject_path = self.root / "pyproject.toml"
        with pyproject_path.open("rb") as f:
            pyproject = tomllib.load(f)

        if out_dir is None:
            package_name = pyproject.get("project", {}).get("name", "").replace("-", "_")
            out_dir = f"{package_name}/dsc" if package_name else "dsc"

        out_path = self.root / out_dir
        for manifest_file in out_path.glob("*.dsc.adaptedResource.json"):
            # force_include: {local_abs_path: path_inside_wheel}
            wheel_path = str(manifest_file.relative_to(self.root))
            build_data.setdefault("force_include", {})[str(manifest_file)] = wheel_path

    def finalize(self, version: str, build_data: dict[str, Any], artifact_path: str) -> None:
        pass
