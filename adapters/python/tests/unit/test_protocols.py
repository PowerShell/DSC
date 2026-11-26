# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""Unit tests for ms_dsc.protocols — runtime-checkable Protocol isinstance checks."""
from __future__ import annotations

from collections.abc import Iterator
from dataclasses import dataclass

import pytest

from ms_dsc.protocols import Deletable, Exportable, Gettable, Settable, Testable
from ms_dsc.results import SetResult, TestResult


@dataclass
class Schema:
    name: str


class _Full:
    """Resource implementing every capability."""

    def get(self, instance: Schema) -> Schema:
        return instance

    def set(self, instance: Schema) -> SetResult[Schema]:
        return SetResult(actual_state=instance, changed_properties=[])

    def test(self, instance: Schema) -> TestResult[Schema]:
        return TestResult(actual_state=instance, differing_properties=[])

    def delete(self, instance: Schema) -> None:
        pass

    def export(self, instance: Schema | None) -> Iterator[Schema]:
        return iter([])


class TestGettable:
    def test_positive(self):
        assert isinstance(_Full(), Gettable)

    def test_negative(self):
        class X:
            pass

        assert not isinstance(X(), Gettable)

    def test_wrong_method_name(self):
        class X:
            def fetch(self, instance):
                return instance

        assert not isinstance(X(), Gettable)


class TestSettable:
    def test_positive(self):
        assert isinstance(_Full(), Settable)

    def test_negative(self):
        class X:
            pass

        assert not isinstance(X(), Settable)


class TestTestable:
    def test_positive(self):
        assert isinstance(_Full(), Testable)

    def test_negative(self):
        class X:
            pass

        assert not isinstance(X(), Testable)


class TestDeletable:
    def test_positive(self):
        assert isinstance(_Full(), Deletable)

    def test_negative(self):
        class X:
            pass

        assert not isinstance(X(), Deletable)


class TestExportable:
    def test_positive(self):
        assert isinstance(_Full(), Exportable)

    def test_negative(self):
        class X:
            pass

        assert not isinstance(X(), Exportable)


class TestPartialCapabilities:
    def test_get_only_is_not_settable(self):
        class GetOnly:
            def get(self, instance):
                return instance

        r = GetOnly()
        assert isinstance(r, Gettable)
        assert not isinstance(r, Settable)
        assert not isinstance(r, Testable)
        assert not isinstance(r, Deletable)
        assert not isinstance(r, Exportable)

    def test_full_resource_satisfies_all_protocols(self):
        r = _Full()
        for P in (Gettable, Settable, Testable, Deletable, Exportable):
            assert isinstance(r, P), f"Expected {r!r} to satisfy {P.__name__}"
