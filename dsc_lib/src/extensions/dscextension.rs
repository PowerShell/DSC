// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use schemars::JsonSchema;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DscExtension {
    /// The namespaced name of the resource.
    #[serde(rename="type")]
    pub type_name: String,
    /// The version of the resource.
    pub version: String,
    /// The capabilities of the resource.
    pub capabilities: Vec<Capability>,
    /// The file path to the resource.
    pub path: String,
    /// The description of the resource.
    pub description: Option<String>,
    // The directory path to the resource.
    pub directory: String,
    /// The author of the resource.
    pub author: Option<String>,
    /// The manifest of the resource.
    pub manifest: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Capability {
    /// The extension aids in discovering resources.
    Discover,
}

impl DscExtension {
    #[must_use]
    pub fn new() -> Self {
        Self {
            type_name: String::new(),
            version: String::new(),
            capabilities: Vec::new(),
            description: None,
            path: String::new(),
            directory: String::new(),
            author: None,
            manifest: None,
        }
    }
}

impl Default for DscExtension {
    fn default() -> Self {
        DscExtension::new()
    }
}