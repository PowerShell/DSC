// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// # Ensure
/// The Ensure enum is used to specify whether a registry key or value should be present or absent.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum EnsureKind {
    /// The registry key and value should be present.
    Present,
    /// The registry key and value should be absent.
    Absent,
}

/// # Registry value data
/// Defines the type and value for the registry value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum RegistryValueData {
    /// # String value data
    /// The value is an arbitrary string that doesn't rely on expanded references to environment
    /// variables.
    String(String),
    /// # Expandable string value data
    /// The value is a string that can contain references to environment variables that are expanded
    /// from their reference, like `%SYSTEMROOT%\System32`.
    ExpandString(String),
    /// # Binary value data
    /// The value is any form of binary data.
    Binary(Vec<u8>),
    /// # DWord value data
    /// The value is a 32-bit unsigned integer.
    DWord(u32),
    /// # Multi-string value data
    /// The value is an array of arbitrary strings.
    MultiString(Vec<String>),
    /// # QWord value data
    /// The value is a 64-bit unsigned integer.
    QWord(u64),
}

/// # Microsoft.Windows/Registry resource instance schema
/// An instance of the Registry resource represents a registry key or value. The resource can
/// manage registry keys and values.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "Registry", deny_unknown_fields)]
pub struct RegistryConfig {
    /// # Schema identifier
    /// The ID of the resource instance schema. This property is read-only.
    #[serde(rename = "$id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// # Key path
    /// Defines the path to the registry key, like
    /// `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion`.
    #[serde(rename = "keyPath")]
    pub key_path: String,
    /// # Value name
    /// Defines the name of the registry value to manage in the key. This property is required when
    /// `valueData` is specified.
    #[serde(rename = "valueName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_name: Option<String>,
    #[serde(rename = "valueData")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_data: Option<RegistryValueData>,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
    /// # Clobber
    /// Indicates whether the registry value should be overwritten if it already exists.
    #[serde(rename = "_clobber")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clobber: Option<bool>,
    /// # Instance is in the desired state
    /// Indicates whether the instance is in the desired state. This property is read-only.
    #[serde(rename = "_inDesiredState")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_desired_state: Option<bool>,
}

impl RegistryConfig {
    pub fn to_json(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Failed to serialize to JSON: {}", e);
                String::new()
            }
        }
    }
}

const ID: &str = "https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json";

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            id: Some(ID.to_string()),
            key_path: String::new(),
            value_name: None,
            value_data: None,
            ensure: None,
            clobber: None,
            in_desired_state: None,
        }
    }
}
