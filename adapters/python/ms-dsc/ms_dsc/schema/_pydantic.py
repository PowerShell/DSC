# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
PydanticSchemaProvider — JSON Schema generation via Pydantic v2.

Pydantic is an optional dependency.  Importing this provider raises a clear
ImportError with installation instructions when Pydantic is not installed.

Usage::

    from pydantic import BaseModel, Field
    from ms_dsc.schema import PydanticSchemaProvider

    class MySchema(BaseModel):
        name: str
        exist: bool = Field(default=True, alias="_exist")

    @dsc_resource(type="MyPackage/MyResource", version="1.0.0")
    class MyResource(DscResource[MySchema]):
        schema_provider = PydanticSchemaProvider(MySchema)
"""
from __future__ import annotations


class PydanticSchemaProvider:
    """Generates a JSON Schema dict from a Pydantic v2 model."""

    def __init__(self, model_type: type) -> None:
        self.schema_type = model_type
        self._model_type = model_type

    def get_schema(self) -> dict:
        try:
            import pydantic  # noqa: F401
        except ImportError as exc:
            raise ImportError(
                "pydantic is required for PydanticSchemaProvider. "
                "Install it with: pip install pydantic"
            ) from exc

        return self._model_type.model_json_schema()
