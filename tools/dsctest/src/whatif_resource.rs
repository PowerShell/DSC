// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WhatIfResource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub what_if_mode: Option<bool>,
}

impl WhatIfResource {
    pub fn new(name: String, value: String, what_if_mode: bool) -> Self {
        WhatIfResource {
            name: Some(name),
            value: Some(value),
            what_if_mode: Some(what_if_mode),
        }
    }
}
