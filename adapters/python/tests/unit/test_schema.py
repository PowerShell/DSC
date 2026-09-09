# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""Unit tests for ms_dsc.schema._dataclass — DataclassSchemaProvider."""
from __future__ import annotations

import pytest
from dataclasses import dataclass, field
from enum import Enum
from typing import Literal, Optional

from ms_dsc.schema._dataclass import DataclassSchemaProvider, _type_to_schema


class TestPrimitiveTypeMapping:
    def test_str(self):
        assert _type_to_schema(str) == {"type": "string"}

    def test_int(self):
        assert _type_to_schema(int) == {"type": "integer"}

    def test_float(self):
        assert _type_to_schema(float) == {"type": "number"}

    def test_bool(self):
        assert _type_to_schema(bool) == {"type": "boolean"}

    def test_none_type(self):
        assert _type_to_schema(type(None)) == {"type": "null"}

    def test_unknown_type_returns_empty(self):
        assert _type_to_schema(object) == {}


class TestCollectionTypeMapping:
    def test_list_str(self):
        assert _type_to_schema(list[str]) == {"type": "array", "items": {"type": "string"}}

    def test_list_int(self):
        schema = _type_to_schema(list[int])
        assert schema["type"] == "array"
        assert schema["items"] == {"type": "integer"}

    def test_list_no_arg(self):
        # bare list (no type arg) — just an array without items constraint
        schema = _type_to_schema(list)
        assert schema == {} or schema.get("type") == "array"

    def test_dict_str_str(self):
        schema = _type_to_schema(dict[str, str])
        assert schema["type"] == "object"
        assert schema["additionalProperties"] == {"type": "string"}

    def test_dict_str_int(self):
        schema = _type_to_schema(dict[str, int])
        assert schema["additionalProperties"] == {"type": "integer"}


class TestNullableTypeMapping:
    def test_optional_str(self):
        schema = _type_to_schema(Optional[str])
        assert "string" in schema.get("type", []) or schema.get("type") in (["string", "null"], ["null", "string"])
        # At minimum, the result should allow null
        t = schema.get("type")
        assert t is None or "null" in (t if isinstance(t, list) else [t])

    def test_union_syntax_str_none(self):
        schema = _type_to_schema(str | None)
        t = schema.get("type")
        assert isinstance(t, list)
        assert "string" in t
        assert "null" in t

    def test_union_syntax_int_none(self):
        schema = _type_to_schema(int | None)
        t = schema.get("type")
        assert "integer" in t and "null" in t

    def test_multi_union_uses_any_of(self):
        schema = _type_to_schema(str | int)
        assert "anyOf" in schema
        types = [s.get("type") for s in schema["anyOf"]]
        assert "string" in types
        assert "integer" in types


class TestLiteralTypeMapping:
    def test_string_literal(self):
        assert _type_to_schema(Literal["a", "b"]) == {"enum": ["a", "b"]}

    def test_int_literal(self):
        assert _type_to_schema(Literal[1, 2, 3]) == {"enum": [1, 2, 3]}

    def test_single_literal(self):
        assert _type_to_schema(Literal["only"]) == {"enum": ["only"]}


class TestEnumTypeMapping:
    def test_string_enum(self):
        class Color(Enum):
            RED = "red"
            BLUE = "blue"

        schema = _type_to_schema(Color)
        assert schema == {"enum": ["red", "blue"]}

    def test_int_enum(self):
        class Status(Enum):
            ACTIVE = 1
            INACTIVE = 0

        schema = _type_to_schema(Status)
        assert set(schema["enum"]) == {1, 0}


class TestNestedDataclassMapping:
    def test_nested_dataclass_inline(self):
        @dataclass
        class Inner:
            x: int
            y: str = "default"

        schema = _type_to_schema(Inner)
        assert schema["type"] == "object"
        assert "x" in schema["properties"]
        assert "y" in schema["properties"]
        assert schema["properties"]["x"] == {"type": "integer"}

    def test_nested_required_fields(self):
        @dataclass
        class Inner:
            required_field: str
            optional_field: int = 0

        schema = _type_to_schema(Inner)
        assert "required_field" in schema.get("required", [])
        assert "optional_field" not in schema.get("required", [])


class TestDataclassSchemaProvider:
    def test_rejects_non_dataclass(self):
        with pytest.raises(TypeError):
            DataclassSchemaProvider(str)

    def test_basic_schema_structure(self):
        @dataclass
        class S:
            name: str
            _exist: bool = True

        p = DataclassSchemaProvider(S)
        schema = p.get_schema()
        assert schema["type"] == "object"
        assert schema["additionalProperties"] is False
        assert "$schema" in schema

    def test_required_and_optional_fields(self):
        @dataclass
        class S:
            required: str
            optional: int = 0

        schema = DataclassSchemaProvider(S).get_schema()
        assert "required" in schema["required"]
        assert "optional" not in schema.get("required", [])

    def test_field_description_from_metadata(self):
        @dataclass
        class S:
            name: str = field(metadata={"description": "The name"})

        schema = DataclassSchemaProvider(S).get_schema()
        assert schema["properties"]["name"]["description"] == "The name"

    def test_field_title_from_metadata(self):
        @dataclass
        class S:
            name: str = field(metadata={"title": "Name"})

        schema = DataclassSchemaProvider(S).get_schema()
        assert schema["properties"]["name"]["title"] == "Name"

    def test_underscore_field_name_preserved(self):
        """DSC meta-properties like _exist must keep their leading underscore."""
        @dataclass
        class S:
            name: str
            _exist: bool = True

        schema = DataclassSchemaProvider(S).get_schema()
        assert "_exist" in schema["properties"]
        assert "exist" not in schema["properties"]

    def test_schema_type_attribute(self):
        @dataclass
        class S:
            name: str

        p = DataclassSchemaProvider(S)
        assert p.schema_type is S

    def test_nullable_field(self):
        @dataclass
        class S:
            name: str | None = None

        schema = DataclassSchemaProvider(S).get_schema()
        t = schema["properties"]["name"].get("type")
        assert t is not None
        assert "null" in (t if isinstance(t, list) else [t])
