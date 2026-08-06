# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
JSON Schema generation from Python dataclasses.

Delegates to ms_dsc.schema utilities for schema generation.
"""
from __future__ import annotations

from ms_dsc.schema import DataclassSchemaProvider, get_schema_type


def generate_schema(cls: type) -> dict:
    """
    Produce a JSON Schema dict for a resource class.

    Resolution order:
      1. cls.get_schema()  — SDK path (DscResource subclass)
      2. cls.schema()      — explicit classmethod
      3. Dataclass introspection via ms_dsc.schema.DataclassSchemaProvider
      4. Empty schema {}   — DSC will accept any input
    """
    if hasattr(cls, "get_schema") and callable(cls.get_schema):
        try:
            return cls.get_schema()
        except Exception:
            pass

    if hasattr(cls, "schema") and callable(cls.schema):
        try:
            return cls.schema()
        except Exception:
            pass

    # Try to use DataclassSchemaProvider from ms_dsc
    schema_type = get_schema_type(cls)
    if schema_type is not None:
        try:
            provider = DataclassSchemaProvider(schema_type)
            return provider.get_schema()
        except Exception:
            pass

    return {}
