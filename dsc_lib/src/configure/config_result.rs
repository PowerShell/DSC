// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::dscresources::invoke_result::{GetResult, SetResult, TestResult};
use crate::configure::config_doc;

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
pub struct GroupResourceGetResult {
    pub results: Vec<ResourceGetResult>,
}

impl GroupResourceGetResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
}

impl Default for GroupResourceGetResult {
    fn default() -> Self {
        Self::new()
    }
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
    #[must_use]
    pub fn new() -> Self {
        Self {
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
pub struct GroupResourceSetResult {
    pub results: Vec<ResourceSetResult>,
}

impl GroupResourceSetResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
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
    pub results: Vec<ResourceSetResult>,
    pub messages: Vec<ResourceMessage>,
    #[serde(rename = "hadErrors")]
    pub had_errors: bool,
}

impl ConfigurationSetResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
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
    pub name: String,
    #[serde(rename="type")]
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
        Self {
            results: Vec::new(),
        }
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
    pub results: Vec<ResourceTestResult>,
    pub messages: Vec<ResourceMessage>,
    #[serde(rename = "hadErrors")]
    pub had_errors: bool,
}

impl ConfigurationTestResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
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
    pub result: Option<config_doc::Configuration>,
    pub messages: Vec<ResourceMessage>,
    #[serde(rename = "hadErrors")]
    pub had_errors: bool,
}

impl ConfigurationExportResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
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
