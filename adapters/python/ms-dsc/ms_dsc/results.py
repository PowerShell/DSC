# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Result types returned by set() and test() resource methods.

Both types carry an actual_state field holding the resource's post-operation
(or currently observed) state, plus an optional list of property names that
differ.  Whether the diff list is emitted on stdout is controlled by the
SetReturn / TestReturn enum values on the @dsc_resource decorator.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Generic, TypeVar

T = TypeVar("T")


@dataclass
class SetResult(Generic[T]):
    """
    Result of a set() operation.

    actual_state
        The resource state after the set operation was applied.

    changed_properties
        Names of properties that were changed, or None when set_return=STATE
        (DSC will not receive a diff list, so the value is irrelevant).
        Required to be a list (may be empty) when set_return=STATE_AND_DIFF.
    """

    actual_state: T
    changed_properties: list[str] | None = field(default=None)


@dataclass
class TestResult(Generic[T]):
    """
    Result of a test() operation.

    actual_state
        The currently observed resource state (not the desired state).

    differing_properties
        Names of properties whose actual value differs from the desired value,
        or None when test_return=STATE.
        Required to be a list (may be empty) when test_return=STATE_AND_DIFF.
    """

    actual_state: T
    differing_properties: list[str] | None = field(default=None)
