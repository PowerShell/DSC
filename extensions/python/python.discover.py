# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Python DSC discovery extension.

Iterates installed Python distributions and emits the absolute path of every
DSC resource manifest found in a distribution's 'dsc' subpackage as NDJSON
to stdout for the DSC engine to load.

Each output line is: {"manifestPath": "/absolute/path/to/manifest.json"}

Discovery uses importlib.resources, which correctly handles regular wheels,
editable installs (PEP 660 / legacy setuptools), and namespace packages.
No caching is performed; the importlib.resources lookup is fast at startup.
"""
from __future__ import annotations

import argparse
import importlib.metadata
import importlib.resources
import json
import sys
from pathlib import Path

_MANIFEST_SUFFIXES = (
    ".dsc.adaptedresource.json",
    ".dsc.resource.json",
)


def _manifests_for_dist(dist, manifest_suffixes: tuple[str, ...]) -> list[dict[str, str]]:
    """
    Return manifest path entries for a single distribution.

    Distribution names are normalised (hyphens/dots → underscores) to derive
    the importable 'dsc' subpackage name, e.g. 'my-package' → 'my_package.dsc'.
    """
    found: list[dict[str, str]] = []
    pkg_name = dist.name.replace("-", "_").replace(".", "_")
    dsc_pkg = f"{pkg_name}.dsc"

    try:
        dsc_ref = importlib.resources.files(dsc_pkg)
    except (ModuleNotFoundError, TypeError):
        return []

    try:
        for resource in dsc_ref.iterdir():
            if any(resource.name.lower().endswith(s) for s in manifest_suffixes):
                try:
                    abs_path = Path(str(resource)).resolve()
                    if abs_path.exists():
                        found.append({"manifestPath": str(abs_path)})
                except Exception:
                    pass
    except Exception:
        pass

    return found


def main() -> int:
    parser = argparse.ArgumentParser(description="Python DSC discovery extension")
    parser.add_argument("--file-extensions", help="Comma-separated list of manifest file extensions")
    args = parser.parse_args()
    
    manifest_suffixes = tuple(f".{ext.lstrip('.')}" for ext in args.file_extensions.split(",")) if args.file_extensions else ()
    
    for dist in importlib.metadata.distributions():
        for entry in _manifests_for_dist(dist, manifest_suffixes):
            print(json.dumps(entry), flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
