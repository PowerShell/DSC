// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde_json::{Map, Value};

use crate::error::SshdConfigError;
use crate::inputs::SshdCommandArgs;
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
pub fn invoke_export(sshd_args: Option<SshdCommandArgs>) -> Result<Map<String, Value>, SshdConfigError> {
    let sshd_config_text = invoke_sshd_config_validation(sshd_args)?;
    let sshd_config: Map<String, Value> = parse_text_to_map(&sshd_config_text)?;
    Ok(sshd_config)
}
