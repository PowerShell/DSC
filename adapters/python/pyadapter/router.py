# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Operation router for the Python DSC adapter.

Resolves a resource class from the `microsoft.dsc.resources` entry-point group,
instantiates the resource and its schema type from the JSON input, dispatches
to the appropriate Protocol method, and formats stdout per DSC's adapter contract.

stdout contract (single-config mode):
  get    → one JSON line: actual state dict
  set    → one line (state only) OR two lines (state + changed_properties)
  test   → one line (state only) OR two lines (state + differing_properties)
  delete → no stdout
  export → zero or more JSON lines, one per exported instance
"""
from __future__ import annotations

import importlib
import importlib.metadata
import json
import logging
import sys
from typing import Any

from ms_dsc.schema import build_schema_instance, to_dict

_logger = logging.getLogger(__name__)


def dispatch(operation: str, resource_type: str, stdin_json: str, adapted_content: dict | None = None) -> int:
    try:
        cls = _resolve_class(resource_type, adapted_content)
    except (ValueError, RuntimeError) as exc:
        print(json.dumps({"error": str(exc)}), file=sys.stderr)
        return 2

    try:
        data: dict[str, Any] = json.loads(stdin_json.strip() or "{}")
    except json.JSONDecodeError as exc:
        print(json.dumps({"error": f"Invalid JSON input: {exc}"}), file=sys.stderr)
        return 2

    try:
        resource = cls()
    except Exception as exc:
        _logger.error("Failed to instantiate %s: %s", resource_type, exc)
        print(json.dumps({"error": str(exc)}), file=sys.stderr)
        return 1

    metadata = getattr(cls, "__dsc_metadata__", None)

    try:
        # Export is the only operation where an empty input means "no filter" (None).
        if operation == "export":
            instance = build_schema_instance(cls, data) if data else None
            return _do_export(resource, instance)

        schema_instance = build_schema_instance(cls, data)

        if operation == "get":
            return _do_get(resource, schema_instance)
        if operation == "set":
            return _do_set(resource, schema_instance, metadata)
        if operation == "test":
            return _do_test(resource, schema_instance, metadata)
        if operation == "delete":
            return _do_delete(resource, schema_instance)
        print(json.dumps({"error": f"Unknown operation: {operation}"}), file=sys.stderr)
        return 2
    except Exception as exc:
        _logger.error("Operation '%s' on '%s' failed: %s", operation, resource_type, exc, exc_info=True)
        print(json.dumps({"error": str(exc)}), file=sys.stderr)
        return 1


def _resolve_class(resource_type: str, adapted_content: dict | None = None) -> type:
    if not resource_type:
        raise ValueError("--resource is required")

    # Primary path: use module + class from the adapted resource manifest's content
    # field, injected by DSC via adaptedContentArg.  This works for both pre-built
    # manifests (installed packages) and bundled resources (not pip-installed).
    if adapted_content:
        module_name = adapted_content.get("module")
        class_name = adapted_content.get("class")
        if module_name and class_name:
            try:
                mod = importlib.import_module(module_name)
                cls = getattr(mod, class_name)
                _logger.debug("Resolved %s via content: %s.%s", resource_type, module_name, class_name)
                return cls
            except Exception as exc:
                _logger.debug("Content-based resolution failed for %s (%s.%s): %s; falling back to entry points",
                              resource_type, module_name, class_name, exc)

    # Fallback path: entry_points (pip-installed resources without pre-built manifests,
    # or manual invocation outside of DSC where no --content is injected).
    type_lower = resource_type.casefold()
    try:
        eps = importlib.metadata.entry_points(group="microsoft.dsc.resources")
    except Exception as exc:
        raise RuntimeError(f"Failed to enumerate entry points: {exc}") from exc

    for ep in eps:
        if ep.name.casefold() == type_lower:
            _logger.debug("Resolved %s via entry point: %s", resource_type, getattr(ep, "value", "?"))
            return ep.load()

    raise ValueError(
        f"Resource type '{resource_type}' not found. "
        "Ensure the package is installed and registers a "
        "'microsoft.dsc.resources' entry point, or that the "
        "adapted resource manifest includes 'content.module' and 'content.class'."
    )


def _do_get(resource: Any, instance: Any) -> int:
    result = resource.get(instance)
    print(json.dumps(to_dict(result)))
    return 0


def _do_set(resource: Any, instance: Any, metadata: Any) -> int:
    result = resource.set(instance)
    print(json.dumps(to_dict(result.actual_state)))

    # Check SetReturn metadata to determine output format
    set_return = getattr(getattr(metadata, "set_return", None), "value", "state") if metadata is not None else "state"
    if set_return == "stateAndDiff":
        print(json.dumps(result.changed_properties or []))
    return 0


def _do_test(resource: Any, instance: Any, metadata: Any) -> int:
    result = resource.test(instance)
    print(json.dumps(to_dict(result.actual_state)))

    # Check TestReturn metadata to determine output format
    test_return = getattr(getattr(metadata, "test_return", None), "value", "state") if metadata is not None else "state"
    if test_return == "stateAndDiff":
        print(json.dumps(result.differing_properties or []))
    return 0


def _do_delete(resource: Any, instance: Any) -> int:
    resource.delete(instance)
    return 0


def _do_export(resource: Any, instance: Any | None) -> int:
    for item in resource.export(instance):
        print(json.dumps(to_dict(item)))
    return 0
