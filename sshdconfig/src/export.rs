// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::error::SshdConfigError;
use crate::parser::parse_text_to_map;
use crate::util::invoke_sshd_config_validation;

/// Invoke the export command.
///
/// # Errors
///
/// This function will return an error if the command cannot invoke sshd -T, parse the return, or convert it to json.
pub fn invoke_export() -> Result<String, SshdConfigError> {
    let sshd_config_text = invoke_sshd_config_validation()?;
    let sshd_config: serde_json::Map<String, serde_json::Value> = parse_text_to_map(&sshd_config_text)?;
    let json = serde_json::to_string_pretty(&sshd_config)?;
    Ok(json)
}
