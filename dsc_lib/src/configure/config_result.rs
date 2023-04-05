// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::dscresources::invoke_result::{GetResult, SetResult, TestResult};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum MessageLevel {
    Error,
    Warning,
    Information,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceMessage {
    pub name: String,
    #[serde(rename="type")]
    pub resource_type: String,
    pub message: String,
    pub level: MessageLevel,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceGetResult {
    pub name: String,
    #[serde(rename="type")]
    pub resource_type: String,
    pub result: GetResult,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationGetResult {
    pub results: Vec<ResourceGetResult>,
    pub messages: Vec<ResourceMessage>,
    #[serde(rename = "hadErrors")]
    pub had_errors: bool,
}

impl ConfigurationGetResult {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            messages: Vec::new(),
            had_errors: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceSetResult {
    pub name: String,
    #[serde(rename="type")]
    pub resource_type: String,
    pub result: SetResult,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationSetResult {
    pub results: Vec<ResourceSetResult>,
    pub messages: Vec<ResourceMessage>,
    #[serde(rename = "hadErrors")]
    pub had_errors: bool,
}

impl ConfigurationSetResult {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            messages: Vec::new(),
            had_errors: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceTestResult {
    pub name: String,
    #[serde(rename="type")]
    pub resource_type: String,
    pub result: TestResult,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationTestResult {
    pub results: Vec<ResourceTestResult>,
    pub messages: Vec<ResourceMessage>,
    #[serde(rename = "hadErrors")]
    pub had_errors: bool,
}

impl ConfigurationTestResult {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            messages: Vec::new(),
            had_errors: false,
        }
    }
}
