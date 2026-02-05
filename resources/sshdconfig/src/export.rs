// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde_json::{Map, Value};

use crate::build_command_info;
use crate::canonical_properties::{CanonicalProperty, CanonicalProperties};
use crate::error::SshdConfigError;
use crate::get_sshd_settings;
use crate::repeat_keyword::{find_name_value_entry_index, NameValueEntry};

pub fn invoke_export(input: Option<&String>, compare: bool) -> Result<Map<String, Value>, SshdConfigError> {
    let cmd_info = build_command_info(input, false)?;
    let mut result = get_sshd_settings(&cmd_info, false)?;

    // For sshd-config-repeat operations (single entry like subsystem)
    // Check if the requested entry exists in the actual state
    if compare && !cmd_info.input.is_empty() {
        let mut exist = false;

        for (keyword, input_value) in &cmd_info.input {
            if !CanonicalProperties::is_canonical(keyword) {
                if let Some(actual_value) = result.get(keyword) {
                    if let Value::Array(entries) = actual_value {
                        // As more keywords are supported, different structured formats may be needed.
                        if let Ok(entry) = serde_json::from_value::<NameValueEntry>(input_value.clone()) {
                            let match_value = entry.value.as_deref();
                            if find_name_value_entry_index(entries, &entry.name, match_value).is_some() {
                                exist = true;
                            }
                        }
                    } else {
                        // Direct value comparison for non-array keywords
                        if actual_value == input_value {
                            exist = true;
                        }
                    }
                }
            }
            // The only result should be the input to match the expected output for DSC
            result.insert(keyword.clone(), input_value.clone());
        }
        result.insert(CanonicalProperty::Exist.to_string(), Value::Bool(exist));
    }
    Ok(result)
}
