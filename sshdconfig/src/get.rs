// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
use {
    registry_lib::{config::{Registry, RegistryValueData}, RegistryHelper},
    crate::args::DefaultShell,
    crate::metadata::windows::{DEFAULT_SHELL, DEFAULT_SHELL_CMD_OPTION, DEFAULT_SHELL_ESCAPE_ARGS, REGISTRY_PATH},
};

use rust_i18n::t;
use serde_json::{Map, Value};
use tracing::debug;

use crate::args::Setting;
use crate::error::SshdConfigError;
use crate::export::invoke_export_to_map;
use crate::util::extract_sshd_defaults;

/// Invoke the get command.
///
/// # Errors
///
/// This function will return an error if the desired settings cannot be retrieved.
pub fn invoke_get(exclude_defaults: bool, input: Option<&String>, setting: &Setting) -> Result<(), SshdConfigError> {
    debug!("{}: {:?}", t!("get.debugSetting").to_string(), setting);
    match *setting {
        Setting::SshdConfig => get_sshd_settings(exclude_defaults, input),
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

fn get_sshd_settings(exclude_defaults: bool, input: Option<&String>) -> Result<(), SshdConfigError> {
    let mut result = invoke_export_to_map()?;

    if exclude_defaults {
        let defaults = extract_sshd_defaults()?;
        // Filter result based on default settings.
        // If a value in result is equal to the default, it will be excluded.
        // Note that this excludes all defaults, even if they are explicitly set in sshd_config.
        result = result.into_iter()
            .filter(|(key, value)| {
                if let Some(default) = defaults.get(key) {
                    default != value
                } else {
                    true
                }
            })
            .collect();
    }

    if let Some(config) = input {
        // Filter result based on the keys provided in the input JSON.
        // If a provided key is not found in the result, its value is null.
        let input_config: Map<String, Value> = serde_json::from_str(config)?;
        result = input_config
            .keys()
            .map(|key| {
                let value = result.get(key)
                    .cloned()
                    .unwrap_or(Value::Null);
                (key.clone(), value)
            })
            .collect();
    }

    let json = serde_json::to_string(&result)?;
    println!("{json}");
    Ok(())
}
