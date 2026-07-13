"""pythonunit.tests.py
Unit tests for pyDscAdapter internals.

**RUNS IN-PLACE**: This test file is NOT copied by the build system and runs directly
from the tests directory. It imports pyDscAdapter from the parent directory using:
    Path(__file__).parent.parent = adapters/python/
Which gives it access to: adapters/python/pyDscAdapter/

These tests exercise adapter functions, classes, and error paths directly
(complementing pythoncomponent.tests.ps1 which tests via subprocess invocation).
They focus on:
- Adapter initialization and configuration
- Manifest and resource class loading
- JSON parsing and error handling
- Logging and operation dispatch (mocking where necessary)
- ResourceAdapter lifecycle and manifest resolution

Use for: testing business logic and error edge cases.
These tests run fast and provide fine-grained failure diagnosis.

Run with: python adapters/python/tests/pythonunit.tests.py
          (from repo root: /DSC/)
"""

import unittest
from unittest.mock import Mock, patch
import json
import sys
from pathlib import Path
import io

# Ensure package-style adapter imports resolve the same way they do in normal use.
tests_dir = Path(__file__).resolve().parent
adapter_root = tests_dir.parent
adapter_root_str = str(adapter_root)
if adapter_root_str not in sys.path:
    sys.path.insert(0, adapter_root_str)
try:
    from pyDscAdapter import adapter as adapter_module
    from pyDscAdapter.adapter import ResourceAdapter
    from pyDscAdapter.utils import parse_json
except ImportError as e:
    raise ImportError(
         "Could not import adapter modules. "
         "Ensure adapters/python is on PYTHONPATH. "
         f"Error: {e}"
    ) from e

class TestResourceAdapterInit(unittest.TestCase):
    """Test ResourceAdapter initialization."""

    def test_adapter_initializes_with_defaults(self):
        """Adapter should initialize without errors."""
        with patch.dict("os.environ", {"DSC_TRACE_LEVEL": "info"}):
            adapter = ResourceAdapter()
            self.assertIsNotNone(adapter)
            self.assertIsNotNone(adapter.logger)
            self.assertFalse(adapter.ENABLE_PROFILING)

    def test_adapter_enables_profiling_on_debug_level(self):
        """Adapter should enable profiling when DSC_TRACE_LEVEL is debug."""
        with patch.dict("os.environ", {"DSC_TRACE_LEVEL": "debug"}):
            adapter = ResourceAdapter()
            self.assertTrue(adapter.ENABLE_PROFILING)

    def test_adapter_enables_profiling_on_trace_level(self):
        """Adapter should enable profiling when DSC_TRACE_LEVEL is trace."""
        with patch.dict("os.environ", {"DSC_TRACE_LEVEL": "trace"}):
            adapter = ResourceAdapter()
            self.assertTrue(adapter.ENABLE_PROFILING)

    def test_adapter_initializes_without_registry_state(self):
        """Adapter should not keep registry state on the instance."""
        with patch.dict("os.environ", {"DSC_TRACE_LEVEL": "info"}):
            adapter = ResourceAdapter()
            self.assertFalse(hasattr(adapter, "_registry"))


class TestParseJson(unittest.TestCase):
    """Test JSON parsing utility."""

    def test_parse_json_valid_object(self):
        """parse_json should parse valid JSON objects."""
        result = parse_json('{"key": "value"}')
        self.assertEqual(result, {"key": "value"})

    def test_parse_json_empty_string_returns_empty_dict(self):
        """parse_json should return empty dict for empty string."""
        result = parse_json("")
        self.assertEqual(result, {})

    def test_parse_json_null_returns_empty_dict(self):
        """parse_json should return empty dict for None input."""
        result = parse_json(None)
        self.assertEqual(result, {})

    def test_parse_json_invalid_returns_empty_dict(self):
        """parse_json should return empty dict for invalid JSON."""
        result = parse_json("not valid json")
        self.assertEqual(result, {})


class TestResourceAdapterOperationRouting(unittest.TestCase):
    """Test adapter operation dispatch."""

    def setUp(self):
        """Set up test fixtures."""
        with patch.dict("os.environ", {"DSC_TRACE_LEVEL": "info"}):
            self.adapter = ResourceAdapter()

    def test_list_operation_returns_empty_resources(self):
        """LIST operation should return empty resources array."""
        exit_code, result = self.adapter.run_operation("list", "{}", "", "")
        self.assertEqual(exit_code, 0)
        self.assertIn("resources", result)
        self.assertEqual(result["resources"], [])

    def test_validate_operation_returns_valid(self):
        """VALIDATE operation should return valid=True."""
        exit_code, result = self.adapter.run_operation("validate", "{}", "", "")
        self.assertEqual(exit_code, 0)
        self.assertEqual(result.get("valid"), True)

    def test_unsupported_operation_returns_error(self):
        """Unsupported operation should return exit code 2 with error."""
        # Must provide a resource_type to get to operation validation
        exit_code, result = self.adapter.run_operation("unknown_op", "{}", "PythonTest/Unknown", "")
        self.assertEqual(exit_code, 2)
        self.assertIn("error", result)
        # Will fail on resource-type resolution before getting to operation check
        self.assertIn("Unsupported", result["error"])

    def test_missing_resource_type_returns_error(self):
        """Missing resource_type for resource operations should fail."""
        exit_code, result = self.adapter.run_operation("get", "{}", "", "")
        self.assertNotEqual(exit_code, 0)
        self.assertIn("error", result)

    def test_unsupported_operation_with_valid_resource_returns_error(self):
        """Unsupported operation should return exit code 2 after successful class resolution."""

        class FakeResource:
            @classmethod
            def from_json(cls, _json_str, operation=None):
                return cls()

        with patch.object(self.adapter, "_resolve_resource_class", return_value=FakeResource):
            exit_code, result = self.adapter.run_operation("noop", "{}", "PythonTest/Get", "")

        self.assertEqual(exit_code, 2)
        self.assertIn("error", result)
        self.assertIn("Unsupported operation", result["error"])


class TestManifestResolution(unittest.TestCase):
    """Test manifest and resource class resolution."""

    def setUp(self):
        """Set up test fixtures."""
        with patch.dict("os.environ", {"DSC_TRACE_LEVEL": "info"}):
            self.adapter = ResourceAdapter()

    def test_resolve_pyproject_path_with_empty_resource_path(self):
        """_resolve_pyproject_path should return None for empty resource_path."""
        result = self.adapter._resolve_pyproject_path("")
        self.assertIsNone(result)

    def test_resolve_pyproject_path_with_nonexistent_path(self):
        """_resolve_pyproject_path should return None if no pyproject found."""
        result = self.adapter._resolve_pyproject_path("/nonexistent/path/resource.py")
        self.assertIsNone(result)

    def test_resolve_pyproject_path_finds_tests_pyproject(self):
        """_resolve_pyproject_path should find tests/pyproject.toml for known test resources."""
        known_resource = Path(__file__).parent / "src" / "get.py"
        result = self.adapter._resolve_pyproject_path(str(known_resource))
        self.assertIsNotNone(result)
        self.assertEqual(result.name, "pyproject.toml")

    def test_resolve_resource_class_without_resource_path_fails(self):
        """_resolve_resource_class should fail if no loader and no resource_path."""
        with self.assertRaises(ValueError):
            self.adapter._resolve_resource_class("UnknownType", "")

    def test_load_manifest_with_empty_resource_path_returns_empty_map(self):
        """_load_manifest should return empty map if resource_path is empty."""
        class_map = self.adapter._load_manifest("")
        self.assertEqual(class_map, {})

    def test_load_manifest_returns_known_resource_mapping(self):
        """_load_manifest should return class mappings from pyproject."""
        resource_path = Path(__file__).parent / "src" / "get.py"
        class_map = self.adapter._load_manifest(str(resource_path))

        self.assertIn("PythonTest/Get", class_map)
        self.assertEqual(class_map["PythonTest/Get"], "GetOnlyResource")

    def test_load_manifest_with_unknown_resource_type_not_present(self):
        """Unknown resource type should not exist in returned class map."""
        resource_path = Path(__file__).parent / "src" / "get.py"
        class_map = self.adapter._load_manifest(str(resource_path))
        self.assertNotIn("PythonTest/Unknown", class_map)

    def test_resolve_resource_class_case_insensitive_lookup(self):
        """_resolve_resource_class should resolve type names case-insensitively."""
        resource_path = Path(__file__).parent / "src" / "get.py"
        cls = self.adapter._resolve_resource_class("pythontest/get", str(resource_path))
        self.assertEqual(cls.__name__, "GetOnlyResource")

    def test_resolve_resource_class_uses_manifest_mapping(self):
        """_resolve_resource_class should resolve class directly from manifest mapping."""
        resource_path = Path(__file__).parent / "src" / "get.py"
        cls = self.adapter._resolve_resource_class("PythonTest/Get", str(resource_path))
        self.assertEqual(cls.__name__, "GetOnlyResource")


class TestLogging(unittest.TestCase):
    """Test logging functionality."""

    def setUp(self):
        """Set up test fixtures."""
        with patch.dict("os.environ", {"DSC_TRACE_LEVEL": "info"}):
            self.adapter = ResourceAdapter()

    def test_adapter_log_method_exists(self):
        """Adapter should have a log method."""
        self.assertTrue(hasattr(self.adapter, "log"))
        self.assertTrue(callable(self.adapter.log))

    def test_adapter_can_log_messages(self):
        """Adapter log method should not raise on valid input."""
        # This should not raise
        self.adapter.log("info", "test message", "test_target")

    def test_profile_block_context_manager_works(self):
        """profile_block should work as context manager."""
        with self.adapter.profile_block("test_operation"):
            pass  # Should complete without error

    def test_log_trace_uses_debug_path(self):
        """trace level should route through debug with [TRACE] marker."""
        with patch.object(self.adapter.logger, "debug") as mock_debug:
            self.adapter.log("trace", "hello", "Adapter", method="get")
            mock_debug.assert_called_once()
            self.assertIn("[TRACE]", mock_debug.call_args[0][0])

    def test_log_error_uses_error_logger(self):
        """error level should call logger.error."""
        with patch.object(self.adapter.logger, "error") as mock_error:
            self.adapter.log("error", "boom", "Adapter", method="set")
            mock_error.assert_called_once()
            self.assertIn("Adapter - set - boom", mock_error.call_args[0][0])

    def test_log_unknown_level_defaults_to_info(self):
        """Unknown levels should default to info logger."""
        with patch.object(self.adapter.logger, "info") as mock_info:
            self.adapter.log("nonsense", "fallback", None, method="test")
            mock_info.assert_called_once()
            self.assertIn("test - fallback", mock_info.call_args[0][0])


class TestProfilingBehavior(unittest.TestCase):
    """Test profile_block behavior in enabled/disabled and failure paths."""

    def test_profile_block_disabled_is_noop(self):
        with patch.dict("os.environ", {"DSC_TRACE_LEVEL": "info"}):
            adapter = ResourceAdapter()

        with patch.object(adapter.logger, "info") as mock_info:
            with adapter.profile_block("noop"):
                _ = 1 + 1
            mock_info.assert_not_called()

    def test_profile_block_enabled_logs_timing(self):
        with patch.dict("os.environ", {"DSC_TRACE_LEVEL": "debug"}):
            adapter = ResourceAdapter()

        with patch.object(adapter.logger, "info") as mock_info:
            with adapter.profile_block("timed"):
                _ = sum(range(10))
            self.assertTrue(mock_info.called)
            self.assertIn("[PROFILE] timed took", mock_info.call_args[0][0])

    def test_profile_block_profiler_creation_failure_still_logs_timing(self):
        with patch.dict("os.environ", {"DSC_TRACE_LEVEL": "debug"}):
            adapter = ResourceAdapter()

        with patch(f"{adapter_module.__name__}.cProfile.Profile", side_effect=Exception("busy")):
            with patch.object(adapter.logger, "info") as mock_info:
                with adapter.profile_block("fallback"):
                    _ = 2 + 2
                self.assertTrue(mock_info.called)
                self.assertIn("[PROFILE] fallback took", mock_info.call_args[0][0])

    def test_profile_block_teardown_failure_still_logs_timing(self):
        with patch.dict("os.environ", {"DSC_TRACE_LEVEL": "debug"}):
            adapter = ResourceAdapter()

        bad_profiler = Mock()
        bad_profiler.enable.return_value = None
        bad_profiler.disable.side_effect = RuntimeError("disable failed")

        with patch(f"{adapter_module.__name__}.cProfile.Profile", return_value=bad_profiler):
            with patch.object(adapter.logger, "info") as mock_info:
                with adapter.profile_block("teardown"):
                    _ = 3 + 3
                self.assertTrue(mock_info.called)
                self.assertIn("[PROFILE] teardown took", mock_info.call_args[0][0])


class TestEdgeCases(unittest.TestCase):
    """Test edge cases and error conditions."""

    def setUp(self):
        """Set up test fixtures."""
        with patch.dict("os.environ", {"DSC_TRACE_LEVEL": "info"}):
            self.adapter = ResourceAdapter()

    def test_instantiate_resource_with_none_input(self):
        """_instantiate_resource should handle None json_input."""
        mock_class = Mock()
        mock_class.from_json = Mock(return_value="instance")
        self.adapter._instantiate_resource(mock_class, "", "get")
        # Should use from_json when available
        mock_class.from_json.assert_called_once_with("", operation="get")

    def test_instantiate_resource_with_from_json_method(self):
        """_instantiate_resource should use from_json if available."""
        mock_class = Mock()
        mock_class.from_json = Mock(return_value="instance")
        self.adapter._instantiate_resource(mock_class, '{"key":"val"}', "get")
        mock_class.from_json.assert_called_once()

    def test_instantiate_resource_falls_back_to_direct_init(self):
        """_instantiate_resource should call class constructor when from_json is absent."""

        class DirectInitResource:
            def __init__(self, **kwargs):
                self.payload = kwargs

        instance = self.adapter._instantiate_resource(DirectInitResource, '{"name":"pkg"}', "get")
        self.assertEqual(instance.payload.get("name"), "pkg")

    def test_instantiate_resource_direct_init_invalid_json_raises(self):
        """Invalid JSON should raise when using direct-init fallback."""

        class DirectInitResource:
            def __init__(self, **kwargs):
                self.payload = kwargs

        with self.assertRaises(json.JSONDecodeError):
            self.adapter._instantiate_resource(DirectInitResource, "{bad-json", "get")

    def test_run_operation_uses_resource_type_from_environment(self):
        """run_operation should use DSC_RESOURCE_TYPE when resource_type argument is empty."""

        class FakeResource:
            @classmethod
            def from_json(cls, _json_str, operation=None):
                return cls()

            def get(self):
                return {"name": "pkg", "_exist": True}

        with patch.dict("os.environ", {"DSC_RESOURCE_TYPE": "PythonTest/Get"}):
            with patch.object(self.adapter, "_resolve_resource_class", return_value=FakeResource) as mock_resolve:
                exit_code, result = self.adapter.run_operation("get", "{}", "", "")

        self.assertEqual(exit_code, 0)
        self.assertIn("result", result)
        mock_resolve.assert_called_once_with("PythonTest/Get", "")

    def test_run_operation_get_wrapper_uses_resolved_type_and_name_fallbacks(self):
        """GET wrapper should use resolved env type for result type and fallback names."""

        class FakeResource:
            @classmethod
            def from_json(cls, _json_str, operation=None):
                return cls()

            def get(self):
                return {"_exist": True}

        with patch.dict("os.environ", {"DSC_RESOURCE_TYPE": "PythonTest/Get"}):
            with patch.object(self.adapter, "_resolve_resource_class", return_value=FakeResource) as mock_resolve:
                exit_code, result = self.adapter.run_operation("get", '{"name":""}', "", "")

        self.assertEqual(exit_code, 0)
        self.assertEqual(result["result"][0]["type"], "PythonTest/Get")
        self.assertEqual(result["name"], "PythonTest/Get")
        self.assertEqual(result["result"][0]["name"], "PythonTest/Get")
        mock_resolve.assert_called_once_with("PythonTest/Get", "")

    def test_run_operation_get_wrapper_uses_normalized_resolved_type(self):
        """GET wrapper should use normalized resolved type for wrapper fields."""

        class FakeResource:
            @classmethod
            def from_json(cls, _json_str, operation=None):
                return cls()

            def get(self):
                return {"_exist": True}

        with patch.dict("os.environ", {"DSC_RESOURCE_TYPE": "PythonTest/Other"}):
            with patch.object(self.adapter, "_resolve_resource_class", return_value=FakeResource) as mock_resolve:
                exit_code, result = self.adapter.run_operation("get", "{}", "  PythonTest/Get  ", "")

        self.assertEqual(exit_code, 0)
        self.assertEqual(result["result"][0]["type"], "PythonTest/Get")
        self.assertEqual(result["name"], "PythonTest/Get")
        self.assertEqual(result["result"][0]["name"], "PythonTest/Get")
        mock_resolve.assert_called_once_with("PythonTest/Get", "")

    def test_run_operation_get_handles_system_exit(self):
        """run_operation should normalize SystemExit raised by resource methods."""

        class SystemExitResource:
            @classmethod
            def from_json(cls, _json_str, operation=None):
                return cls()

            def get(self):
                raise SystemExit(7)

        with patch.object(self.adapter, "_resolve_resource_class", return_value=SystemExitResource):
            exit_code, result = self.adapter.run_operation("get", "{}", "PythonTest/Get", "")

        self.assertEqual(exit_code, 7)
        self.assertIn("error", result)
        self.assertIn("Resource terminated", result["error"])

    def test_run_operation_get_handles_exceptions(self):
        """run_operation should return exit code 1 for unexpected exceptions."""

        class ErrorResource:
            @classmethod
            def from_json(cls, _json_str, operation=None):
                return cls()

            def get(self):
                raise RuntimeError("boom")

        with patch.object(self.adapter, "_resolve_resource_class", return_value=ErrorResource):
            exit_code, result = self.adapter.run_operation("get", "{}", "PythonTest/Get", "")

        self.assertEqual(exit_code, 1)
        self.assertEqual(result.get("error"), "boom")

    def test_run_operation_set_marks_stdout_emitted(self):
        """SET operation should emit lines to stdout and return stdout-emitted marker."""

        class SetResource:
            @classmethod
            def from_json(cls, _json_str, operation=None):
                return cls()

            def set(self):
                return ({"name": "pkg", "_exist": True}, ["_exist"])

        with patch.object(self.adapter, "_resolve_resource_class", return_value=SetResource):
            fake_stdout = io.StringIO()
            with patch("sys.stdout", fake_stdout):
                exit_code, result = self.adapter.run_operation("set", "{}", "PythonTest/Set", "")

        self.assertEqual(exit_code, 0)
        self.assertTrue(result.get("_stdout_emitted"))
        output_lines = [line for line in fake_stdout.getvalue().splitlines() if line.strip()]
        self.assertEqual(len(output_lines), 2)

    def test_run_operation_test_marks_stdout_emitted(self):
        """TEST operation should emit lines to stdout and return stdout-emitted marker."""

        class TestResource:
            @classmethod
            def from_json(cls, _json_str, operation=None):
                return cls()

            def test(self):
                return ({"name": "pkg", "_exist": True}, [])

        with patch.object(self.adapter, "_resolve_resource_class", return_value=TestResource):
            fake_stdout = io.StringIO()
            with patch("sys.stdout", fake_stdout):
                exit_code, result = self.adapter.run_operation("test", "{}", "PythonTest/Test", "")

        self.assertEqual(exit_code, 0)
        self.assertTrue(result.get("_stdout_emitted"))
        output_lines = [line for line in fake_stdout.getvalue().splitlines() if line.strip()]
        self.assertEqual(len(output_lines), 2)

    def test_run_operation_export_without_filters_passes_none_instance(self):
        """EXPORT should call class export with None when no filter fields are provided."""
        sentinel = {"packages": []}

        class ExportResource:
            @staticmethod
            def export(instance=None):
                if instance is not None:
                    raise AssertionError("Instance should be None when no filters are provided")
                return sentinel

        with patch.object(self.adapter, "_resolve_resource_class", return_value=ExportResource):
            with patch.object(self.adapter, "_instantiate_resource") as mock_instantiate:
                exit_code, result = self.adapter.run_operation("export", "{}", "PythonTest/Export", "")

        self.assertEqual(exit_code, 0)
        self.assertEqual(result, sentinel)
        mock_instantiate.assert_not_called()

    def test_run_operation_export_with_filters_instantiates_resource(self):
        """EXPORT should instantiate resource when filter fields are present."""
        fake_instance = object()
        sentinel = {"packages": [{"name": "alpha"}]}

        class ExportResource:
            @staticmethod
            def export(instance=None):
                if instance is not fake_instance:
                    raise AssertionError("Expected instantiated resource instance")
                return sentinel

        with patch.object(self.adapter, "_resolve_resource_class", return_value=ExportResource):
            with patch.object(self.adapter, "_instantiate_resource", return_value=fake_instance) as mock_instantiate:
                exit_code, result = self.adapter.run_operation(
                    "export",
                    '{"name":"alpha"}',
                    "PythonTest/Export",
                    ""
                )

        self.assertEqual(exit_code, 0)
        self.assertEqual(result, sentinel)
        mock_instantiate.assert_called_once()

    def test_run_operation_export_with_exist_filter_instantiates_resource(self):
        """EXPORT should instantiate resource for non-empty object filters, including _exist."""
        fake_instance = object()
        sentinel = {"packages": [{"name": "alpha", "_exist": False}]}

        class ExportResource:
            @staticmethod
            def export(instance=None):
                if instance is not fake_instance:
                    raise AssertionError("Expected instantiated resource instance")
                return sentinel

        with patch.object(self.adapter, "_resolve_resource_class", return_value=ExportResource):
            with patch.object(self.adapter, "_instantiate_resource", return_value=fake_instance) as mock_instantiate:
                exit_code, result = self.adapter.run_operation(
                    "export",
                    '{"_exist":false}',
                    "PythonTest/Export",
                    ""
                )

        self.assertEqual(exit_code, 0)
        self.assertEqual(result, sentinel)
        mock_instantiate.assert_called_once()

    def test_run_operation_export_invalid_json_returns_error(self):
        """EXPORT should fail on invalid JSON input."""

        class ExportResource:
            @staticmethod
            def export(instance=None):
                return {"packages": []}

        with patch.object(self.adapter, "_resolve_resource_class", return_value=ExportResource):
            exit_code, result = self.adapter.run_operation(
                "export",
                "{bad-json",
                "PythonTest/Export",
                ""
            )

        self.assertEqual(exit_code, 1)
        self.assertIn("error", result)


if __name__ == "__main__":
    unittest.main()
