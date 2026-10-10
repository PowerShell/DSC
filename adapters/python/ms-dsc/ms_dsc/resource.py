# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
DscResource — base class for Python DSC resources.

Resource classes inherit from DscResource[T] where T is the schema dataclass
(or Pydantic model) that describes the resource's properties.  Capabilities are
declared by implementing the matching Protocol from ms_dsc.protocols.

Example::

    from dataclasses import dataclass
    from ms_dsc import DscResource, dsc_resource, SetResult, TestResult
    from ms_dsc.schema import DataclassSchemaProvider

    @dataclass
    class MySchema:
        name: str
        _exist: bool = True

    @dsc_resource(type="MyPackage/MyResource", version="1.0.0")
    class MyResource(DscResource[MySchema]):
        schema_provider = DataclassSchemaProvider(MySchema)

        def get(self, instance: MySchema) -> MySchema:
            ...
"""
from __future__ import annotations

from typing import ClassVar, Generic, TypeVar

from ms_dsc.schema import SchemaProvider

T = TypeVar("T")


class DscResource(Generic[T]):
    """
    Base class for DSC resources.

    Subclasses must set ``schema_provider`` as a class variable.
    Capabilities are declared by implementing the Gettable, Settable,
    Testable, Deletable, and/or Exportable Protocols from ms_dsc.protocols.
    """

    schema_provider: ClassVar[SchemaProvider]

    @classmethod
    def get_schema(cls) -> dict:
        """Return a JSON Schema dict describing the resource's properties."""
        return cls.schema_provider.get_schema()
