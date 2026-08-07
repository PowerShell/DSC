# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Adapter list operation and cache management for the Python DSC adapter.

Two discovery paths:

  1. Pre-built manifests (fast, preferred)
     The discovery *extension* (extensions/python/python.discover.py) handles
     this path by using importlib.resources to locate package_name/dsc/*.dsc.adaptedResource.json
     files.  The adapter list command covers only resources that have NO pre-built
     manifest — packages that registered entry points but did not run `dsc-gen manifest`.

  2. Runtime entry-point enumeration (fallback)
     Iterates importlib.metadata entry_points(group="microsoft.dsc.resources"),
     loads each class, introspects capabilities and schema, then emits a DSC
     list entry.  Results are cached by distribution fingerprint.
"""
from __future__ import annotations

import importlib.metadata
import importlib.resources
import json
import logging
from pathlib import Path

from ms_dsc.protocols import Deletable, Exportable, Gettable, Settable, Testable

from pyadapter.cache import LIST_CACHE, dist_fingerprint

_logger = logging.getLogger(__name__)

_MANIFEST_SUFFIXES = (
    ".dsc.adaptedresource.json",
    ".dsc.resource.json",
)

_ADAPTER_TYPE = "Microsoft.Adapter/Python"


def _iter_manifest_paths() -> list[dict]:
    """
    Iterate installed distributions and yield manifest paths for any that have
    a 'dsc' subpackage containing *.dsc.adaptedResource.json files.

    Uses importlib.resources for reliable cross-install-mode path resolution
    (regular wheels, editable installs, zip imports).  Distribution names are
    normalised (hyphens → underscores) to derive the importable package name.
    """
    results: list[dict] = []
    for dist in importlib.metadata.distributions():
        results.extend(_manifests_for_dist(dist))
    return results


def _manifests_for_dist(dist) -> list[dict]:
    """Return manifest path entries for a single distribution."""
    found: list[dict] = []

    # Derive the importable package name from the distribution name.
    # PEP 503 normalisation: hyphens and dots become underscores.
    pkg_name = dist.name.replace("-", "_").replace(".", "_")
    dsc_pkg = f"{pkg_name}.dsc"

    try:
        dsc_ref = importlib.resources.files(dsc_pkg)
    except (ModuleNotFoundError, TypeError):
        # Package has no 'dsc' subpackage — not a DSC resource package.
        return []

    try:
        for resource in dsc_ref.iterdir():
            name_lower = resource.name.lower()
            if any(name_lower.endswith(s) for s in _MANIFEST_SUFFIXES):
                try:
                    # Materialise to a real filesystem path when possible;
                    # fall back to reading via the resource API.
                    abs_path = Path(str(resource)).resolve()
                    if abs_path.exists():
                        found.append({"manifestPath": str(abs_path)})
                except Exception as exc:
                    _logger.debug("Could not resolve path for %s in %s: %s", resource.name, dsc_pkg, exc)
    except Exception as exc:
        _logger.debug("Could not list resources in %s: %s", dsc_pkg, exc)

    return found


def _covered_types() -> set[str]:
    """
    Return the set of resource type names (lower-cased) that already have a
    pre-built adapted resource manifest discoverable via importlib.resources.
    No caching: importlib.resources lookup is fast enough for startup.
    """
    covered: set[str] = set()
    for entry in _iter_manifest_paths():
        path = Path(entry["manifestPath"])
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
            if resource_type := data.get("type"):
                covered.add(resource_type.casefold())
        except Exception as exc:
            _logger.debug("Could not read manifest %s: %s", path, exc)
    return covered


def _capabilities_for(cls: type) -> list[str]:
    """Determine DSC capabilities by checking Protocol membership."""
    caps: list[str] = []
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
    return caps


def _source_path_for(cls: type) -> str:
    """Resolve the source file path for a resource class."""
    try:
        import importlib.util as _ilu
        spec = _ilu.find_spec(cls.__module__)
        if spec and spec.origin:
            return spec.origin
    except Exception:
        pass
    return f"{cls.__module__}:{cls.__qualname__}"


def _build_list_entry(type_name: str, cls: type) -> dict:
    """Generate a DSC adapter list entry from a resource class."""
    metadata = getattr(cls, "__dsc_metadata__", None)

    version = metadata.version if metadata else "0.0.0"
    description = metadata.description if metadata else ""
    capabilities = _capabilities_for(cls)

    # DscResource requires path + directory (non-optional).
    # adaptedContent is set so DSC injects module+class into the adapter call
    # via --content, enabling direct class resolution without entry-point fallback.
    src_path = _source_path_for(cls)
    entry: dict = {
        "type": type_name,
        "kind": "resource",
        "version": version,
        "capabilities": capabilities,
        "requireAdapter": _ADAPTER_TYPE,
        "path": src_path,
        "directory": str(Path(src_path).parent),
        "adaptedContent": {
            "module": cls.__module__,
            "class": cls.__name__,
        },
    }
    if description:
        entry["description"] = description

    if hasattr(cls, "get_schema") and callable(cls.get_schema):
        try:
            schema = cls.get_schema()
            if schema:
                entry["schema"] = {"embedded": schema}
        except Exception as exc:
            _logger.debug("Could not generate schema for %s: %s", type_name, exc)

    return entry


def _list_from_entry_points(covered: set[str]) -> list[dict]:
    """Enumerate entry points and return list entries for uncovered resources."""
    results: list[dict] = []
    try:
        eps = importlib.metadata.entry_points(group="microsoft.dsc.resources")
    except Exception as exc:
        _logger.error("Failed to enumerate entry points: %s", exc)
        return results

    for ep in eps:
        if ep.name.casefold() in covered:
            _logger.debug("Skipping %s — covered by pre-built manifest", ep.name)
            continue
        try:
            cls = ep.load()
            results.append(_build_list_entry(ep.name, cls))
            _logger.debug("Listed %s from entry point", ep.name)
        except Exception as exc:
            _logger.warning("Failed to load entry point %s: %s", ep.name, exc)
    return results


def cmd_discover() -> int:
    """
    Emit pre-built manifest paths as NDJSON ({"manifestPath": "..."}).
    This mirrors what extensions/python/python.discover.py does but is
    available directly through the adapter for convenience.
    """
    for entry in _iter_manifest_paths():
        print(json.dumps(entry), flush=True)
    return 0


def cmd_list() -> int:
    """
    Emit list entries for resources NOT covered by pre-built manifests.
    Results are cached; cache is invalidated when the installed package set changes.
    """
    fingerprint = dist_fingerprint()
    entries = LIST_CACHE.load(fingerprint)
    if entries is None:
        covered = _covered_types()
        entries = _list_from_entry_points(covered)
        LIST_CACHE.save(fingerprint, entries)

    for entry in entries:
        print(json.dumps(entry), flush=True)
    return 0


def cmd_clear_cache() -> int:
    LIST_CACHE.clear()
    _logger.info("Python DSC adapter list cache cleared")
    return 0


    for ep in eps:
        if ep.name.casefold() in covered:
            _logger.debug("Skipping %s — covered by pre-built manifest", ep.name)
            continue
        try:
            cls = ep.load()
            results.append(_build_list_entry(ep.name, cls))
            _logger.debug("Listed %s from entry point", ep.name)
        except Exception as exc:
            _logger.warning("Failed to load entry point %s: %s", ep.name, exc)
    return results


def cmd_discover() -> int:
    """
    Emit pre-built manifest paths as NDJSON ({"manifestPath": "..."}).
    This mirrors what extensions/python/python.discover.py does but is
    available directly through the adapter for convenience.
    """
    fingerprint = dist_fingerprint()
    manifests = DISCOVER_CACHE.load(fingerprint)
    if manifests is None:
        manifests = _scan_distributions()
        DISCOVER_CACHE.save(fingerprint, manifests)

    for entry in manifests:
        print(json.dumps(entry), flush=True)
    return 0


def cmd_list() -> int:
    """
    Emit list entries for resources NOT covered by pre-built manifests.
    Results are cached; cache is invalidated when the installed package set changes.
    """
    fingerprint = dist_fingerprint()
    entries = LIST_CACHE.load(fingerprint)
    if entries is None:
        covered = _covered_types()
        entries = _list_from_entry_points(covered)
        LIST_CACHE.save(fingerprint, entries)

    for entry in entries:
        print(json.dumps(entry), flush=True)
    return 0


def cmd_clear_cache() -> int:
    LIST_CACHE.clear()
    _logger.info("Python DSC adapter list cache cleared")
    return 0
