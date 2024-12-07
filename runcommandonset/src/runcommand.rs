// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct RunCommand {
    pub executable: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,
    // default value for exit code is 0
    #[serde(rename = "exitCode", default, skip_serializing_if = "is_default")]
    pub exit_code: i32,
}

impl RunCommand {
    #[must_use]
    pub fn to_json(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("{}: {e}", t!("runcommand.invalidJson"));
                String::new()
            }
        }
    }
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}
