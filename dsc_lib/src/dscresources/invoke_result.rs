// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GroupResourceGetResponse {
    pub results: Vec<ResourceGetResponse>,
}

impl GroupResourceGetResponse {
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
}

impl Default for GroupResourceGetResponse {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum GetResult {
    Resource(ResourceGetResponse),
    Group(GroupResourceGetResponse),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceGetResponse {
    /// The state of the resource as it was returned by the Get method.
    #[serde(rename = "actualState")]
    pub actual_state: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GroupResourceSetResponse {
    pub results: Vec<ResourceSetResponse>,
}

impl GroupResourceSetResponse {
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
}

impl Default for GroupResourceSetResponse {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum SetResult {
    Resource(ResourceSetResponse),
    Group(GroupResourceSetResponse),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceSetResponse {
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
pub struct GroupResourceTestResponse {
    pub results: Vec<ResourceTestResponse>,
}

impl GroupResourceTestResponse {
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
}

impl Default for GroupResourceTestResponse {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum TestResult {
    Resource(ResourceTestResponse),
    Group(GroupResourceTestResponse),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceTestResponse {
    /// The state of the resource as it was expected to be.
    #[serde(rename = "desiredState")]
    pub desired_state: Value,
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExportResult {
    /// The state of the resource as it was returned by the Get method.
    #[serde(rename = "actualState")]
    pub actual_state: Vec<Value>,
}
