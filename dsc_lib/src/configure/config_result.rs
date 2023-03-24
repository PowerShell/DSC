use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::dscresources::invoke_result::{GetResult, SetResult, TestResult};

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
    pub errors: Vec<String>,
}

impl ConfigurationGetResult {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            errors: Vec::new(),
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
    pub errors: Vec<String>,
}

impl ConfigurationSetResult {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            errors: Vec::new(),
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
    pub errors: Vec<String>,
}

impl ConfigurationTestResult {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            errors: Vec::new(),
        }
    }
}
