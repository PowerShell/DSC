// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum ContextKind {
    Configuration,
    Resource,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum SecurityContextKind {
    Current,
    Elevated,
    Restricted,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum Operation {
    Get,
    Set,
    Test,
    Export,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum ExecutionKind {
    Actual,
    WhatIf,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct MicrosoftDscMetadata {
    /// Version of DSC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The operation being performed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<Operation>,
    /// The type of execution
    #[serde(rename = "executionType", skip_serializing_if = "Option::is_none")]
    pub execution_type: Option<ExecutionKind>,
    /// The start time of the configuration operation
    #[serde(rename = "startDatetime", skip_serializing_if = "Option::is_none")]
    pub start_datetime: Option<String>,
    /// The end time of the configuration operation
    #[serde(rename = "endDatetime", skip_serializing_if = "Option::is_none")]
    pub end_datetime: Option<String>,
    /// The duration of the configuration operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    /// The security context of the configuration operation, can be specified to be required
    #[serde(rename = "securityContext", skip_serializing_if = "Option::is_none")]
    pub security_context: Option<SecurityContextKind>,
    /// Identifies if the operation is part of a configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ContextKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Metadata {
    #[serde(rename = "Microsoft.DSC", skip_serializing_if = "Option::is_none")]
    pub microsoft: Option<MicrosoftDscMetadata>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(rename = "$schema")]
    pub schema: DocumentSchemaUri,
    #[serde(rename = "contentVersion")]
    pub content_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, Parameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, Value>>,
    pub resources: Vec<Resource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
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
    pub min_value: Option<i64>,
    #[serde(rename = "maxValue", skip_serializing_if = "Option::is_none")]
    pub max_value: Option<i64>,
    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<i64>,
    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum DataType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "secureString")]
    SecureString,
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "object")]
    Object,
    #[serde(rename = "secureObject")]
    SecureObject,
    #[serde(rename = "array")]
    Array,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Resource {
    /// The fully qualified name of the resource type
    #[serde(rename = "type")]
    pub resource_type: String,
    /// A friendly name for the resource instance
    pub name: String, // friendly unique instance name
    #[serde(rename = "dependsOn", skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = r"^\[resourceId\(\s*'[a-zA-Z0-9\.]+/[a-zA-Z0-9]+'\s*,\s*'[a-zA-Z0-9 ]+'\s*\)]$"))]
    pub depends_on: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

// Defines the valid and recognized canonical URIs for the configuration schema
#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum DocumentSchemaUri {
    #[default]
    #[serde(rename = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json")]
    Version2024_04,
    #[serde(rename = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/bundled/config/document.json")]
    Bundled2024_04,
    #[serde(rename = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/bundled/config/document.vscode.json")]
    VSCode2024_04,
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
        Self::new()
    }
}

impl Configuration {
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema: DocumentSchemaUri::Version2024_04,
            content_version: Some("1.0.0".to_string()),
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
            metadata: None,
        }
    }
}

impl Default for Resource {
    fn default() -> Self {
        Self::new()
    }
}
