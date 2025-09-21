// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Input {
    pub parameters: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct SecureString {
    #[serde(rename = "secureString")]
    pub secure_string: String,
}

impl Display for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<secureString>")
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct SecureObject {
    #[serde(rename = "secureObject")]
    pub secure_object: Value,
}

impl Display for SecureObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<secureObject>")
    }
}

/// Check if a given JSON value is a secure value (either `SecureString` or `SecureObject`).
///
/// # Arguments
///
/// * `value` - The JSON value to check.
///
/// # Returns
///
/// `true` if the value is a secure value, `false` otherwise.
#[must_use]
pub fn is_secure_value(value: &Value) -> bool {
    if let Some(obj) = value.as_object() {
        if obj.len() == 1 && (obj.contains_key("secureString") || obj.contains_key("secureObject")) {
            return true;
        }
    }
    false
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, untagged)]
pub enum SecureKind {
    #[serde(rename = "secureString")]
    SecureString(SecureString),
    #[serde(rename = "secureObject")]
    SecureObject(SecureObject),
}
