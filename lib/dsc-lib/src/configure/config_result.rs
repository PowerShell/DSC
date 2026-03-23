// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::dscresources::invoke_result::{GetResult, SetResult, TestResult};
use crate::configure::config_doc::{Configuration, ExecutionInformation, Metadata};
use crate::schemas::{dsc_repo::DscRepoSchema, transforms::idiomaticize_string_enum};
use crate::types::FullyQualifiedTypeName;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(transform = idiomaticize_string_enum)]
pub enum MessageLevel {
    Error,
    Warning,
    Information,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(base_name = "message", folder_path = "definitions")]
pub struct ResourceMessage {
    pub name: String,
    #[serde(rename="type")]
    pub resource_type: FullyQualifiedTypeName,
    pub message: String,
    pub level: MessageLevel,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(base_name = "get.full", folder_path = "outputs/resource")]
pub struct ResourceGetResult {
    #[serde(rename = "executionInformation", skip_serializing_if = "Option::is_none")]
    pub execution_information: Option<ExecutionInformation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    pub name: String,
    #[serde(rename="type")]
    pub resource_type: FullyQualifiedTypeName,
    pub result: GetResult,
}

impl From<ResourceTestResult> for ResourceGetResult {
    fn from(test_result: ResourceTestResult) -> Self {
        Self {
            execution_information: None,
            metadata: None,
            name: test_result.name,
            resource_type: test_result.resource_type,
            result: test_result.result.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "get", folder_path = "outputs/config")]
pub struct ConfigurationGetResult {
    pub execution_information: Option<ExecutionInformation>,
    pub metadata: Option<Metadata>,
    pub results: Vec<ResourceGetResult>,
    pub messages: Vec<ResourceMessage>,
    pub had_errors: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Map<String, Value>>,
}

impl ConfigurationGetResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
            execution_information: None,
            metadata: None,
            results: Vec::new(),
            messages: Vec::new(),
            had_errors: false,
            outputs: None,
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
            execution_information: None,
            metadata: None,
            results,
            messages: test_result.messages,
            had_errors: test_result.had_errors,
            outputs: test_result.outputs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(base_name = "set.full", folder_path = "outputs/resource")]
pub struct ResourceSetResult {
    #[serde(rename = "executionInformation", skip_serializing_if = "Option::is_none")]
    pub execution_information: Option<ExecutionInformation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    pub name: String,
    #[serde(rename="type")]
    pub resource_type: FullyQualifiedTypeName,
    pub result: SetResult,
}

impl From<ResourceTestResult> for ResourceSetResult {
    fn from(test_result: ResourceTestResult) -> Self {
        Self {
            execution_information: None,
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "set", folder_path = "outputs/config")]
pub struct ConfigurationSetResult {
    pub execution_information: Option<ExecutionInformation>,
    pub metadata: Option<Metadata>,
    pub results: Vec<ResourceSetResult>,
    pub messages: Vec<ResourceMessage>,
    pub had_errors: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Map<String, Value>>,
}

impl ConfigurationSetResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
            execution_information: None,
            metadata: None,
            results: Vec::new(),
            messages: Vec::new(),
            had_errors: false,
            outputs: None,
        }
    }
}

impl Default for ConfigurationSetResult {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(base_name = "test.full", folder_path = "outputs/resource")]
pub struct ResourceTestResult {
    #[serde(rename = "executionInformation", skip_serializing_if = "Option::is_none")]
    pub execution_information: Option<ExecutionInformation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    pub name: String,
    #[serde(rename="type")]
    pub resource_type: FullyQualifiedTypeName,
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "test", folder_path = "outputs/config")]
pub struct ConfigurationTestResult {
    pub execution_information: Option<ExecutionInformation>,
    pub metadata: Option<Metadata>,
    pub results: Vec<ResourceTestResult>,
    pub messages: Vec<ResourceMessage>,
    pub had_errors: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Map<String, Value>>,
}

impl ConfigurationTestResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
            execution_information: None,
            metadata: None,
            results: Vec::new(),
            messages: Vec::new(),
            had_errors: false,
            outputs: None,
        }
    }
}

impl Default for ConfigurationTestResult {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "export", folder_path = "outputs/config")]
pub struct ConfigurationExportResult {
    pub execution_information: Option<ExecutionInformation>,
    pub metadata: Option<Metadata>,
    pub result: Option<Configuration>,
    pub messages: Vec<ResourceMessage>,
    pub had_errors: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Map<String, Value>>,
}

impl ConfigurationExportResult {
    #[must_use]
    pub fn new() -> Self {
        Self {
            execution_information: None,
            metadata: None,
            result: None,
            messages: Vec::new(),
            had_errors: false,
            outputs: None,
        }
    }
}

impl Default for ConfigurationExportResult {
    fn default() -> Self {
        Self::new()
    }
}
