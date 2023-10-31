// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(rename = "$schema")]
    pub schema: DocumentSchemaUri,
    // `contentVersion` is required by ARM, but doesn't serve a purpose here
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, Parameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, Value>>,
    pub resources: Vec<Resource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub parameter_type: DataType,
    #[serde(rename = "defaultValue", skip_serializing_if = "Option::is_none")]
    pub default_value: Option<Value>,
    #[serde(rename = "allowedValues", skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<Value>>,
    #[serde(rename = "minValue", skip_serializing_if = "Option::is_none")]
    pub min_value: Option<Value>,
    #[serde(rename = "maxValue", skip_serializing_if = "Option::is_none")]
    pub max_value: Option<Value>,
    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<Value>,
    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum DataType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "securestring")]
    SecureString,
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "object")]
    Object,
    #[serde(rename = "secureobject")]
    SecureObject,
    #[serde(rename = "array")]
    Array,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Resource {
    /// The fully qualified name of the resource type
    #[serde(rename = "type")]
    pub resource_type: String,
    // TODO: `apiVersion` is required by ARM but doesn't make sense here

    /// A friendly name for the resource instance
    pub name: String, // friendly unique instance name
    #[serde(rename = "dependsOn", skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = r"^\[resourceId\(\s*'[a-zA-Z0-9\.]+/[a-zA-Z0-9]+'\s*,\s*'[a-zA-Z0-9 ]+'\s*\)]$"))]
    pub depends_on: Option<Vec<String>>,
    // `identity` can be used for run-as
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Value>>,
}

// Defines the valid and recognized canonical URIs for the configuration schema
#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum DocumentSchemaUri {
    #[default]
    #[serde(rename = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json")]
    Version2023_10,
    #[serde(rename = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/bundled/config/document.json")]
    Bundled2023_10,
    #[serde(rename = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/bundled/config/document.vscode.json")]
    VSCode2023_10,
    #[serde(rename = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json")]
    Version2023_08,
    #[serde(rename = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/bundled/config/document.json")]
    Bundled2023_08,
    #[serde(rename = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/bundled/config/document.vscode.json")]
    VSCode2023_08,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            schema: DocumentSchemaUri::Version2023_08,
            parameters: None,
            variables: None,
            resources: Vec::new(),
            metadata: None,
        }
    }
}

impl Configuration {
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema: DocumentSchemaUri::Version2023_08,
            parameters: None,
            variables: None,
            resources: Vec::new(),
            metadata: None,
        }
    }
}

impl Resource {
    #[must_use]
    pub fn new() -> Self {
        Self {
            resource_type: String::new(),
            name: String::new(),
            depends_on: None,
            properties: None,
        }
    }
}

impl Default for Resource {
    fn default() -> Self {
        Self::new()
    }
}
