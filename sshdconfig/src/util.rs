// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::process::Command;

use crate::error::SshdConfigError;
use rust_i18n::t;

/// Invoke sshd -T.
///
/// # Errors
///
/// This function will return an error if sshd -T fails to validate `sshd_config`.
pub fn invoke_sshd_config_validation() -> Result<String, SshdConfigError> {
    let sshd_command = if cfg!(target_os = "windows") {
        "sshd.exe"
    } else {
        "sshd"
    };

    let output = Command::new(sshd_command)
        .arg("-T")
        .output()
        .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
        Ok(stdout)
    } else {
        let stderr = String::from_utf8(output.stderr)
            .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
        if stderr.contains("sshd: no hostkeys available") || stderr.contains("Permission denied") {
            return Err(SshdConfigError::CommandError(
                t!("util.sshdNoHostkeys").to_string()
            ));
        }
        Err(SshdConfigError::CommandError(stderr))
    }
}
