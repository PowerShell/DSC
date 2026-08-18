// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDefault {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(default = "default_enabled")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(default = "default_count")]
    pub count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nested: Option<Nested>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referenced_nested: Option<Nested>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Nested {
    pub value: String,
    #[serde(skip_serializing)]
    pub secret: Option<Secret>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Secret {
    pub token: String,
}

fn default_enabled() -> Option<bool> {
    Some(true)
}

fn default_count() -> Option<i32> {
    Some(5)
}
