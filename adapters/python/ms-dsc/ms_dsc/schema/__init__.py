# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
"""
Schema provider Protocol and public re-exports.

from ms_dsc.schema import (
    DataclassSchemaProvider,
    PydanticSchemaProvider,
    SchemaProvider,
    get_schema_type,
    build_schema_instance,
    to_dict,
)
"""
from ms_dsc.schema._dataclass import DataclassSchemaProvider
from ms_dsc.schema._pydantic import PydanticSchemaProvider
from ms_dsc.schema._protocol import SchemaProvider
from ms_dsc.schema._utils import build_schema_instance, get_schema_type, to_dict

__all__ = [
    "DataclassSchemaProvider",
    "PydanticSchemaProvider",
    "SchemaProvider",
    "get_schema_type",
    "build_schema_instance",
    "to_dict",
]
