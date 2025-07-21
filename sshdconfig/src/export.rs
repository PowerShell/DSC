// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde_json::{Map, Value};

use crate::error::SshdConfigError;
use crate::parser::parse_text_to_map;
use crate::util::invoke_sshd_config_validation;

/// Invoke the export command and return a map.
///
/// # Errors
///
/// This function will return an error if the command cannot invoke sshd -T, parse the return, or convert it to json.
///
/// # Returns
///
/// This function will return `Ok(Map<String, Value>)` if the export is successful.
pub fn invoke_export_to_map() -> Result<Map<String, Value>, SshdConfigError> {
    let sshd_config_text = invoke_sshd_config_validation(None)?;
    let sshd_config: Map<String, Value> = parse_text_to_map(&sshd_config_text)?;
    Ok(sshd_config)
}

/// Invoke the export command and print the result as JSON.
///
/// # Errors
/// This function will return an error if the export fails to retrieve the sshd configuration or convert it to JSON.
///
/// # Returns
///
/// This function will return `Ok(())` if the export is successful.
pub fn invoke_export() -> Result<(), SshdConfigError> {
    let result = invoke_export_to_map()?;
    let json = serde_json::to_string(&result)?;
    println!("{json}");
    Ok(())
}
