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
    pub actual_state: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetResult {
    pub before_state: Value,
    pub after_state: Value,
    pub changed_properties: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TestResult {
    pub expected_state: Value,
    pub actual_state: Value,
    pub diff_properties: Option<Vec<String>>,
}
