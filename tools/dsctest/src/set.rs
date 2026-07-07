// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Set {
    pub value: Option<String>,
    pub object: Option<Map<String, Value>>,
}

impl Default for Set {
    fn default() -> Self {
        let mut object = Map::new();
        object.insert("property".to_string(), Value::String("Original".to_string()));
        Set {
            value: Some("Original".to_string()),
            object: Some(object)
        }
    }
}
pub fn invoke_set(get: bool, input: Option<String>) -> String {
    let set = if get {
        Set::default()
    } else {
        serde_json::from_str(&input.expect("Input is required")).expect("Failed to parse input JSON")
    };
    serde_json::to_string(&set).expect("Failed to serialize result")
}
