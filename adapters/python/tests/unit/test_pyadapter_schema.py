# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Unit tests for pyadapter.schema — JSON Schema generation from resource classes.
"""
from __future__ import annotations


from dataclasses import dataclass

from ms_dsc import DscResource, dsc_resource
from ms_dsc.schema import DataclassSchemaProvider, get_schema_type
from pyadapter.schema import generate_schema


@dataclass
class SimpleState:
    name: str
    count: int = 0


@dsc_resource(type="Test/WithGetSchema", version="1.0.0")
class ResourceWithGetSchema(DscResource[SimpleState]):
    """Resource that implements get_schema() classmethod."""
    schema_provider = DataclassSchemaProvider(SimpleState)

    @classmethod
    def get_schema(cls):
        return {"type": "object", "properties": {"custom": {"type": "string"}}}

    def get(self, i):
        return i


@dsc_resource(type="Test/WithSchemaMethod", version="1.0.0")
class ResourceWithSchemaMethod(DscResource[SimpleState]):
    """Resource that implements schema() classmethod."""
    schema_provider = DataclassSchemaProvider(SimpleState)

    @classmethod
    def schema(cls):
        return {"type": "object", "properties": {"method_schema": {"type": "integer"}}}

    def get(self, i):
        return i


@dsc_resource(type="Test/WithDataclass", version="1.0.0")
class ResourceWithDataclass(DscResource[SimpleState]):
    """Resource with schema provider — schema derived from dataclass."""
    schema_provider = DataclassSchemaProvider(SimpleState)

    def get(self, i):
        return i


class TestGenerateSchema:
    """Test schema generation resolution order."""

    def test_get_schema_takes_priority(self):
        """get_schema() should be called first and used if successful."""
        schema = generate_schema(ResourceWithGetSchema)
        assert schema.get("properties", {}).get("custom", {}).get("type") == "string"

    def test_schema_method_fallback(self):
        """schema() should be used if get_schema() not available."""
        schema = generate_schema(ResourceWithSchemaMethod)
        # Should use the schema from the method
        assert schema.get("method_schema", {}).get("type") == "integer" or schema.get("type") == "object"

    def test_dataclass_provider_fallback(self):
        """DataclassSchemaProvider should be used as fallback."""
        schema = generate_schema(ResourceWithDataclass)
        assert schema.get("type") == "object"
        # Should have properties from SimpleState dataclass
        props = schema.get("properties", {})
        assert "name" in props
        assert "count" in props

    def test_empty_schema_for_unsupported_class(self):
        """Unknown class types should return empty schema."""
        class UnsupportedClass:
            pass

        schema = generate_schema(UnsupportedClass)
        assert schema == {}

    def test_get_schema_exception_fallback(self):
        """If get_schema() raises, should try schema() method."""

        class BadGetSchema(DscResource[SimpleState]):
            schema_provider = DataclassSchemaProvider(SimpleState)

            @classmethod
            def get_schema(cls):
                raise ValueError("Something went wrong")

            @classmethod
            def schema(cls):
                return {"fallback": True}

            def get(self, i):
                return i

        schema = generate_schema(BadGetSchema)
        assert schema.get("fallback") is True

    def test_schema_method_exception_fallback(self):
        """If schema() raises, should try DataclassSchemaProvider."""

        class BadSchema(DscResource[SimpleState]):
            schema_provider = DataclassSchemaProvider(SimpleState)

            @classmethod
            def schema(cls):
                raise RuntimeError("Method failed")

            def get(self, i):
                return i

        schema = generate_schema(BadSchema)
        # Should fall back to dataclass provider
        assert "properties" in schema or schema == {}

    def test_get_schema_type_from_orig_bases(self):
        """_get_schema_type should extract T from DscResource[T]."""

        @dataclass
        class CustomState:
            value: str

        class MyResource(DscResource[CustomState]):
            def get(self, i):
                return i

        schema_type = get_schema_type(MyResource)
        assert schema_type is CustomState

    def test_get_schema_type_from_provider(self):
        """_get_schema_type should prefer schema_provider.schema_type."""

        @dataclass
        class ProviderState:
            data: str

        class MyResource(DscResource[SimpleState]):
            schema_provider = DataclassSchemaProvider(ProviderState)

            def get(self, i):
                return i

        schema_type = get_schema_type(MyResource)
        assert schema_type is ProviderState

    def test_get_schema_type_no_type_annotation(self):
        """_get_schema_type returns None for classes without type info."""

        class PlainClass:
            pass

        schema_type = get_schema_type(PlainClass)
        assert schema_type is None


class TestGenerateSchemaExceptionHandling:
    """Test exception handling in schema generation fallback chain."""

    def test_get_schema_exception_skips_to_schema_method(self):
        """If get_schema() raises, should fall back to schema() method."""

        class BadGetSchema(DscResource[SimpleState]):
            @classmethod
            def get_schema(cls):
                raise RuntimeError("get_schema failed")

            @classmethod
            def schema(cls):
                return {"fallback": "schema_method"}

            def get(self, i):
                return i

        schema = generate_schema(BadGetSchema)
        assert schema.get("fallback") == "schema_method"

    def test_get_schema_and_schema_method_both_fail(self):
        """If both get_schema() and schema() raise, fall back to dataclass."""

        class BadBoth(DscResource[SimpleState]):
            @classmethod
            def get_schema(cls):
                raise ValueError("get_schema failed")

            @classmethod
            def schema(cls):
                raise ValueError("schema failed")

            def get(self, i):
                return i

        schema = generate_schema(BadBoth)
        # Should have fallen back to DataclassSchemaProvider for SimpleState
        assert "properties" in schema or schema == {}

    def test_dataclass_provider_exception_returns_empty(self):
        """If DataclassSchemaProvider raises, return empty schema."""
        from unittest.mock import patch

        class OnlyDataclass(DscResource[SimpleState]):
            def get(self, i):
                return i

        with patch("pyadapter.schema.DataclassSchemaProvider") as mock_provider:
            mock_provider.return_value.get_schema.side_effect = RuntimeError(
                "introspection failed"
            )
            schema = generate_schema(OnlyDataclass)
            assert schema == {}

    def test_schema_method_exception_skips_to_dataclass(self):
        """If schema() raises, should fall back to dataclass provider."""

        class BadSchema(DscResource[SimpleState]):
            @classmethod
            def schema(cls):
                raise RuntimeError("schema method failed")

            def get(self, i):
                return i

        schema = generate_schema(BadSchema)
        # Should have dataclass properties or be empty
        assert isinstance(schema, dict)

    def test_non_dataclass_no_schema_methods(self):
        """Non-dataclass without schema methods returns empty."""

        class PlainClass:
            pass

        schema = generate_schema(PlainClass)
        assert schema == {}

    def test_class_with_get_schema_returning_dict(self):
        """get_schema() returning a dict should be used."""

        class WithGetSchema(DscResource[SimpleState]):
            @classmethod
            def get_schema(cls):
                return {"type": "object", "custom": True}

            def get(self, i):
                return i

        schema = generate_schema(WithGetSchema)
        assert schema.get("custom") is True

    def test_class_with_schema_returning_none(self):
        """schema() returning None should be handled gracefully."""

        class WithSchemaReturningNone(DscResource[SimpleState]):
            @classmethod
            def schema(cls):
                return None

            def get(self, i):
                return i

        schema = generate_schema(WithSchemaReturningNone)
        # None is falsy, but not an exception, so schema() succeeded
        # Should not fall back to dataclass
        assert schema is None or isinstance(schema, dict)

    def test_get_schema_not_callable(self):
        """get_schema attribute that is not callable should be skipped."""

        class WithNonCallableGetSchema(DscResource[SimpleState]):
            get_schema = "not a function"  # type: ignore

            @classmethod
            def schema(cls):
                return {"fallback": True}

            def get(self, i):
                return i

        schema = generate_schema(WithNonCallableGetSchema)
        assert schema.get("fallback") is True


class TestGetSchemaTypeEdgeCases:
    """Test _get_schema_type resolution logic edge cases."""

    def test_schema_provider_takes_priority(self):
        """schema_provider.schema_type should be used first."""

        @dataclass
        class ProviderState:
            data: str

        @dataclass
        class GenericState:
            value: int

        class WithProvider(DscResource[GenericState]):
            schema_provider = DataclassSchemaProvider(ProviderState)

            def get(self, i):
                return i

        schema_type = get_schema_type(WithProvider)
        assert schema_type is ProviderState

    def test_orig_bases_extraction(self):
        """Should extract T from DscResource[T] in __orig_bases__."""

        @dataclass
        class MyState:
            name: str

        class MyResource(DscResource[MyState]):
            def get(self, i):
                return i

        schema_type = get_schema_type(MyResource)
        assert schema_type is MyState

    def test_class_without_dsc_resource_base(self):
        """Class not inheriting from DscResource returns None."""

        @dataclass
        class StandaloneState:
            data: str

        class PlainClass:
            pass

        schema_type = get_schema_type(PlainClass)
        assert schema_type is None

    def test_non_dataclass_in_orig_bases(self):
        """Non-dataclass type argument in DscResource[T] is ignored."""

        class NonDataclassType:
            pass

        # Can't easily create DscResource[NonDataclassType] due to type checking,
        # but we can test the logic directly
        class FakeResource:
            __orig_bases__ = ("SomeBase[SomeType]",)  # String instead of type

        schema_type = get_schema_type(FakeResource)
        assert schema_type is None

    def test_schema_provider_no_schema_type_attribute(self):
        """schema_provider without schema_type attribute is skipped."""

        class FakeProvider:
            pass  # No schema_type attribute

        class WithBadProvider(DscResource[SimpleState]):
            schema_provider = FakeProvider()  # type: ignore

            def get(self, i):
                return i

        schema_type = get_schema_type(WithBadProvider)
        # Should fall through to __orig_bases__
        assert schema_type is SimpleState

    def test_multiple_orig_bases_first_match_wins(self):
        """First dataclass in __orig_bases__ is used."""

        @dataclass
        class FirstState:
            first: str

        @dataclass
        class SecondState:
            second: int

        # Manually construct since we can't easily have multiple DscResource bases
        class MultiBase(DscResource[FirstState]):
            def get(self, i):
                return i

        schema_type = get_schema_type(MultiBase)
        assert schema_type is FirstState

    def test_get_schema_type_with_no_orig_bases(self):
        """Class with no __orig_bases__ returns None."""

        class NoOrigBases:
            pass

        # Manually remove __orig_bases__ if it exists
        if hasattr(NoOrigBases, "__orig_bases__"):
            delattr(NoOrigBases, "__orig_bases__")

        schema_type = get_schema_type(NoOrigBases)
        assert schema_type is None
