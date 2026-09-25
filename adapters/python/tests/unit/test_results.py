# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""Unit tests for ms_dsc.results — SetResult and TestResult."""
from __future__ import annotations

from dataclasses import dataclass

from ms_dsc.results import SetResult, TestResult


@dataclass
class Schema:
    name: str
    _exist: bool = True


class TestSetResult:
    def test_actual_state_stored(self):
        state = Schema(name="foo")
        r = SetResult(actual_state=state)
        assert r.actual_state is state

    def test_changed_properties_defaults_to_none(self):
        r = SetResult(actual_state=Schema(name="foo"))
        assert r.changed_properties is None

    def test_changed_properties_list(self):
        r = SetResult(actual_state=Schema(name="foo"), changed_properties=["_exist"])
        assert r.changed_properties == ["_exist"]

    def test_changed_properties_empty_list(self):
        r = SetResult(actual_state=Schema(name="foo"), changed_properties=[])
        assert r.changed_properties == []

    def test_generic_type_preserved(self):
        state = Schema(name="widget")
        r: SetResult[Schema] = SetResult(actual_state=state)
        assert r.actual_state.name == "widget"


class TestTestResult:
    def test_actual_state_stored(self):
        state = Schema(name="bar")
        r = TestResult(actual_state=state)
        assert r.actual_state is state

    def test_differing_properties_defaults_to_none(self):
        r = TestResult(actual_state=Schema(name="bar"))
        assert r.differing_properties is None

    def test_differing_properties_list(self):
        r = TestResult(actual_state=Schema(name="bar"), differing_properties=["_exist"])
        assert r.differing_properties == ["_exist"]

    def test_in_desired_state_when_empty_list(self):
        r = TestResult(actual_state=Schema(name="bar"), differing_properties=[])
        assert r.differing_properties == []
        assert r.differing_properties is not None
        assert len(r.differing_properties) == 0

    def test_generic_type_preserved(self):
        state = Schema(name="item", _exist=False)
        r: TestResult[Schema] = TestResult(actual_state=state)
        assert r.actual_state._exist is False
