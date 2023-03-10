use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GetResult {
    pub actual_state: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SetResult {
    pub pre_state: Value,
    pub post_state: Value,
    pub changed_properties: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TestResult {
    pub expected_state: Value,
    pub actual_state: Value,
    pub diff_properties: Option<Vec<String>>,
}
