// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{collections::{BTreeMap, HashMap}, sync::RwLock, sync::LazyLock};
use serde_json::Value;

use crate::types::{FullyQualifiedTypeName, ResourceVersion};

pub(crate) type SchemaCache = BTreeMap<FullyQualifiedTypeName, HashMap<ResourceVersion, Value>>;

pub(crate) static RESOURCE_SCHEMAS: LazyLock<RwLock<SchemaCache>> = LazyLock::new(|| RwLock::new(SchemaCache::new()));

pub(crate) fn get_resource_schema(type_name: &FullyQualifiedTypeName, version: &ResourceVersion) -> Option<Value> {
    let cache = RESOURCE_SCHEMAS.read().unwrap();
    if let Some(schemas) = cache.get(type_name) && let Some(schema) = schemas.get(version) {
        return Some(schema.clone());
    }
    None
}
