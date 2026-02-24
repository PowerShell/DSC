// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::args::AdapterOperation;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdaptedOne {
    pub one: String,
    #[serde(rename = "_name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdaptedTwo {
    pub two: String,
    #[serde(rename = "_name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
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
    /// The directory path to the resource.
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

pub fn adapt(resource_type: &str, input: &str, operation: &AdapterOperation, resource_path: &Option<String>) -> Result<String, String> {
    match operation {
        AdapterOperation::List => {
            let resource_one = DscResource {
                type_name: "Adapted/One".to_string(),
                kind: "resource".to_string(),
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
                kind: "resource".to_string(),
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
        AdapterOperation::Get => {
            match resource_type {
                "Adapted/One" => {
                    let adapted_one = AdaptedOne {
                        one: "value1".to_string(),
                        name: None,
                        path: resource_path.clone(),
                    };
                    Ok(serde_json::to_string(&adapted_one).unwrap())
                },
                "Adapted/Two" => {
                    let adapted_two = AdaptedTwo {
                        two: "value2".to_string(),
                        name: None,
                        path: resource_path.clone(),
                    };
                    Ok(serde_json::to_string(&adapted_two).unwrap())
                },
                "Adapted/Three" => {
                    let adapted_three = AdaptedOne {
                        one: "value3".to_string(),
                        name: None,
                        path: resource_path.clone(),
                    };
                    Ok(serde_json::to_string(&adapted_three).unwrap())
                },
                "Adapted/Deprecated" => {
                    let adapted_deprecated = AdaptedOne {
                        one: "deprecated".to_string(),
                        name: None,
                        path: resource_path.clone(),
                    };
                    Ok(serde_json::to_string(&adapted_deprecated).unwrap())
                },
                _ => Err(format!("Unknown resource type: {resource_type}")),
            }
        },
        AdapterOperation::Set | AdapterOperation::Test => {
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
                "Adapted/Three" => {
                    let adapted_three: AdaptedOne = serde_json::from_str(input)
                        .map_err(|e| format!("Failed to parse input for Adapted/Three: {e}"))?;
                    Ok(serde_json::to_string(&adapted_three).unwrap())
                },
                _ => Err(format!("Unknown resource type: {resource_type}")),
            }
        },
        AdapterOperation::Export => {
            match resource_type {
                "Adapted/One" => {
                    let adapted_one = AdaptedOne {
                        one: "first1".to_string(),
                        name: Some("first".to_string()),
                        path: None,
                    };
                    println!("{}", serde_json::to_string(&adapted_one).unwrap());
                    let adapted_one = AdaptedOne {
                        one: "second1".to_string(),
                        name: Some("second".to_string()),
                        path: None,
                    };
                    println!("{}", serde_json::to_string(&adapted_one).unwrap());
                    std::process::exit(0);
                },
                "Adapted/Two" => {
                    let adapted_two = AdaptedTwo {
                        two: "first2".to_string(),
                        name: Some("first".to_string()),
                        path: None,
                    };
                    println!("{}", serde_json::to_string(&adapted_two).unwrap());
                    let adapted_two = AdaptedTwo {
                        two: "second2".to_string(),
                        name: Some("second".to_string()),
                        path: None,
                    };
                    println!("{}", serde_json::to_string(&adapted_two).unwrap());
                    std::process::exit(0);
                },
                "Adapted/Three" => {
                    let adapted_three = AdaptedOne {
                        one: "first3".to_string(),
                        name: Some("first".to_string()),
                        path: None,
                    };
                    println!("{}", serde_json::to_string(&adapted_three).unwrap());
                    let adapted_three = AdaptedOne {
                        one: "second3".to_string(),
                        name: Some("second".to_string()),
                        path: None,
                    };
                    println!("{}", serde_json::to_string(&adapted_three).unwrap());
                    std::process::exit(0);
                },
                _ => Err(format!("Unknown resource type: {resource_type}")),
            }
        },
        AdapterOperation::Validate => {
            Ok("{\"valid\": true}".to_string())
        },
    }
}
