# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Resource metadata and the @dsc_resource decorator.

Usage::

    from ms_dsc import dsc_resource, SetReturn, TestReturn

    @dsc_resource(
        type="MyPackage/MyResource",
        version="1.0.0",
        description="Manages widgets",
        set_return=SetReturn.STATE_AND_DIFF,
        test_return=TestReturn.STATE_AND_DIFF,
    )
    class MyResource(DscResource[MySchema]):
        ...

The decorator attaches a DscResourceMetadata instance as __dsc_metadata__ on
the class.  The adapter and dsc-gen read this attribute at runtime to determine
manifest content and how to format stdout.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from typing import TypeVar

T = TypeVar("T")


class SetReturn(Enum):
    """
    Controls the return format of the set() operation.

    STATE
        The adapter emits one JSON line: the actual state after the operation.
        DSC will calculate changed properties itself if needed.

    STATE_AND_DIFF
        The adapter emits two JSON lines: the actual state, then a JSON array
        of changed property names.  The resource MUST populate
        SetResult.changed_properties with a (possibly empty) list.
    """

    STATE = "state"
    STATE_AND_DIFF = "stateAndDiff"


class TestReturn(Enum):
    """
    Controls the return format of the test() operation.

    STATE
        The adapter emits one JSON line: the actual (observed) state.
        DSC compares this against the desired state.

    STATE_AND_DIFF
        The adapter emits two JSON lines: the actual state, then a JSON array
        of differing property names.  The resource MUST populate
        TestResult.differing_properties with a (possibly empty) list.
    """

    STATE = "state"
    STATE_AND_DIFF = "stateAndDiff"


@dataclass
class DscResourceMetadata:
    """Metadata attached to a resource class by @dsc_resource."""

    type: str
    version: str
    description: str = ""
    tags: list[str] = field(default_factory=list)
    set_return: SetReturn = SetReturn.STATE
    test_return: TestReturn = TestReturn.STATE


def dsc_resource(
    type: str,  # noqa: A002
    version: str,
    *,
    description: str = "",
    tags: tuple[str, ...] | list[str] = (),
    set_return: SetReturn = SetReturn.STATE,
    test_return: TestReturn = TestReturn.STATE,
):
    """
    Class decorator that registers DSC resource metadata.

    Parameters
    ----------
    type:
        Fully-qualified DSC resource type, e.g. ``"MyPackage/MyResource"``.
    version:
        Semantic version string, e.g. ``"1.0.0"``.
    description:
        Human-readable description included in the resource manifest.
    tags:
        Optional list of string tags for discovery filtering.
    set_return:
        Whether set() returns state only (default) or state + changed properties.
    test_return:
        Whether test() returns state only (default) or state + differing properties.
    """

    def decorator(cls: type) -> type:
        cls.__dsc_metadata__ = DscResourceMetadata(
            type=type,
            version=version,
            description=description,
            tags=list(tags),
            set_return=set_return,
            test_return=test_return,
        )
        return cls

    return decorator
