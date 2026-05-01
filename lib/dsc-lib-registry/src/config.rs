// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum RegistryValueData {
    String(String),
    ExpandString(String),
    Binary(Vec<u8>),
    DWord(u32),
    MultiString(Vec<String>),
    QWord(u64),
    None,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "RegistryKey", deny_unknown_fields)]
pub struct RegistryKey {
    /// The path to the registry key.
    #[serde(rename = "keyPath")]
    pub key_path: String,
    /// The information from a config set --what-if operation.
    #[serde(rename = "_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// The name of the registry value.
    #[serde(rename = "valueName", skip_serializing_if = "Option::is_none")]
    pub value_name: Option<String>,
    /// The data of the registry value.
    #[serde(rename = "valueData", skip_serializing_if = "Option::is_none")]
    pub value_data: Option<RegistryValueData>,
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "RegistryList", deny_unknown_fields)]
pub struct RegistryList {
    /// One or more registry keys/values to manage.
    #[serde(rename = "registryKeys")]
    pub registry_keys: Vec<RegistryKey>,
    /// The information from a config set --what-if operation.
    #[serde(rename = "_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Registry {
    List(RegistryList),
    Single(RegistryKey),
}

impl Default for Registry {
    fn default() -> Self {
        Registry::Single(RegistryKey::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    #[serde(rename = "whatIf", skip_serializing_if = "Option::is_none")]
    pub what_if: Option<Vec<String>>
}
