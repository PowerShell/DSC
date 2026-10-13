// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StateAndDiff {
    #[serde(rename = "_inDesiredState", skip_serializing_if = "Option::is_none")]
    pub in_desired_state: Option<bool>,
    #[serde(rename = "valueOne")]
    pub value_one: i32,
    #[serde(rename = "valueTwo")]
    pub value_two: i32,
}
