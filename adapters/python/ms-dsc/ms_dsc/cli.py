# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
dsc-gen — CLI tool for generating DSC adapted resource manifests.

Usage::

    dsc-gen manifest [--out <dir>] [--pyproject <path>]

Reads [project.entry-points."microsoft.dsc.resources"] from pyproject.toml,
imports each resource class, and writes a *.dsc.adaptedResource.json file per
resource into the output directory.

The output directory is included as package data so that the Python DSC
discovery extension can find the manifests without running the adapter.
"""
from __future__ import annotations

import argparse
import importlib
import json
import sys
import tomllib
from pathlib import Path


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="dsc-gen",
        description="Generate DSC adapted resource manifest files.",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    manifest_cmd = sub.add_parser(
        "manifest",
        help="Generate *.dsc.adaptedResource.json files from entry points.",
    )
    manifest_cmd.add_argument(
        "--out",
        default=None,
        metavar="DIR",
        help="Output directory (default: <package_name>/dsc/).",
    )
    manifest_cmd.add_argument(
        "--pyproject",
        default="pyproject.toml",
        metavar="FILE",
        help="Path to pyproject.toml (default: pyproject.toml in CWD).",
    )

    args = parser.parse_args(argv)
    if args.command == "manifest":
        return cmd_manifest(args)
    return 0


def cmd_manifest(args: argparse.Namespace) -> int:
    pyproject_path = Path(args.pyproject).resolve()
    if not pyproject_path.exists():
        print(f"error: {pyproject_path} not found", file=sys.stderr)
        return 1

    with pyproject_path.open("rb") as f:
        pyproject = tomllib.load(f)

    entry_points: dict[str, str] = (
        pyproject
        .get("project", {})
        .get("entry-points", {})
        .get("microsoft.dsc.resources", {})
    )
    if not entry_points:
        print(
            'error: no [project.entry-points."microsoft.dsc.resources"] section found',
            file=sys.stderr,
        )
        return 1

    if args.out:
        out_dir = Path(args.out)
    else:
        package_name = pyproject.get("project", {}).get("name", "").replace("-", "_")
        out_dir = Path(package_name) / "dsc" if package_name else Path("dsc")

    out_dir.mkdir(parents=True, exist_ok=True)

    for resource_type, ep_spec in entry_points.items():
        module_path, _, class_name = ep_spec.partition(":")
        try:
            module = importlib.import_module(module_path)
            cls = getattr(module, class_name)
        except Exception as exc:
            print(f"error: could not import {ep_spec}: {exc}", file=sys.stderr)
            return 1

        try:
            manifest = _generate_manifest(resource_type, cls)
        except Exception as exc:
            print(f"error: could not generate manifest for {resource_type}: {exc}", file=sys.stderr)
            return 1

        filename = resource_type.replace("/", ".") + ".dsc.adaptedResource.json"
        out_file = out_dir / filename
        out_file.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
        print(f"  generated {out_file}")

    return 0


def _capabilities_for(cls: type) -> list[str]:
    caps: list[str] = []
    try:
        from ms_dsc.protocols import Deletable, Exportable, Gettable, Settable, Testable

        probe = object.__new__(cls)
        if isinstance(probe, Gettable):
            caps.append("get")
        if isinstance(probe, Settable):
            caps.append("set")
        if isinstance(probe, Testable):
            caps.append("test")
        if isinstance(probe, Deletable):
            caps.append("delete")
        if isinstance(probe, Exportable):
            caps.append("export")
    except ImportError:
        pass
    return caps


def _generate_manifest(resource_type: str, cls: type) -> dict:
    metadata = getattr(cls, "__dsc_metadata__", None)

    version = metadata.version if metadata else "0.0.0"
    description = metadata.description if metadata else ""
    tags: list[str] = list(metadata.tags) if metadata else []
    set_return = metadata.set_return.value if metadata else "state"
    test_return = metadata.test_return.value if metadata else "state"

    capabilities = _capabilities_for(cls)

    schema: dict = {}
    if hasattr(cls, "get_schema") and callable(cls.get_schema):
        try:
            schema = cls.get_schema()
        except Exception as exc:
            print(f"  warning: schema generation failed for {resource_type}: {exc}", file=sys.stderr)

    # Use AdaptedPathOrContent::Content so DSC injects module+class into the
    # adapter call as --content, enabling direct class resolution without a
    # Python entry-point fallback.
    manifest: dict = {
        "$schema": "https://aka.ms/dsc/schemas/v3/bundled/adaptedresource/manifest.json",
        "type": resource_type,
        "kind": "resource",
        "version": version,
        "capabilities": capabilities,
        "requireAdapter": "Microsoft.Adapter/Python",
        "content": {
            "module": cls.__module__,
            "class": cls.__name__,
        },
    }
    if description:
        manifest["description"] = description
    if schema:
        manifest["schema"] = {"embedded": schema}

    return manifest
