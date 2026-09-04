# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
ms-dsc — Python SDK for writing Microsoft DSC v3 resources.

Public API::

    from ms_dsc import DscResource, dsc_resource, SetResult, TestResult
    from ms_dsc import SetReturn, TestReturn
    from ms_dsc.schema import DataclassSchemaProvider, PydanticSchemaProvider
"""
from ms_dsc.metadata import DscResourceMetadata, SetReturn, TestReturn, dsc_resource
from ms_dsc.resource import DscResource
from ms_dsc.results import SetResult, TestResult

__all__ = [
    "DscResource",
    "DscResourceMetadata",
    "SetReturn",
    "TestReturn",
    "dsc_resource",
    "SetResult",
    "TestResult",
]
