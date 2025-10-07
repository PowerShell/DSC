// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::config_result::{ResourceGetResult, ResourceSetResult, ResourceTestResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum GetResult {
    Resource(ResourceGetResponse),
    Group(Vec<ResourceGetResult>),
}

impl From<TestResult> for GetResult {
    fn from(value: TestResult) -> Self {
        match value {
            TestResult::Group(group) => {
                let mut results = Vec::<ResourceGetResult>::new();
                for result in group {
                    results.push(result.into());
                }
                GetResult::Group(results)
            }
            TestResult::Resource(resource) => GetResult::Resource(ResourceGetResponse {
                actual_state: resource.actual_state,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceGetResponse {
    /// The state of the resource as it was returned by the Get method.
    #[serde(rename = "actualState")]
    pub actual_state: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum SetResult {
    Resource(ResourceSetResponse),
    Group(Vec<ResourceSetResult>),
}

impl From<TestResult> for SetResult {
    fn from(value: TestResult) -> Self {
        match value {
            TestResult::Group(group) => {
                let mut results = Vec::<ResourceSetResult>::new();
                for result in group {
                    results.push(result.into());
                }
                SetResult::Group(results)
            }
            TestResult::Resource(resource) => SetResult::Resource(ResourceSetResponse {
                before_state: resource.actual_state,
                after_state: resource.desired_state,
                changed_properties: if resource.diff_properties.is_empty() {
                    None
                } else {
                    Some(resource.diff_properties)
                },
            }),
        }
    }
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
#[serde(untagged)]
pub enum TestResult {
    Resource(ResourceTestResponse),
    Group(Vec<ResourceTestResult>),
}

#[must_use]
pub fn get_in_desired_state(test_result: &TestResult) -> bool {
    match test_result {
        TestResult::Resource(ref resource_test_result) => resource_test_result.in_desired_state,
        TestResult::Group(ref group_test_result) => {
            for result in group_test_result {
                if !get_in_desired_state(&(result.result)) {
                    return false;
                }
            }
            true
        }
    }
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
    /// The state of the resource as it was returned by the Export method.
    #[serde(rename = "actualState")]
    pub actual_state: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolveResult {
    /// The resolved configuration.
    pub configuration: Value,
    /// The optional resolved parameters.
    pub parameters: Option<HashMap<String, Value>>,
}
