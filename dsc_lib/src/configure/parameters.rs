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
#[serde(deny_unknown_fields, untagged)]
pub enum SecureKind {
    #[serde(rename = "secureString")]
    SecureString(String),
    #[serde(rename = "secureObject")]
    SecureObject(Value),
}

impl Display for SecureKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecureKind::SecureString(_) => write!(f, "<secureString>"),
            SecureKind::SecureObject(_) => write!(f, "<secureObject>"),
        }
    }
}
