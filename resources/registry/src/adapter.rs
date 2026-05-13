// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::error::RegistryResourceError;
use dsc_lib_registry::RegistryHelper;
use serde::Deserialize;
use serde_json::{Map, Value};
use tracing::debug;

#[derive(Deserialize)]
struct AdaptedRegistryResource {
    #[serde(flatten)]
    properties: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdaptedRegistryValue {
    key_path: String,
    value_name: String,
    value_type: String,
    map_json_to_registry: Value,
}

pub fn adapter_get(input: &str, adapted_resource: &str) -> Result<String, RegistryResourceError> {
    debug!("Adapter Get with input: {input}");
    let adapted_resource: AdaptedRegistryResource = serde_json::from_str(adapted_resource)
        .map_err(|e| RegistryResourceError::AdaptedResourceDeserializationError(e.to_string()))?;
    let mut result = Map::new();

    for (key, value) in adapted_resource.properties.iter() {
        let adapted_registry_value: AdaptedRegistryValue = serde_json::from_value(value.clone())
            .map_err(|e| RegistryResourceError::AdaptedResourceDeserializationError(e.to_string()))?;
        let reg_helper = RegistryHelper::new()
    }
    Ok("{}".to_string())
}

pub fn adapter_set(input: &str, adapted_resource: &str) -> Result<String, RegistryResourceError> {
    debug!("Adapter Set with input: {input}");
    let adapted_resource: AdaptedRegistryResource = serde_json::from_str(adapted_resource)
        .map_err(|e| RegistryResourceError::AdaptedResourceDeserializationError(e.to_string()))?;
    
    for (key, value) in adapted_resource.properties.iter() {
        debug!("Property: {key} = {value}");
    }
    Ok("{}".to_string())
}
