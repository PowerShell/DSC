# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Fixture DSC resources used by the Python adapter test suite.

Three resources cover the full spectrum of DSC capabilities:

  DscTest/Get        — read-only (get only)
  DscTest/ReadWrite  — full lifecycle (get/set/test/delete), STATE_AND_DIFF
  DscTest/Export     — enumeration only (export)
"""
from __future__ import annotations

import logging
from collections.abc import Iterator
from dataclasses import dataclass, field

from ms_dsc import DscResource, SetResult, TestResult, dsc_resource
from ms_dsc.metadata import SetReturn, TestReturn
from ms_dsc.schema import DataclassSchemaProvider

_logger = logging.getLogger(__name__)

# Simple in-process store backing DscTest/ReadWrite.
# Keyed by resource name; value is the _exist bool.
_STORE: dict[str, bool] = {}


@dataclass
class TestSchema:
    """Common schema shared by all fixture resources."""

    name: str = field(metadata={"description": "Unique name of the test instance."})
    _exist: bool = field(
        default=True,
        metadata={"description": "Whether the instance should exist."},
    )


# ---------------------------------------------------------------------------
# DscTest/Get — read-only resource
# ---------------------------------------------------------------------------

@dsc_resource(
    type="DscTest/Get",
    version="0.1.0",
    description="Read-only fixture resource.  Always reports _exist=True.",
    tags=["test"],
)
class GetResource(DscResource[TestSchema]):
    schema_provider = DataclassSchemaProvider(TestSchema)

    def get(self, instance: TestSchema) -> TestSchema:
        _logger.info("DscTest/Get.get(%s)", instance.name)
        return TestSchema(name=instance.name, _exist=True)


# ---------------------------------------------------------------------------
# DscTest/ReadWrite — full lifecycle resource
# ---------------------------------------------------------------------------

@dsc_resource(
    type="DscTest/ReadWrite",
    version="0.1.0",
    description="Read/write fixture resource backed by an in-process dict.",
    tags=["test"],
    set_return=SetReturn.STATE_AND_DIFF,
    test_return=TestReturn.STATE_AND_DIFF,
)
class ReadWriteResource(DscResource[TestSchema]):
    schema_provider = DataclassSchemaProvider(TestSchema)

    def get(self, instance: TestSchema) -> TestSchema:
        actual = _STORE.get(instance.name, False)
        _logger.info("DscTest/ReadWrite.get(%s) → _exist=%s", instance.name, actual)
        return TestSchema(name=instance.name, _exist=actual)

    def set(self, instance: TestSchema) -> SetResult[TestSchema]:
        before = _STORE.get(instance.name, False)
        _STORE[instance.name] = instance._exist
        changed = ["_exist"] if before != instance._exist else []
        _logger.info("DscTest/ReadWrite.set(%s, _exist=%s) changed=%s", instance.name, instance._exist, changed)
        return SetResult(
            actual_state=TestSchema(name=instance.name, _exist=instance._exist),
            changed_properties=changed,
        )

    def test(self, instance: TestSchema) -> TestResult[TestSchema]:
        actual = self.get(instance)
        diffs = ["_exist"] if actual._exist != instance._exist else []
        return TestResult(actual_state=actual, differing_properties=diffs)

    def delete(self, instance: TestSchema) -> None:
        _logger.info("DscTest/ReadWrite.delete(%s)", instance.name)
        _STORE.pop(instance.name, None)


# ---------------------------------------------------------------------------
# DscTest/Export — export-only resource
# ---------------------------------------------------------------------------

_EXPORT_ITEMS = [
    TestSchema(name="item-alpha", _exist=True),
    TestSchema(name="item-beta", _exist=True),
    TestSchema(name="item-gamma", _exist=False),
]


@dsc_resource(
    type="DscTest/Export",
    version="0.1.0",
    description="Export-only fixture resource returning a fixed list of items.",
    tags=["test"],
)
class ExportResource(DscResource[TestSchema]):
    schema_provider = DataclassSchemaProvider(TestSchema)

    def export(self, instance: TestSchema | None) -> Iterator[TestSchema]:
        _logger.info("DscTest/Export.export(filter=%s)", instance)
        if instance is not None and instance.name:
            # Filter by name when a filter instance is provided.
            for item in _EXPORT_ITEMS:
                if item.name == instance.name:
                    yield item
        else:
            yield from _EXPORT_ITEMS
