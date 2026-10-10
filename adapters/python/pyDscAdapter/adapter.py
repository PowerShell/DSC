import sys
import json
import cProfile
import pstats
import time
import io
import os
import inspect
from contextlib import contextmanager
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple
# TODO: Currently using absolute imports. Switch to relative imports if this is later used as a module in the adapter manifest; otherwise, keep as-is for direct script execution.
from dsc_logging import setup_dsc_logging, operation_context
from discovery import get_class_map_from_pyproject, get_project_metadata_from_pyproject, get_resource_metadata_from_pyproject, import_class_from_file

#----------------------------------------------------------------------------
# ResourceAdapter - main adapter class with registry, profiling, and logging
#----------------------------------------------------------------------------
class ResourceAdapter:
    """
    Routes adapter operations to Python resource classes discovered from a
    pyproject.toml file near the provided resource path.

    The adapter also provides:
        - profile_block for optional timing and cProfile instrumentation
        - log(level, message, target, **kwargs) for structured adapter logging
        - direct resource-type to class resolution from pyproject manifest data
    """

    def __init__(self) -> None:
        # Normalize DSC trace level to standard Python logging levels
        # Supported inputs: trace, debug, info, warning, error, critical
        dsc_level = (os.getenv("DSC_TRACE_LEVEL", "info") or "info").strip().lower()

        self.logger = setup_dsc_logging(dsc_level)

        # Enable Profiling based on DSC trace level
        self.ENABLE_PROFILING = dsc_level in ("trace", "debug")

        self.logger.debug(f"Trace level: '{dsc_level}', profiling: {self.ENABLE_PROFILING}")
        self.logger.info("Adapter initialization complete")


    def _resolve_pyproject_path(self, resource_path: str = "") -> Optional[Path]:
        """Resolve the nearest pyproject.toml containing [tool.dsc.resources] starting from resource_path."""
        self.logger.debug(f"Resolving pyproject.toml for resource_path='{resource_path}'")
        candidates: List[Path] = []

        if resource_path:
            resolved_resource_path = Path(resource_path).resolve()
            candidates.append(resolved_resource_path.parent / "pyproject.toml")
            candidates.extend(parent / "pyproject.toml" for parent in resolved_resource_path.parents)

        seen_candidates = set()
        for candidate in candidates:
            candidate_key = str(candidate).casefold()
            if candidate_key in seen_candidates:
                continue
            seen_candidates.add(candidate_key)

            self.logger.debug(f"Checking pyproject candidate: '{candidate}'")
            if candidate.exists() and get_class_map_from_pyproject(candidate):
                self.logger.debug(f"Found pyproject.toml with DSC resources at '{candidate}'")
                return candidate.resolve()

        self.logger.warning(f"No pyproject.toml with [tool.dsc.resources] found for resource_path='{resource_path}'")
        return None

    @contextmanager
    def profile_block(self, label):
        """Context manager for optional profiling of code blocks."""
        if self.ENABLE_PROFILING:
            start_time = time.perf_counter()
            profiler = None
            try:
                profiler = cProfile.Profile()
                profiler.enable()
            except Exception:
                # Another profiler may already be active; fall back to timing only
                profiler = None
            try:
                yield
            finally:
                end_time = time.perf_counter()
                if profiler:
                    try:
                        profiler.disable()
                        s = io.StringIO()
                        ps = pstats.Stats(profiler, stream=s).sort_stats('cumulative')
                        ps.print_stats(10)
                        self.logger.info(f"[PROFILE] {label} took {end_time - start_time:.4f}s")
                        self.logger.debug(f"[PROFILE DETAILS] {label}:\n{s.getvalue()}")
                    except Exception:
                        # If profiling teardown fails, still log duration
                        self.logger.info(f"[PROFILE] {label} took {end_time - start_time:.4f}s")
                else:
                    self.logger.info(f"[PROFILE] {label} took {end_time - start_time:.4f}s")
        else:
            yield

    def log(self, level: str, message: str, target: str = None, **kwargs) -> None:
        """Structured logging method for adapter code."""
        lvl = level.lower()
        method = kwargs.get("method", "?")
        core_msg = f"{target} - {method} - {message}" if target else f"{method} - {message}"

        if lvl == "trace": # and hasattr(self.logger, "trace"):
            self.logger.debug(f"[TRACE] {core_msg}")
            return

        log_fn = getattr(self.logger, lvl, self.logger.info)
        log_fn(core_msg)


    def _load_manifest(self, resource_path: str = "") -> Dict[str, str]:
        """
        Resolve the nearest pyproject.toml for the supplied resource path and
        return the [tool.dsc.resources] class mapping.
        """
        if not resource_path:
            self.logger.debug("_load_manifest called with empty resource_path; returning empty class map")
            return {}

        self.logger.debug(f"Loading manifest class map for resource_path='{resource_path}'")

        pyproject_path = self._resolve_pyproject_path(resource_path)
        if not pyproject_path:
            self.logger.warning(f"No pyproject.toml found for '{resource_path}'; class map will be empty")
        class_map = get_class_map_from_pyproject(pyproject_path) if pyproject_path else {}
        self.logger.debug(f"Class map loaded: {class_map}")
        return class_map


    def _resolve_resource_class(self, resource_type: str, resource_path: str = "") -> type:
        """Resolve the resource class for a given resource type and path using the manifest mapping."""
        self.logger.debug(f"Resolving class for resource_type='{resource_type}', resource_path='{resource_path}'")
        if not resource_type.strip():
            raise ValueError("resource-type must be provided")

        class_map = self._load_manifest(resource_path)
        class_name = class_map.get(resource_type)
        if not class_name:
            lowered = {k.lower(): v for k, v in class_map.items()}
            class_name = lowered.get(resource_type.lower())
            if class_name:
                self.logger.debug(f"Exact lookup missed; using case-insensitive match for '{resource_type}'")

        if not class_name:
            supported = sorted(set(class_map.keys()))
            self.logger.error(f"No class mapping found for '{resource_type}'. Supported: {supported}")
            raise ValueError(f"Unsupported resource-type '{resource_type}'. Supported: {supported}")

        self.logger.debug(f"Class '{class_name}' found for '{resource_type}'; importing class")
        return import_class_from_file(resource_path, resource_type, class_name)


    def _instantiate_resource(self, cls: type, json_input: str, operation: Optional[str]) -> Any:
        """Instantiate a resource class from JSON input."""
        # Resource classes may expect operation-aware validation
        if hasattr(cls, "from_json"): 
            return cls.from_json(json_input, operation=operation)
        # Fallback: direct init from dict if needed
        data = json.loads(json_input or "{}")
        return cls(**data)

    def _build_list_entry(self, resource_type: str, class_name: str, resource_path: str, pyproject_path: Path, resource_metadata: Optional[Dict[str, str]] = None) -> Dict[str, Any]:
        """Build a list entry compatible with DSC adapter list output."""
        capabilities: List[str] = []
        properties: List[str] = []
        project_metadata = get_project_metadata_from_pyproject(pyproject_path)
        resource_metadata = resource_metadata or {}

        try:
            cls = import_class_from_file(resource_path, resource_type, class_name)
            for operation in ("get", "set", "test", "export"):
                if callable(getattr(cls, operation, None)):
                    capabilities.append(operation)

            init_fn = getattr(cls, "__init__", None)
            if callable(init_fn):
                signature = inspect.signature(init_fn)
                properties = [
                    name for name in signature.parameters
                    if name != "self" and not name.startswith("_")
                ]
        except Exception as err:
            # List should be best-effort; avoid failing the whole operation for one resource.
            self.logger.debug(
                "Unable to introspect resource '%s' at '%s': %s",
                resource_type,
                resource_path,
                err,
            )

        if not capabilities:
            capabilities = ["get", "set", "test"]  # remove this

        return {
            "type": resource_type,
            "kind": "resource",
            "version": resource_metadata.get("version") or project_metadata["version"],
            "capabilities": capabilities,
            "path": str(Path(resource_path).resolve()),
            "directory": str(pyproject_path.parent.resolve()),
            "implementedAs": "Python",
            "author": resource_metadata.get("author") or project_metadata["author"],
            "properties": properties,
            "requireAdapter": "Microsoft.DSC.Adapters/Python",
            "description": resource_metadata.get("description") or project_metadata["description"],
        }

    def _discover_python_resources(self, resource_path: str = "") -> List[Dict[str, Any]]:
        """Discover Python resources from pyproject.toml files and create list entries."""
        resources: List[Dict[str, Any]] = []
        pyproject_candidates: List[Path] = []

        # If a resource path is provided, scope discovery to the nearest pyproject.toml.
        if resource_path:
            pyproject_path = self._resolve_pyproject_path(resource_path)
            if pyproject_path:
                pyproject_candidates.append(pyproject_path)
        else:
            # For list without resource path, search from adapter root and include nested test/resource projects.
            adapter_root = Path(__file__).resolve().parents[1]
            pyproject_candidates = list(adapter_root.rglob("pyproject.toml"))

        seen = set()
        for pyproject_path in pyproject_candidates:
            key = str(pyproject_path.resolve()).casefold()
            if key in seen:
                continue
            seen.add(key)

            class_map = get_class_map_from_pyproject(pyproject_path)
            resource_metadata_map = get_resource_metadata_from_pyproject(pyproject_path)
            if not class_map:
                continue

            for discovered_type, class_name in class_map.items():
                resource_file = pyproject_path.parent / (discovered_type.split("/")[-1].lower() + ".py")
                if not resource_file.exists():
                    resource_file = pyproject_path.parent / (class_name.lower() + ".py")

                resources.append(
                    self._build_list_entry(
                        resource_type=discovered_type,
                        class_name=class_name,
                        resource_path=str(resource_file),
                        pyproject_path=pyproject_path,
                        resource_metadata=resource_metadata_map.get(discovered_type, {}),
                    )
                )

        resources.sort(key=lambda item: item.get("type", ""))
        return resources

    # -----------------
    # Operation routing
    # ----------------- 
    
    def run_operation(self, operation: str, json_input: str, resource_type: str, resource_path: str = "") -> Tuple[int, Dict[str, Any]]:
        """
        Execute a single adapter operation for one resource instance.

        Returns a tuple of (exit_code, result_dict). Most operations return a
        JSON-serializable result dictionary for the caller to print. The set
        and test operations are exceptions: they write their state and diff
        payloads directly to stdout and return a marker dictionary indicating
        that stdout has already been emitted.
        """
        op = (operation or "").strip().lower()
        self.logger.info(f"Operation '{op}' requested for resource_type='{resource_type}'")

        with operation_context(op, resource_type):
            if op == "list":
                self.logger.debug("List operation: discovering Python resources")
                resources = self._discover_python_resources(resource_path)
                for discovered_resource in resources:
                    sys.stdout.write(json.dumps(discovered_resource, ensure_ascii=False) + "\n")

                # Signal stdout already emitted so CLI does not wrap list output.
                return 0, {"_stdout_emitted": True}
            if op == "validate":
                self.logger.debug(f"Validate operation: returning valid=True for '{resource_type}'")
                #TODO: This will be removed later once adapted resource manifest schema is supported
                return 0, {"valid": True}

            # Resolve resource class
            try:
                self.logger.debug(f"Resolving resource_type='{resource_type}', resource_path='{resource_path}'")
                trimmed_resource_type = (resource_type or "").strip()
                resolved_type = trimmed_resource_type or os.getenv("DSC_RESOURCE_TYPE", "").strip()
                if not trimmed_resource_type and resolved_type:
                    self.logger.debug(f"resource_type resolved from env to '{resolved_type}'")
                cls = self._resolve_resource_class(resolved_type, resource_path)
                self.logger.debug(f"Resolved class '{cls.__name__}' for '{resolved_type}'")
            except Exception as e:
                self.log("error", str(e), "Adapter", operation=op)
                return 2, {"error": str(e)}

            try:
                if op == "get":
                    self.logger.info(f"Executing GET on '{resolved_type}'")
                    with self.profile_block("DSC Get Operation"):
                        instance = self._instantiate_resource(cls, json_input, operation="get")
                        data = instance.get()
                    self.logger.debug(f"GET returned: {data}")

                    return (0, data)

                elif op == "set":
                    self.logger.info(f"Executing SET on '{resolved_type}'")
                    with self.profile_block("DSC Set Operation"):
                        instance = self._instantiate_resource(cls, json_input, operation="set")
                        state, diffs = instance.set()
                    self.logger.debug(f"SET completed. diffs={diffs}")

                    sys.stdout.write(json.dumps(state, ensure_ascii=False) + "\n")
                    sys.stdout.write(json.dumps(diffs, ensure_ascii=False) + "\n")

                    # Signal to caller that we've already printed the required stdout
                    return (0, {"_stdout_emitted": True})

                elif op == "test":
                    self.logger.info(f"Executing TEST on '{resolved_type}'")
                    with self.profile_block("DSC Test Operation"):
                        instance = self._instantiate_resource(cls, json_input, operation="test")
                        actual_state, diffs = instance.test()
                    self.logger.debug(f"TEST completed. in_desired_state={len(diffs) == 0}, diffs={diffs}")

                    sys.stdout.write(json.dumps(actual_state if isinstance(actual_state, dict) else {}, ensure_ascii=False) + "\n")
                    sys.stdout.write(json.dumps(diffs if isinstance(diffs, list) else [], ensure_ascii=False) + "\n")

                    # Signal stdout already emitted so main() doesn't print a wrapper
                    return (0, {"_stdout_emitted": True})

                elif op == "export":
                    self.logger.info(f"Executing EXPORT on '{resolved_type}'")
                    # If your resource supports filtered export with provided input, pass instance; else pass None for full export
                    with self.profile_block("DSC Export Operation"):
                        # Parse export input strictly; unlike parse_json helper, invalid JSON must fail the operation.
                        raw_input = (json_input or "").strip()
                        as_obj = {} if not raw_input else json.loads(raw_input)
                        if not isinstance(as_obj, dict):
                            raise ValueError("Export input must be a JSON object")

                        # Any non-empty input object is treated as a filter payload.
                        has_filters = bool(as_obj)
                        self.logger.debug(f"Export has_filters={has_filters}")
                        instance = self._instantiate_resource(cls, json.dumps(as_obj), operation="export") if has_filters else None
                        data = cls.export(instance)
                        self.logger.debug("Export completed")
                        # If export returns None (prints only), still return an empty dict for adapter contract
                        return (0, data if isinstance(data, dict) else {})

                else:
                    msg = f"Unsupported operation '{operation}'. Expected one of: list, get, set, test, export, validate"
                    self.log("error", msg, "Adapter")
                    return 2, {"error": msg}

            except SystemExit as se:
                # Resource may call sys.exit() with an int, string, or arbitrary object.
                # Normalize without assuming SystemExit.code is numeric.
                raw_code = getattr(se, "code", 1)
                if isinstance(raw_code, int):
                    code = raw_code
                    error_message = f"Resource terminated with exit {code}"
                elif raw_code is None:
                    code = 1
                    error_message = "Resource terminated with exit 1"
                else:
                    code = 1
                    error_message = f"Resource terminated with exit {raw_code}"
                self.logger.error(f"Operation '{op}' on '{resolved_type}' terminated with sys.exit({raw_code})")
                return code, {"error": error_message}
            except Exception as err:
                self.logger.error(f"Operation '{op}' on '{resolved_type}' failed: {err}", exc_info=True)
                return 1, {"error": str(err)}
