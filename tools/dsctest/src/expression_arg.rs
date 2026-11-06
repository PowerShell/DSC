// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpressionArg {
    pub first: String,
    pub second: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub third: Option<String>,
}
