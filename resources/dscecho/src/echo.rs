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
    #[serde(rename = "secureObject")]
    SecureObject(SecureObject),
    #[serde(rename = "secureString")]
    SecureString(SecureString),
    #[serde(rename = "string")]
    String(String),
    // Object has to be last so it doesn't get matched first
    #[serde(rename = "object")]
    Object(Value),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecureString {
    #[serde(rename = "secureString")]
    pub secure_string: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecureObject {
    #[serde(rename = "secureObject")]
    pub secure_object: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Echo {
    pub output: Output,
    #[serde(rename = "showSecrets", skip_serializing_if = "Option::is_none")]
    pub show_secrets: Option<bool>,
}
