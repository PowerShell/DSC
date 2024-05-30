// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
// #[serde(untagged)]
// pub enum ExecutionKind {
//     #[serde(rename = "actual")]
//     Actual,
//     #[serde(rename = "whatIf")]
//     WhatIf,
// }

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WhatIf {
    #[serde(rename = "executionType")]
    pub execution_type: String,
}
