# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Unit tests for the dsc-example-resource demo package.

These tests verify resource logic without requiring the DSC CLI or the adapter
to be on PATH.  They import the resource classes directly.
"""
from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest

# Ensure the demo package is importable even if not pip-installed (test-only path setup).
_DEMO_ROOT = Path(__file__).parent.parent.parent / "examples" / "dsc-example-resource"
if str(_DEMO_ROOT) not in sys.path:
    sys.path.insert(0, str(_DEMO_ROOT))

from dsc_example_resource.resources import (
    CounterResource,
    CounterSchema,
    EnvVarResource,
    EnvVarSchema,
    GreetingResource,
    GreetingSchema,
)


# ===========================================================================
# Example/Greeting
# ===========================================================================

class TestGreetingResource:
    def setup_method(self):
        self.resource = GreetingResource()

    def test_get_returns_greeting(self):
        result = self.resource.get(GreetingSchema(name="Alice"))
        assert result.name == "Alice"
        assert "Alice" in result.message

    def test_get_name_in_message(self):
        result = self.resource.get(GreetingSchema(name="Bob"))
        assert "Bob" in result.message

    def test_get_returns_greeting_schema(self):
        result = self.resource.get(GreetingSchema(name="X"))
        assert isinstance(result, GreetingSchema)

    def test_message_not_empty(self):
        result = self.resource.get(GreetingSchema(name="Test"))
        assert result.message != ""


# ===========================================================================
# Example/Counter
# ===========================================================================

class TestCounterResource:
    def setup_method(self):
        self.resource = CounterResource()

    def _cleanup(self, counter_id: str) -> None:
        """Remove counter from the persistence file after each test."""
        try:
            self.resource.delete(CounterSchema(id=counter_id))
        except Exception:
            pass

    def test_get_returns_zero_when_absent(self, tmp_path, monkeypatch):
        """A counter that has never been set reports value=0."""
        monkeypatch.setattr(
            "dsc_example_resource.resources._COUNTER_FILE",
            tmp_path / "counters.json",
        )
        result = self.resource.get(CounterSchema(id="new-counter"))
        assert result.value == 0

    def test_set_persists_value(self, tmp_path, monkeypatch):
        import dsc_example_resource.resources as _mod
        monkeypatch.setattr(_mod, "_COUNTER_FILE", tmp_path / "counters.json")

        self.resource.set(CounterSchema(id="c1", value=7))
        result = self.resource.get(CounterSchema(id="c1"))
        assert result.value == 7

    def test_set_returns_changed_properties_when_value_changes(self, tmp_path, monkeypatch):
        import dsc_example_resource.resources as _mod
        monkeypatch.setattr(_mod, "_COUNTER_FILE", tmp_path / "counters.json")

        result = self.resource.set(CounterSchema(id="c2", value=10))
        assert "value" in result.changed_properties

    def test_set_returns_no_changed_properties_when_unchanged(self, tmp_path, monkeypatch):
        import dsc_example_resource.resources as _mod
        monkeypatch.setattr(_mod, "_COUNTER_FILE", tmp_path / "counters.json")

        self.resource.set(CounterSchema(id="c3", value=5))
        result = self.resource.set(CounterSchema(id="c3", value=5))
        assert result.changed_properties == []

    def test_test_in_desired_state(self, tmp_path, monkeypatch):
        import dsc_example_resource.resources as _mod
        monkeypatch.setattr(_mod, "_COUNTER_FILE", tmp_path / "counters.json")

        self.resource.set(CounterSchema(id="c4", value=3))
        result = self.resource.test(CounterSchema(id="c4", value=3))
        assert result.differing_properties == []

    def test_test_out_of_desired_state(self, tmp_path, monkeypatch):
        import dsc_example_resource.resources as _mod
        monkeypatch.setattr(_mod, "_COUNTER_FILE", tmp_path / "counters.json")

        self.resource.set(CounterSchema(id="c5", value=1))
        result = self.resource.test(CounterSchema(id="c5", value=99))
        assert "value" in result.differing_properties

    def test_delete_removes_counter(self, tmp_path, monkeypatch):
        import dsc_example_resource.resources as _mod
        monkeypatch.setattr(_mod, "_COUNTER_FILE", tmp_path / "counters.json")

        self.resource.set(CounterSchema(id="c6", value=42))
        self.resource.delete(CounterSchema(id="c6"))
        result = self.resource.get(CounterSchema(id="c6"))
        assert result.value == 0

    def test_export_yields_all_counters(self, tmp_path, monkeypatch):
        import dsc_example_resource.resources as _mod
        monkeypatch.setattr(_mod, "_COUNTER_FILE", tmp_path / "counters.json")

        self.resource.set(CounterSchema(id="a", value=1))
        self.resource.set(CounterSchema(id="b", value=2))
        items = list(self.resource.export(None))
        ids = {i.id for i in items}
        assert "a" in ids
        assert "b" in ids


# ===========================================================================
# Example/EnvVar
# ===========================================================================

class TestEnvVarResource:
    _VAR = "DSC_EXAMPLE_TEST_VAR_XYZ"

    def setup_method(self):
        self.resource = EnvVarResource()
        os.environ.pop(self._VAR, None)

    def teardown_method(self):
        os.environ.pop(self._VAR, None)

    def test_get_absent_returns_exist_false(self):
        result = self.resource.get(EnvVarSchema(name=self._VAR))
        assert result._exist is False
        assert result.value == ""

    def test_set_creates_variable(self):
        self.resource.set(EnvVarSchema(name=self._VAR, value="hello", _exist=True))
        result = self.resource.get(EnvVarSchema(name=self._VAR))
        assert result._exist is True
        assert result.value == "hello"

    def test_set_reports_exist_changed_when_created(self):
        result = self.resource.set(EnvVarSchema(name=self._VAR, value="x", _exist=True))
        assert "_exist" in result.changed_properties

    def test_set_removes_variable(self):
        os.environ[self._VAR] = "before"
        self.resource.set(EnvVarSchema(name=self._VAR, _exist=False))
        assert self._VAR not in os.environ

    def test_test_in_desired_state(self):
        os.environ[self._VAR] = "val"
        result = self.resource.test(EnvVarSchema(name=self._VAR, value="val", _exist=True))
        assert result.differing_properties == []

    def test_test_detects_value_drift(self):
        os.environ[self._VAR] = "actual"
        result = self.resource.test(EnvVarSchema(name=self._VAR, value="desired", _exist=True))
        assert "value" in result.differing_properties

    def test_test_detects_exist_drift(self):
        result = self.resource.test(EnvVarSchema(name=self._VAR, _exist=True))
        assert "_exist" in result.differing_properties

    def test_delete_removes_variable(self):
        os.environ[self._VAR] = "to-remove"
        self.resource.delete(EnvVarSchema(name=self._VAR))
        assert self._VAR not in os.environ

    def test_delete_is_idempotent(self):
        self.resource.delete(EnvVarSchema(name=self._VAR))  # does not raise

    def test_export_yields_matching_prefix(self):
        os.environ["DSC_EXAMPLE_TEST_VAR_XYZ"] = "1"
        os.environ["DSC_EXAMPLE_TEST_VAR_ABC"] = "2"
        try:
            items = list(self.resource.export(EnvVarSchema(name="DSC_EXAMPLE_TEST_VAR_")))
            names = {i.name for i in items}
            assert "DSC_EXAMPLE_TEST_VAR_XYZ" in names
            assert "DSC_EXAMPLE_TEST_VAR_ABC" in names
        finally:
            os.environ.pop("DSC_EXAMPLE_TEST_VAR_ABC", None)
