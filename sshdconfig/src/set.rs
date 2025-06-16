// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use registry_lib::{config::{Registry, RegistryValueData}, RegistryHelper};
use serde_json::{json, Value};
use std::{path::Path, process::Command};

use crate::args::SetCommand;
use crate::error::SshdConfigError;
use crate::metadata::RepeatableKeyword;

/// Invoke the set command.
///
/// # Errors
///
/// This function will return an error if the sshd_config is not updated with the desired settings.
pub fn invoke_set(input: &Option<String>, subcommand: &Option<SetCommand>) -> Result<(), SshdConfigError> {
    if let Some(subcommand) = subcommand {
        match subcommand {
            SetCommand::DefaultShell { shell, cmd_option, escape_arguments, shell_arguments } => {
                set_default_shell(shell, cmd_option, escape_arguments, shell_arguments)
            }
            SetCommand::RepeatableKeyworld { keyword, name, value } => {
                set_repeatable_keyword(keyword, name, value)
            }
        }
    } else {
        set_regular_keywords(input)
    }
}

fn set_default_shell(shell: &String, cmd_option: &Option<String>, escape_arguments: &bool, shell_arguments: &Option<Vec<String>>) -> Result<(), SshdConfigError> {
    let shell_path = Path::new(shell);
    if shell_path.is_relative() {
        if shell_path.components().any(|c| c == std::path::Component::ParentDir) {
            return Err(SshdConfigError::InvalidInput("shell path must not be relative".to_string()));
        }
    }
    // TODO check if binary exists when it is not an absolute path
    if !shell_path.exists() {
        return Err(SshdConfigError::InvalidInput(format!("shell path does not exist: {}", shell)));
    }

    let mut shell_data = shell.clone();
    if let Some(shell_args) = shell_arguments {
        let args_str = shell_args.join(" ");
        shell_data = format!("{} {}", shell, args_str);
    }

    set_registry_default_shell("DefaultShell", RegistryValueData::String(shell_data))?;

    if let Some(cmd_option) = cmd_option {
        set_registry_default_shell("DefaultShellCommandOption", RegistryValueData::String(cmd_option.clone()))?;
    }
    else {
        return Err(SshdConfigError::InvalidInput("cmd_option is required".to_string()));
    }

    let mut escape_data = 0;
    if *escape_arguments {
        escape_data = 1;
    }
    set_registry_default_shell("DefaultShellEscapeArguments ", RegistryValueData::DWord(escape_data))?;

    Ok(())
}

pub fn set_repeatable_keyword(_keyword: &RepeatableKeyword, _name: &String, _value: &Option<String>) -> Result<(), SshdConfigError> {
    // TODO: handle repeat keywords like subsystem
    Ok(())
}

pub fn set_regular_keywords(_input: &Option<String>) -> Result<(), SshdConfigError> {
    // modify or add all values in the input file
    // have a banner that we are managing the file and check that before modifying
    // if the file is managed by this tool, we can modify it
    // if the file is not managed by the tool, we should back it up
    // get existing sshd_config settings from sshd -T
    // get default sshd_config settings from sshd -T
    // save explicit settings
    // for each of the input keys, update it with the new value
    // or insert it if it doesn't exist (above any match statements)
    // write the updated settings back to the sshd_config file
    // ensure sshd -T is valid after the update?
    Ok(())
}

fn set_registry_default_shell(name: &str, data: RegistryValueData) -> Result<(), SshdConfigError> {
    let registry_helper = RegistryHelper::new("HKLM\\SOFTWARE\\OpenSSH", Some(name.to_string()), Some(data))?;
    registry_helper.set()?;
    Ok(())
}
