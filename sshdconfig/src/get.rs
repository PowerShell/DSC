// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
use {
    registry_lib::{config::{Registry, RegistryValueData}, RegistryHelper},
    crate::args::{DefaultShell, Resource},
    crate::metadata::{DEFAULT_SHELL, DEFAULT_SHELL_CMD_OPTION, DEFAULT_SHELL_ESCAPE_ARGS, REGISTRY_PATH},
};

use crate::error::SshdConfigError;

/// Invoke the get command.
///
/// # Errors
///
/// This function will return an error if the desired settings cannot be retrieved.
pub fn invoke_get(resource: &Resource) -> Result<(), SshdConfigError> {
    match resource {
        &Resource::DefaultShell => get_default_shell(),
        &Resource::SshdConfig => Err(SshdConfigError::NotImplemented("get not yet implemented for Microsoft.OpenSSH.SSHD/sshd_config".to_string())),
    }
}

#[cfg(windows)]
fn get_default_shell() -> Result<(), SshdConfigError> {
    let registry_helper = RegistryHelper::new(REGISTRY_PATH, Some(DEFAULT_SHELL.to_string()), None)?;
    let default_shell: Registry = registry_helper.get()?;
    let mut shell = None;
    let mut shell_arguments = None;
    // default_shell is a single string consisting of the shell exe path and, optionally, any arguments
    if let Some(value) = default_shell.value_data {
        match value {
            RegistryValueData::String(s) => {
                let parts: Vec<&str> = s.split_whitespace().collect();
                if parts.is_empty() {
                    return Err(SshdConfigError::InvalidInput(format!("{} cannot be empty", DEFAULT_SHELL)));
                }
                shell = Some(parts[0].to_string());
                if parts.len() > 1 {
                    shell_arguments = Some(parts[1..].iter().map(|&s| s.to_string()).collect());
                }
            }
            _ => return Err(SshdConfigError::InvalidInput(format!("{} must be a string", DEFAULT_SHELL))),
        }
    }

    let registry_helper = RegistryHelper::new(REGISTRY_PATH, Some(DEFAULT_SHELL_CMD_OPTION.to_string()), None)?;
    let option: Registry = registry_helper.get()?;
    let mut cmd_option = None;
    if let Some(value) = option.value_data {
        match value {
            RegistryValueData::String(s) => cmd_option = Some(s),
            _ => return Err(SshdConfigError::InvalidInput(format!("{} must be a string", DEFAULT_SHELL_CMD_OPTION))),
        }
    }

    let registry_helper = RegistryHelper::new(REGISTRY_PATH, Some(DEFAULT_SHELL_ESCAPE_ARGS.to_string()), None)?;
    let escape_args: Registry = registry_helper.get()?;
    let mut escape_arguments = None;
    if let Some(value) = escape_args.value_data {
        if let RegistryValueData::DWord(b) = value {
            if b == 0 || b == 1 {
                escape_arguments = if b == 1 { Some(true) } else { Some(false) };
            } else {
                return Err(SshdConfigError::InvalidInput(format!("{} must be a 0 or 1", DEFAULT_SHELL_ESCAPE_ARGS)));
            }
        } else {
            return Err(SshdConfigError::InvalidInput(format!("{} must be a DWord", DEFAULT_SHELL_ESCAPE_ARGS)));
        }
    }

    let result = DefaultShell {
        shell,
        cmd_option,
        escape_arguments,
        shell_arguments
    };

    let output = serde_json::to_string(&result)?;
    println!("{output}");
    Ok(())
}

#[cfg(not(windows))]
pub fn get_default_shell() -> Result<(), SshdConfigError> {
    Err(SshdConfigError::InvalidInput("Microsoft.OpenSSH.SSHD/Windows is only applicable to Windows".to_string()))
}