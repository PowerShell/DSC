// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetResult {
    /// The state of the resource as it was returned by the Get method.
    #[serde(rename = "actualState")]
    pub actual_state: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetResult {
    /// The state of the resource as it was before the Set method was called.
    #[serde(rename = "beforeState")]
    pub before_state: Value,
    /// The state of the resource as it was after the Set method was called.
    #[serde(rename = "afterState")]
    pub after_state: Value,
    /// The properties that were changed by the Set method from the before state.
    #[serde(rename = "changedProperties")]
    pub changed_properties: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TestResult {
    /// The state of the resource as it was expected to be.
    #[serde(rename = "expectedState")]
    pub expected_state: Value,
    /// The state of the resource as it was returned by the Get method.
    #[serde(rename = "actualState")]
    pub actual_state: Value,
    /// Whether the resource was in the desired state.
    #[serde(rename = "inDesiredState")]
    pub in_desired_state: bool,
    /// The properties that were different from the expected state.
    #[serde(rename = "differingProperties")]
    pub diff_properties: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidateResult {
    /// Whether the supplied configuration is valid.
    pub valid: bool,
    /// Reason for the validation result.
    pub reason: Option<String>,
}
