// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
use {
    std::path::Path,
    dsc_lib_registry::{config::RegistryValueData, RegistryHelper},
    crate::metadata::windows::{DEFAULT_SHELL, DEFAULT_SHELL_CMD_OPTION, DEFAULT_SHELL_ESCAPE_ARGS, REGISTRY_PATH},
};

use rust_i18n::t;
use serde_json::{Map, Value};

use crate::args::DefaultShell;
use crate::error::SshdConfigError;

/// Invoke the set command.
///
/// # Errors
///
/// This function will return an error if the desired settings cannot be applied.
pub fn invoke_set(input: &str) -> Result<Map<String, Value>, SshdConfigError> {
    match serde_json::from_str::<DefaultShell>(input) {
        Ok(default_shell) => {
            set_default_shell(default_shell.shell, default_shell.cmd_option, default_shell.escape_arguments)?;
            Ok(Map::new())
        },
        Err(e) => {
            Err(SshdConfigError::InvalidInput(t!("set.failedToParseInput", error = e).to_string()))
        }
    }
}

#[cfg(windows)]
fn set_default_shell(shell: Option<String>, cmd_option: Option<String>, escape_arguments: Option<bool>) -> Result<(), SshdConfigError> {
    if let Some(shell) = shell {
        // TODO: if shell contains quotes, we need to remove them
        let shell_path = Path::new(&shell);
        if shell_path.is_relative() && shell_path.components().any(|c| c == std::path::Component::ParentDir) {
            return Err(SshdConfigError::InvalidInput(t!("set.shellPathMustNotBeRelative").to_string()));
        }
        if !shell_path.exists() {
            return Err(SshdConfigError::InvalidInput(t!("set.shellPathDoesNotExist", shell = shell).to_string()));
        }

        set_registry(DEFAULT_SHELL, RegistryValueData::String(shell))?;
    } else {
        remove_registry(DEFAULT_SHELL)?;
    }


    if let Some(cmd_option) = cmd_option {
        set_registry(DEFAULT_SHELL_CMD_OPTION, RegistryValueData::String(cmd_option.clone()))?;
    } else {
        remove_registry(DEFAULT_SHELL_CMD_OPTION)?;
    }

    if let Some(escape_args) = escape_arguments {
        let mut escape_data = 0;
        if escape_args {
            escape_data = 1;
        }
        set_registry(DEFAULT_SHELL_ESCAPE_ARGS, RegistryValueData::DWord(escape_data))?;
    } else {
        remove_registry(DEFAULT_SHELL_ESCAPE_ARGS)?;
    }

    Ok(())
}

#[cfg(not(windows))]
fn set_default_shell(_shell: Option<String>, _cmd_option: Option<String>, _escape_arguments: Option<bool>) -> Result<(), SshdConfigError> {
    Err(SshdConfigError::InvalidInput(t!("get.windowsOnly").to_string()))
}

#[cfg(windows)]
fn set_registry(name: &str, data: RegistryValueData) -> Result<(), SshdConfigError> {
    let registry_helper = RegistryHelper::new(REGISTRY_PATH, Some(name.to_string()), Some(data))?;
    registry_helper.set()?;
    Ok(())
}

#[cfg(windows)]
fn remove_registry(name: &str) -> Result<(), SshdConfigError> {
    let registry_helper = RegistryHelper::new(REGISTRY_PATH, Some(name.to_string()), None)?;
    registry_helper.remove()?;
    Ok(())
}
