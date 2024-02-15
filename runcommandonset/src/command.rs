// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct Command {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    pub executable: String,
    // default value for exit code is 0
    #[serde(default, skip_serializing_if = "is_default")]
    pub exit_code: i32,
}

impl Command {
    pub fn to_json(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Failed to serialize to JSON: {e}");
                String::new()
            }
        }
    }
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}