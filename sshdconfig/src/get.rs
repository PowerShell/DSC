// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
use {
    registry_lib::{config::{Registry, RegistryValueData}, RegistryHelper},
    crate::args::DefaultShell,
    crate::metadata::windows::{DEFAULT_SHELL, DEFAULT_SHELL_CMD_OPTION, DEFAULT_SHELL_ESCAPE_ARGS, REGISTRY_PATH},
};

use rust_i18n::t;
use tracing::debug;

use crate::args::Setting;
use crate::error::SshdConfigError;

/// Invoke the get command.
///
/// # Errors
///
/// This function will return an error if the desired settings cannot be retrieved.
pub fn invoke_get(setting: &Setting) -> Result<(), SshdConfigError> {
    debug!("Get setting: {:?}", setting);
    match *setting {
        Setting::SshdConfig => Err(SshdConfigError::NotImplemented(t!("get.notImplemented").to_string())),
        Setting::WindowsGlobal => get_default_shell()
    }
}

#[cfg(windows)]
fn get_default_shell() -> Result<(), SshdConfigError> {
    let registry_helper = RegistryHelper::new(REGISTRY_PATH, Some(DEFAULT_SHELL.to_string()), None)?;
    let default_shell: Registry = registry_helper.get()?;
    let mut shell = None;
    // default_shell is a single string consisting of the shell exe path
    if let Some(value) = default_shell.value_data {
        match value {
            RegistryValueData::String(s) => {
                shell = Some(s);
            }
            _ => return Err(SshdConfigError::InvalidInput(t!("get.defaultShellMustBeString").to_string())),
        }
    }

    let registry_helper = RegistryHelper::new(REGISTRY_PATH, Some(DEFAULT_SHELL_CMD_OPTION.to_string()), None)?;
    let option: Registry = registry_helper.get()?;
    let mut cmd_option = None;
    if let Some(value) = option.value_data {
        match value {
            RegistryValueData::String(s) => cmd_option = Some(s),
            _ => return Err(SshdConfigError::InvalidInput(t!("get.defaultShellCmdOptionMustBeString").to_string())),
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
                return Err(SshdConfigError::InvalidInput(t!("get.defaultShellEscapeArgsMustBe0Or1", input = b).to_string()));
            }
        } else {
            return Err(SshdConfigError::InvalidInput(t!("get.defaultShellEscapeArgsMustBeDWord").to_string()));
        }
    }

    let result = DefaultShell {
        shell,
        cmd_option,
        escape_arguments
    };

    let output = serde_json::to_string(&result)?;
    println!("{output}");
    Ok(())
}

#[cfg(not(windows))]
fn get_default_shell() -> Result<(), SshdConfigError> {
    Err(SshdConfigError::InvalidInput(t!("get.windowsOnly").to_string()))
}
