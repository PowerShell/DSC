// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct SchemaDefault {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(default = "default_enabled")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(default = "default_count")]
    pub count: Option<i32>,
}
fn default_enabled() -> Option<bool> {
    Some(true)
}
fn default_count() -> Option<i32> {
    Some(5)
}
