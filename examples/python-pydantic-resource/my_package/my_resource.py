# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Example DSC resource using Pydantic v2 for schema definition and validation.

This demonstrates how to use PydanticSchemaProvider instead of the default
DataclassSchemaProvider. Pydantic is useful when you need:
- Runtime validation of input properties
- Advanced type annotations (custom validators, constraints)
- Leverage existing Pydantic models

Compare with: ../python-dataclass-resource/
"""
from __future__ import annotations

import logging
from collections.abc import Iterator
from typing import Annotated

from pydantic import BaseModel, Field, field_validator
from ms_dsc import DscResource, SetResult, TestResult, dsc_resource
from ms_dsc.metadata import SetReturn, TestReturn
from ms_dsc.protocols import Deletable, Exportable, Gettable, Settable, Testable
from ms_dsc.schema import PydanticSchemaProvider

logger = logging.getLogger(__name__)


class MySchema(BaseModel):
    """
    Schema for MyPackage/MyResource using Pydantic.
    
    Note: Use Field(alias="_exist") to handle underscores in DSC property names.
    """
    name: str = Field(
        description="Unique name of the managed instance.",
        min_length=1,
        max_length=255,
    )
    _exist: bool = Field(
        default=True,
        description="Whether the instance should exist.",
    )

    @field_validator("name")
    @classmethod
    def validate_name(cls, v: str) -> str:
        """Custom validation: name cannot be 'reserved'."""
        if v.lower() == "reserved":
            raise ValueError("name cannot be 'reserved'")
        return v


@dsc_resource(
    type="MyPackage/MyResource",
    version="1.0.0",
    description="Example resource demonstrating Pydantic schema validation",
    set_return=SetReturn.STATE_AND_DIFF,
    test_return=TestReturn.STATE_AND_DIFF,
)
class MyResource(DscResource[MySchema], Gettable, Settable, Testable, Deletable, Exportable):
    schema_provider = PydanticSchemaProvider(MySchema)

    def get(self, instance: MySchema) -> MySchema:
        """Return the current state of the managed instance."""
        logger.info("Getting %s", instance.name)
        # Query actual system state here.
        return MySchema(name=instance.name, **{"_exist": True})

    def set(self, instance: MySchema) -> SetResult[MySchema]:
        """Apply the desired state."""
        logger.info("Setting %s to _exist=%s", instance.name, instance._exist)
        # Apply desired state here.
        return SetResult(
            actual_state=MySchema(name=instance.name, **{"_exist": instance._exist}),
            changed_properties=["_exist"],
        )

    def test(self, instance: MySchema) -> TestResult[MySchema]:
        """Compare actual vs desired state."""
        actual = self.get(instance)
        diffs = ["_exist"] if actual._exist != instance._exist else []
        return TestResult(actual_state=actual, differing_properties=diffs)

    def delete(self, instance: MySchema) -> None:
        """Remove the managed instance."""
        logger.info("Deleting %s", instance.name)
        pass  # Remove the managed instance here.

    def export(self, instance: MySchema | None) -> Iterator[MySchema]:
        """Enumerate all instances (used by dsc resource list)."""
        yield MySchema(name="example", **{"_exist": True})
