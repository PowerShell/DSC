use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::dscresources::invoke_result::{GetResult, SetResult, TestResult};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationGetResult {
    pub resources: Vec<GetResult>,
    pub errors: Vec<String>,
}

impl ConfigurationGetResult {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationSetResult {
    pub resources: Vec<SetResult>,
    pub errors: Vec<String>,
}

impl ConfigurationSetResult {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            errors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationTestResult {
    pub resources: Vec<TestResult>,
    pub errors: Vec<String>,
}

impl ConfigurationTestResult {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            errors: Vec::new(),
        }
    }
}
