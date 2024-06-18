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
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "Registry", deny_unknown_fields)]
pub struct Registry {
    /// The path to the registry key.
    #[serde(rename = "keyPath")]
    pub key_path: String,
    /// The information from a config set --what-if operation.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub what_if: Option<WhatIf>,
    /// The name of the registry value.
    #[serde(rename = "valueName", skip_serializing_if = "Option::is_none")]
    pub value_name: Option<String>,
    /// The data of the registry value.
    #[serde(rename = "valueData", skip_serializing_if = "Option::is_none")]
    pub value_data: Option<RegistryValueData>,
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WhatIf {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>
}
