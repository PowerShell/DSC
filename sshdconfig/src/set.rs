// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use registry_lib::{config::RegistryValueData, RegistryHelper};
use std::path::Path;

use crate::args::DefaultShell;
use crate::error::SshdConfigError;

/// Invoke the set command.
///
/// # Errors
///
/// This function will return an error if the desired settings cannot be applied.
pub fn invoke_set(input: &str) -> Result<(), SshdConfigError> {
    match serde_json::from_str::<DefaultShell>(input) {
        Ok(default_shell) => {
            set_default_shell(default_shell.shell, default_shell.cmd_option, default_shell.escape_arguments, default_shell.shell_arguments)
        },
        Err(e) => {
            // TODO: handle other commands like repeatable keywords or sshd_config modifications
            Err(SshdConfigError::InvalidInput(format!("Failed to parse input as DefaultShell: {e}")))
        }
    }
}

fn set_default_shell(shell: Option<String>, cmd_option: Option<String>, escape_arguments: Option<bool>, shell_arguments: Option<Vec<String>>) -> Result<(), SshdConfigError> {
    if let Some(shell) = shell {
        let shell_path = Path::new(&shell);
        if shell_path.is_relative() && shell_path.components().any(|c| c == std::path::Component::ParentDir) {
            return Err(SshdConfigError::InvalidInput("shell path must not be relative".to_string()));
        }
        if !shell_path.exists() {
            return Err(SshdConfigError::InvalidInput(format!("shell path does not exist: {shell}")));
        }

        let mut shell_data = shell.clone();
        if let Some(shell_args) = shell_arguments {
            let args_str = shell_args.join(" ");
            shell_data = format!("{shell} {args_str}");
        }

        set_registry("DefaultShell", RegistryValueData::String(shell_data))?;
    } else {
        remove_registry("DefaultShell")?;
    }


    if let Some(cmd_option) = cmd_option {
        set_registry("DefaultShellCommandOption", RegistryValueData::String(cmd_option.clone()))?;
    } else {
        remove_registry("DefaultShellCommandOption")?;
    }

    if let Some(escape_args) = escape_arguments {
        let mut escape_data = 0;
        if escape_args {
            escape_data = 1;
        }
        set_registry("DefaultShellEscapeArguments", RegistryValueData::DWord(escape_data))?;
    } else {
        remove_registry("DefaultShellEscapeArguments")?;
    }

    Ok(())
}

fn set_registry(name: &str, data: RegistryValueData) -> Result<(), SshdConfigError> {
    let registry_helper = RegistryHelper::new("HKLM\\SOFTWARE\\OpenSSH", Some(name.to_string()), Some(data))?;
    registry_helper.set()?;
    Ok(())
}

fn remove_registry(name: &str) -> Result<(), SshdConfigError> {
    let registry_helper = RegistryHelper::new("HKLM\\SOFTWARE\\OpenSSH", Some(name.to_string()), None)?;
    registry_helper.remove()?;
    Ok(())
}
