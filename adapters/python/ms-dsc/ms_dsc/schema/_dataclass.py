# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
DataclassSchemaProvider — JSON Schema generation from Python dataclasses.

Uses only the Python standard library (dataclasses, typing, inspect, types).
No external packages are required.

Field-level JSON Schema keywords can be supplied via dataclasses.field(metadata={...}):

    @dataclass
    class MySchema:
        name: str = field(metadata={"description": "Widget name", "title": "Name"})
        _exist: bool = field(default=True, metadata={"description": "Whether the widget exists"})

Supported metadata keys: description, title, examples, default (overrides the
dataclass default in the schema).

Type mapping:
    str              → {"type": "string"}
    int              → {"type": "integer"}
    float            → {"type": "number"}
    bool             → {"type": "boolean"}
    list[T]          → {"type": "array", "items": <T schema>}
    dict[str, V]     → {"type": "object", "additionalProperties": <V schema>}
    Optional[T]      → nullable variant of T
    T | None         → nullable variant of T  (Python 3.10+ union syntax)
    Literal[a, b]    → {"enum": [a, b]}
    Enum subclass    → {"enum": [member.value, ...]}
    nested @dataclass → inline object schema
"""
from __future__ import annotations

import dataclasses
import enum
import inspect
import types
import typing

from ms_dsc.schema._protocol import SchemaProvider


class DataclassSchemaProvider:
    """Generates a JSON Schema dict from a dataclass type."""

    def __init__(self, schema_type: type) -> None:
        if not dataclasses.is_dataclass(schema_type):
            raise TypeError(f"{schema_type!r} must be a @dataclass")
        self.schema_type = schema_type

    def get_schema(self) -> dict:
        return _dataclass_to_json_schema(self.schema_type)


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------

def _dataclass_to_json_schema(cls: type) -> dict:
    props: dict = {}
    required: list[str] = []

    try:
        hints = typing.get_type_hints(cls, include_extras=True)
    except Exception:
        hints = {f.name: f.type for f in dataclasses.fields(cls)}

    for f in dataclasses.fields(cls):
        field_schema = _type_to_schema(hints.get(f.name, type(None)))

        # Apply field-level JSON Schema keywords from metadata.
        for key in ("description", "title", "examples"):
            if key in f.metadata:
                field_schema[key] = f.metadata[key]

        props[f.name] = field_schema

        # A field is required only if it has no default.
        if (
            f.default is dataclasses.MISSING  # type: ignore[misc]
            and f.default_factory is dataclasses.MISSING  # type: ignore[misc]
        ):
            required.append(f.name)

    schema: dict = {
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": props,
        "additionalProperties": False,
    }
    if required:
        schema["required"] = required
    return schema


def _type_to_schema(t: type) -> dict:  # noqa: C901
    # Python 3.10+ union syntax: X | Y
    if isinstance(t, types.UnionType):
        return _union_schema(typing.get_args(t))

    origin = typing.get_origin(t)
    args = typing.get_args(t)

    # typing.Union / Optional
    if origin is typing.Union:
        return _union_schema(args)

    # list[T]
    if origin is list:
        schema: dict = {"type": "array"}
        if args:
            schema["items"] = _type_to_schema(args[0])
        return schema

    # dict[str, V]
    if origin is dict:
        schema = {"type": "object"}
        if len(args) >= 2:
            schema["additionalProperties"] = _type_to_schema(args[1])
        return schema

    # Literal["a", "b"]
    if origin is typing.Literal:
        return {"enum": list(args)}

    # Primitives
    _PRIM = {str: "string", int: "integer", float: "number", bool: "boolean"}
    if t in _PRIM:
        return {"type": _PRIM[t]}
    if t is type(None):
        return {"type": "null"}

    # Enum subclass
    if inspect.isclass(t) and issubclass(t, enum.Enum):
        return {"enum": [m.value for m in t]}

    # Nested dataclass
    if inspect.isclass(t) and dataclasses.is_dataclass(t):
        return _dataclass_to_json_schema(t)

    return {}


def _union_schema(args: tuple) -> dict:
    non_none = [a for a in args if a is not type(None)]
    has_none = len(non_none) < len(args)

    if len(non_none) == 1:
        schema = _type_to_schema(non_none[0])
        if has_none:
            t = schema.get("type")
            schema["type"] = [t, "null"] if isinstance(t, str) else "null"
        return schema

    return {"anyOf": [_type_to_schema(a) for a in args]}

