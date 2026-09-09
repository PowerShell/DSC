# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Utility functions for schema handling and serialization.

These utilities support common operations needed by adapter implementations
and resource code that work with typed schema instances.
"""
from __future__ import annotations

import dataclasses
import typing
from typing import Any, TypeVar

T = TypeVar("T")


def get_schema_type(cls: type) -> type | None:
    """
    Resolve T from DscResource[T] via schema_provider or __orig_bases__.

    Attempts to find the schema type (T) from a DscResource generic type
    by checking two paths:
    1. cls.schema_provider.schema_type attribute
    2. __orig_bases__ generic type arguments

    Parameters
    ----------
    cls : type
        A resource class or similar type with potential generic parameters.

    Returns
    -------
    type | None
        The resolved schema type, or None if not found.
    """
    provider = getattr(cls, "schema_provider", None)
    if provider is not None and hasattr(provider, "schema_type"):
        return provider.schema_type  # type: ignore[no-any-return]

    for base in getattr(cls, "__orig_bases__", ()):
        args = typing.get_args(base)
        if len(args) == 1 and dataclasses.is_dataclass(args[0]):
            return args[0]
    return None


def build_schema_instance(cls: type, data: dict) -> Any:
    """
    Construct a typed schema instance from JSON-compatible dictionary data.

    Supports three schema types:
    1. Dataclasses (via dataclasses.is_dataclass)
    2. Pydantic v2 models (with model_validate method)
    3. Any type accepting keyword arguments

    Filters dictionary keys for dataclasses to only include known fields.

    Parameters
    ----------
    cls : type
        The resource class with schema_provider or generic type annotation.
    data : dict
        JSON-compatible dictionary of properties.

    Returns
    -------
    Any
        A typed instance of the schema type, or the raw dict if schema type
        cannot be resolved.
    """
    schema_type = get_schema_type(cls)

    if schema_type is None:
        return data

    if dataclasses.is_dataclass(schema_type):
        known = {f.name for f in dataclasses.fields(schema_type)}
        filtered = {k: v for k, v in data.items() if k in known}
        return schema_type(**filtered)

    # Pydantic v2 BaseModel
    if hasattr(schema_type, "model_validate"):
        return schema_type.model_validate(data)

    try:
        return schema_type(**data)
    except Exception:
        return data


def to_dict(obj: Any) -> dict:
    """
    Convert a resource instance to a JSON-compatible dictionary.

    Supports conversion of:
    1. Dataclass instances
    2. Plain dicts
    3. Pydantic v2 models (with model_dump method)
    4. Other objects (raises TypeError)

    Parameters
    ----------
    obj : Any
        A resource instance or other object to serialize.

    Returns
    -------
    dict
        A JSON-compatible dictionary representation.

    Raises
    ------
    TypeError
        If the object type is not supported for serialization.
    """
    if dataclasses.is_dataclass(obj) and not isinstance(obj, type):
        return dataclasses.asdict(obj)
    if isinstance(obj, dict):
        return obj
    if hasattr(obj, "model_dump"):
        return obj.model_dump()
    raise TypeError(f"Cannot serialise {type(obj)!r} to dict")
