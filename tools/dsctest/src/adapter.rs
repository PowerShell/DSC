// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::args::AdapterOperation;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdaptedOne {
    pub one: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdaptedTwo {
    pub two: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DscResource {
    /// The namespaced name of the resource.
    #[serde(rename="type")]
    pub type_name: String,
    /// The kind of resource.
    pub kind: String,
    /// The version of the resource.
    pub version: String,
    /// The capabilities of the resource.
    pub capabilities: Vec<String>,
    /// The file path to the resource.
    pub path: String,
    // The directory path to the resource.
    pub directory: String,
    /// The implementation of the resource.
    #[serde(rename="implementedAs")]
    pub implemented_as: String,
    /// The properties of the resource.
    pub properties: Vec<String>,
    /// The required resource adapter for the resource.
    #[serde(rename="requireAdapter")]
    pub require_adapter: Option<String>,
}

pub fn adapt(resource_type: &str, input: &str, operation: &AdapterOperation) -> Result<String, String> {
    match operation {
        AdapterOperation::List => {
            let resource_one = DscResource {
                type_name: "Adapted/One".to_string(),
                kind: "DscResource".to_string(),
                version: "1.0.0".to_string(),
                capabilities: vec!["get".to_string(), "set".to_string(), "test".to_string(), "export".to_string()],
                path: "path/to/adapted/one".to_string(),
                directory: "path/to/adapted".to_string(),
                implemented_as: "TestAdapted".to_string(),
                properties: vec!["one".to_string()],
                require_adapter: Some("Test/Adapter".to_string()),
            };
            let resource_two = DscResource {
                type_name: "Adapted/Two".to_string(),
                kind: "DscResource".to_string(),
                version: "1.0.0".to_string(),
                capabilities: vec!["get".to_string(), "set".to_string(), "test".to_string(), "export".to_string()],
                path: "path/to/adapted/two".to_string(),
                directory: "path/to/adapted".to_string(),
                implemented_as: "TestAdapted".to_string(),
                properties: vec!["two".to_string()],
                require_adapter: Some("Test/Adapter".to_string()),
            };
            println!("{}", serde_json::to_string(&resource_one).unwrap());
            println!("{}", serde_json::to_string(&resource_two).unwrap());
            std::process::exit(0);
        },
        AdapterOperation::Get | AdapterOperation::Set | AdapterOperation::Test | AdapterOperation::Export => {
            match resource_type {
                "Adapted/One" => {
                    let adapted_one: AdaptedOne = serde_json::from_str(input)
                        .map_err(|e| format!("Failed to parse input for Adapted/One: {e}"))?;
                    Ok(serde_json::to_string(&adapted_one).unwrap())
                },
                "Adapted/Two" => {
                    let adapted_two: AdaptedTwo = serde_json::from_str(input)
                        .map_err(|e| format!("Failed to parse input for Adapted/Two: {e}"))?;
                    Ok(serde_json::to_string(&adapted_two).unwrap())
                },
                _ => Err(format!("Unknown resource type: {resource_type}")),
            }
        },
    }
}
