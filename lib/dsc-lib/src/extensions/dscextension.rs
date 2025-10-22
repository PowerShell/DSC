// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib_jsonschema::transforms::idiomaticize_string_enum;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use schemars::JsonSchema;
use std::fmt::Display;

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
    /// The extensions supported for importing.
    #[serde(rename = "importFileExtensions")]
    pub import_file_extensions: Option<Vec<String>>,
    /// The file path to the resource.
    pub path: String,
    /// The description of the resource.
    pub description: Option<String>,
    // The directory path to the resource.
    pub directory: String,
    /// The author of the resource.
    pub author: Option<String>,
    /// The manifest of the resource.
    pub manifest: Value,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(transform = idiomaticize_string_enum)]
pub enum Capability {
    /// The extension aids in discovering resources.
    Discover,
    /// The extension aids in retrieving secrets.
    Secret,
    /// The extension imports configuration from a different format.
    Import,
}

impl Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::Discover => write!(f, "Discover"),
            Capability::Secret => write!(f, "Secret"),
            Capability::Import => write!(f, "Import"),
        }
    }
}

impl DscExtension {
    #[must_use]
    pub fn new() -> Self {
        Self {
            type_name: String::new(),
            version: String::new(),
            capabilities: Vec::new(),
            import_file_extensions: None,
            description: None,
            path: String::new(),
            directory: String::new(),
            author: None,
            manifest: Value::Null,
        }
    }
}

impl Default for DscExtension {
    fn default() -> Self {
        DscExtension::new()
    }
}
