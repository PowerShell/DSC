// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WhatIf {
    #[serde(rename = "executionType")]
    pub execution_type: String,
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
    #[serde(rename = "fromResource", skip_serializing_if = "Option::is_none")]
    pub from_resource: Option<String>,
}
