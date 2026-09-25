# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""Unit tests for ms_dsc.metadata — @dsc_resource decorator and enums."""
from __future__ import annotations

import pytest

from ms_dsc.metadata import DscResourceMetadata, SetReturn, TestReturn, dsc_resource


class TestSetReturnEnum:
    def test_state_value(self):
        assert SetReturn.STATE.value == "state"

    def test_state_and_diff_value(self):
        assert SetReturn.STATE_AND_DIFF.value == "stateAndDiff"

    def test_members(self):
        assert set(SetReturn) == {SetReturn.STATE, SetReturn.STATE_AND_DIFF}


class TestTestReturnEnum:
    def test_state_value(self):
        assert TestReturn.STATE.value == "state"

    def test_state_and_diff_value(self):
        assert TestReturn.STATE_AND_DIFF.value == "stateAndDiff"


class TestDscResourceDecorator:
    def test_attaches_metadata(self):
        @dsc_resource(type="Test/Foo", version="1.0.0")
        class R:
            pass

        assert hasattr(R, "__dsc_metadata__")
        assert isinstance(R.__dsc_metadata__, DscResourceMetadata)

    def test_type_and_version(self):
        @dsc_resource(type="My/Resource", version="2.3.4")
        class R:
            pass

        meta = R.__dsc_metadata__
        assert meta.type == "My/Resource"
        assert meta.version == "2.3.4"

    def test_default_values(self):
        @dsc_resource(type="T/R", version="0.0.1")
        class R:
            pass

        meta = R.__dsc_metadata__
        assert meta.description == ""
        assert meta.tags == []
        assert meta.set_return is SetReturn.STATE
        assert meta.test_return is TestReturn.STATE

    def test_description(self):
        @dsc_resource(type="T/R", version="1.0.0", description="A test resource")
        class R:
            pass

        assert R.__dsc_metadata__.description == "A test resource"

    def test_tags_list(self):
        @dsc_resource(type="T/R", version="1.0.0", tags=["windows", "test"])
        class R:
            pass

        assert R.__dsc_metadata__.tags == ["windows", "test"]

    def test_tags_tuple_converted_to_list(self):
        @dsc_resource(type="T/R", version="1.0.0", tags=("a", "b"))
        class R:
            pass

        assert R.__dsc_metadata__.tags == ["a", "b"]

    def test_set_return_state_and_diff(self):
        @dsc_resource(type="T/R", version="1.0.0", set_return=SetReturn.STATE_AND_DIFF)
        class R:
            pass

        assert R.__dsc_metadata__.set_return is SetReturn.STATE_AND_DIFF

    def test_test_return_state_and_diff(self):
        @dsc_resource(type="T/R", version="1.0.0", test_return=TestReturn.STATE_AND_DIFF)
        class R:
            pass

        assert R.__dsc_metadata__.test_return is TestReturn.STATE_AND_DIFF

    def test_decorator_returns_class_unchanged(self):
        @dsc_resource(type="T/R", version="1.0.0")
        class MyClass:
            x = 42

        assert MyClass.x == 42
        assert MyClass().__class__.__name__ == "MyClass"

    def test_multiple_classes_independent_metadata(self):
        @dsc_resource(type="T/A", version="1.0.0")
        class A:
            pass

        @dsc_resource(type="T/B", version="2.0.0")
        class B:
            pass

        assert A.__dsc_metadata__.type == "T/A"
        assert B.__dsc_metadata__.type == "T/B"
