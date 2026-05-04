// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::error::RegistryResourceError;
use serde::Deserialize;
use serde_json::{Map, Value};
use tracing::debug;

#[derive(Deserialize)]
struct AdaptedRegistryResource {
    #[serde(flatten)]
    properties: Map<String, Value>,
}

pub fn adapter_get(input: &str) -> Result<String, RegistryResourceError> {
    debug!("Adapter Get with input: {input}");
    let adapted_resource: AdaptedRegistryResource = serde_json::from_str(input)
        .map_err(|e| RegistryResourceError::AdapterInputParseError(e.to_string()))?;
    
    for (key, value) in adapted_resource.properties.iter() {
        debug!("Property: {key} = {value}");
    }
    Ok("{}".to_string())
}

pub fn adapter_set(input: &str) -> Result<String, RegistryResourceError> {
    debug!("Adapter Set with input: {input}");
    // adapter set is not implemented, return empty result for now
    Ok("{}".to_string())
}
