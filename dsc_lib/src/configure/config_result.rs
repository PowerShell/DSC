// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::config_doc::{Configuration, Metadata};
use crate::dscresources::invoke_result::{GetResult, SetResult, TestResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum MessageLevel {
    Error,
    Warning,
    Information,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceMessage {
    pub name: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    pub message: String,
    pub level: MessageLevel,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceGetResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    pub name: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    pub result: GetResult,
}

impl From<ResourceTestResult> for ResourceGetResult {
    fn from(test_result: ResourceTestResult) -> Self {
        Self {
            metadata: None,
            name: test_result.name,
            resource_type: test_result.resource_type,
            result: test_result.result.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationGetResult {
    pub metadata: Option<Metadata>,
    pub results: Vec<ResourceGetResult>,
    pub messages: Vec<ResourceMessage>,
    #[serde(rename = "hadErrors")]
    pub had_errors: bool,
}

impl ConfigurationGetResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: None,
            results: Vec::new(),
            messages: Vec::new(),
            had_errors: false,
        }
    }
}

impl Default for ConfigurationGetResult {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ConfigurationTestResult> for ConfigurationGetResult {
    fn from(test_result: ConfigurationTestResult) -> Self {
        let mut results = Vec::<ResourceGetResult>::new();
        for result in test_result.results {
            results.push(result.into());
        }
        Self {
            metadata: None,
            results,
            messages: test_result.messages,
            had_errors: test_result.had_errors,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceSetResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    pub name: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    pub result: SetResult,
}

impl From<ResourceTestResult> for ResourceSetResult {
    fn from(test_result: ResourceTestResult) -> Self {
        Self {
            metadata: None,
            name: test_result.name,
            resource_type: test_result.resource_type,
            result: test_result.result.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GroupResourceSetResult {
    pub results: Vec<ResourceSetResult>,
}

impl GroupResourceSetResult {
    #[must_use]
    pub fn new() -> Self {
        Self { results: Vec::new() }
    }
}

impl Default for GroupResourceSetResult {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationSetResult {
    pub metadata: Option<Metadata>,
    pub results: Vec<ResourceSetResult>,
    pub messages: Vec<ResourceMessage>,
    #[serde(rename = "hadErrors")]
    pub had_errors: bool,
}

impl ConfigurationSetResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: None,
            results: Vec::new(),
            messages: Vec::new(),
            had_errors: false,
        }
    }
}

impl Default for ConfigurationSetResult {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceTestResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    pub name: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    pub result: TestResult,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GroupResourceTestResult {
    pub results: Vec<ResourceTestResult>,
}

impl GroupResourceTestResult {
    #[must_use]
    pub fn new() -> Self {
        Self { results: Vec::new() }
    }
}

impl Default for GroupResourceTestResult {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationTestResult {
    pub metadata: Option<Metadata>,
    pub results: Vec<ResourceTestResult>,
    pub messages: Vec<ResourceMessage>,
    #[serde(rename = "hadErrors")]
    pub had_errors: bool,
}

impl ConfigurationTestResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: None,
            results: Vec::new(),
            messages: Vec::new(),
            had_errors: false,
        }
    }
}

impl Default for ConfigurationTestResult {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationExportResult {
    pub metadata: Option<Metadata>,
    pub result: Option<Configuration>,
    pub messages: Vec<ResourceMessage>,
    #[serde(rename = "hadErrors")]
    pub had_errors: bool,
}

impl ConfigurationExportResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: None,
            result: None,
            messages: Vec::new(),
            had_errors: false,
        }
    }
}

impl Default for ConfigurationExportResult {
    fn default() -> Self {
        Self::new()
    }
}
