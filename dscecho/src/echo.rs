// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum Output {
    #[serde(rename = "array")]
    Array(Vec<Value>),
    #[serde(rename = "bool")]
    Bool(bool),
    #[serde(rename = "number")]
    Number(i64),
    #[serde(rename = "object")]
    Object(Value),
    #[serde(rename = "secureObject")]
    SecureObject(SecureObject),
    #[serde(rename = "secureString")]
    SecureString(SecureString),
    #[serde(rename = "string")]
    String(String),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecureString {
    pub secure_string: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecureObject {
    pub secure_object: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Echo {
    pub output: Output,
}
