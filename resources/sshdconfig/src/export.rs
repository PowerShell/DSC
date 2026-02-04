// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde_json::{Map, Value};

use crate::build_command_info;
use crate::error::SshdConfigError;
use crate::get_sshd_settings;

pub fn invoke_export(input: Option<&String>, compare: bool) -> Result<Map<String, Value>, SshdConfigError> {
    let cmd_info = build_command_info(input, false)?;
    let mut result = get_sshd_settings(&cmd_info, false)?;
    if compare {
        let mut exist = false;
        for (key, input_value) in &cmd_info.input {
            if key != "_metadata" {
                if let Some(result_value) = result.get(key) {
                    if input_value == result_value {
                        exist = true;
                    }
                }
            }
        }
        result.insert("_exist".to_string(), serde_json::Value::Bool(exist));
    }
    Ok(result)
}
