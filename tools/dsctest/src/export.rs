// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Export {
    /// Number of instances to return
    pub count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _name: Option<String>,
    #[serde(rename = "_securityContext", skip_serializing_if = "Option::is_none")]
    pub _security_context: Option<String>,
}
