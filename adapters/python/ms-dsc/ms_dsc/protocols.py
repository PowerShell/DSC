# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
DSC capability Protocols.

Resources declare capabilities by implementing the matching Protocol.
The adapter and dsc-gen CLI use isinstance() checks against these Protocols
to determine which operations to include in the resource manifest.

All Protocols are @runtime_checkable so isinstance() works without
requiring explicit inheritance.
"""
from __future__ import annotations

from collections.abc import Iterator
from typing import Generic, Protocol, TypeVar, runtime_checkable

from ms_dsc.results import SetResult, TestResult

T = TypeVar("T")


@runtime_checkable
class Gettable(Protocol[T]):
    """Resource can return its current state."""

    def get(self, instance: T) -> T: ...


@runtime_checkable
class Settable(Protocol[T]):
    """Resource can enforce a desired state."""

    def set(self, instance: T) -> SetResult[T]: ...


@runtime_checkable
class Testable(Protocol[T]):
    """Resource can compare actual state against desired state."""

    def test(self, instance: T) -> TestResult[T]: ...


@runtime_checkable
class Deletable(Protocol[T]):
    """Resource can remove the managed entity."""

    def delete(self, instance: T) -> None: ...


@runtime_checkable
class Exportable(Protocol[T]):
    """Resource can enumerate all instances of the managed entity."""

    def export(self, instance: T | None) -> Iterator[T]: ...
