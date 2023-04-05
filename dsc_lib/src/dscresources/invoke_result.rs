// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JsonResult {
    pub json: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetResult {
    /// The state of the resource as it was returned by the Get method.
    pub actual_state: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetResult {
    /// The state of the resource as it was before the Set method was called.
    pub before_state: Value,
    /// The state of the resource as it was after the Set method was called.
    pub after_state: Value,
    /// The properties that were changed by the Set method from the before state.
    pub changed_properties: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TestResult {
    /// The state of the resource as it was expected to be.
    pub expected_state: Value,
    /// The state of the resource as it was returned by the Get method.
    pub actual_state: Value,
    /// The properties that were different from the expected state.
    pub diff_properties: Option<Vec<String>>,
}
