# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""SchemaProvider Protocol."""
from __future__ import annotations

from typing import Protocol, runtime_checkable


@runtime_checkable
class SchemaProvider(Protocol):
    """
    Structural protocol for schema providers.

    Any object with a ``get_schema()`` method returning a dict satisfies
    this protocol — no inheritance required.
    """

    schema_type: type

    def get_schema(self) -> dict: ...
