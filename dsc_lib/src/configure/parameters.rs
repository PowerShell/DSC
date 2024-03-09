// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt::{self, Display, Formatter}};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Input {
    pub parameters: HashMap<String, Value>,
}

pub struct SecureString {
    pub value: String,
}

impl Display for SecureString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<SecureString>")
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        self.value.clear();
    }
}

pub struct SecureObject {
    pub value: Value,
}

impl Display for SecureObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<SecureObject>")
    }
}

impl Drop for SecureObject {
    fn drop(&mut self) {
        self.value = Value::Null;
    }
}
