// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
use {
    dsc_lib_registry::{config::{Registry, RegistryValueData}, RegistryHelper},
    crate::args::DefaultShell,
    crate::metadata::windows::{DEFAULT_SHELL, DEFAULT_SHELL_CMD_OPTION, DEFAULT_SHELL_ESCAPE_ARGS, REGISTRY_PATH},
};

use rust_i18n::t;
use serde_json::{Map, Value};
use tracing::{debug, trace, warn};

use crate::args::Setting;
use crate::canonical_properties::CanonicalProperty;
use crate::error::SshdConfigError;
use crate::inputs::CommandInfo;
use crate::parser::parse_text_to_map;
use crate::util::{
    build_command_info,
    extract_sshd_defaults,
    invoke_sshd_config_validation,
    read_sshd_config
};

/// Invoke the get command.
///
/// # Errors
///
/// This function will return an error if the desired settings cannot be retrieved.
pub fn invoke_get(input: Option<&String>, setting: &Setting) -> Result<Map<String, Value>, SshdConfigError> {
    debug!("{} {:?}", t!("get.debugSetting").to_string(), setting);
    trace!("{} {:?}", t!("get.traceInput").to_string(), input);
    match *setting {
        Setting::SshdConfig => {
            let cmd_info = build_command_info(input, true)?;
            get_sshd_settings(&cmd_info, true)
        },
        Setting::WindowsGlobal => {
            get_default_shell()?;
            Ok(Map::new())
        },
        _ => {
            Err(SshdConfigError::InvalidInput(t!("get.invalidSetting", setting = format!("{:?}", setting)).to_string()))
        }
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

/// Retrieve sshd settings.
///
/// # Arguments
///
/// * `cmd_info` - `CommandInfo` struct containing optional filters, metadata, and includeDefaults flag.
///
/// # Errors
///
/// This function will return an error if it cannot retrieve the sshd settings.
pub fn get_sshd_settings(cmd_info: &CommandInfo, is_get: bool) -> Result<Map<String, Value>, SshdConfigError> {
    let sshd_config_text = invoke_sshd_config_validation(cmd_info.sshd_args.clone())?;
    let mut result = parse_text_to_map(&sshd_config_text)?;
    let mut inherited_defaults: Vec<String> = Vec::new();

    // parse settings from sshd_config file
    let sshd_config_file = read_sshd_config(cmd_info.metadata.filepath.clone())?;
    let explicit_settings = parse_text_to_map(&sshd_config_file)?;

    // handle special cases for keywords
    if explicit_settings.contains_key("include") {
        warn!("{}", t!("get.includeWarning").to_string());
    }

    if explicit_settings.contains_key("match") {
        let Some(match_value) = explicit_settings.get("match") else {
            return Err(SshdConfigError::InvalidInput(t!("get.matchParsingError").to_string()));
        };
        result.insert("match".to_string(), match_value.clone());
    }

    if cmd_info.include_defaults {
        // get default from SSHD -T with empty config
        let mut defaults = extract_sshd_defaults()?;
        for key in explicit_settings.keys() {
            if defaults.contains_key(key) {
                defaults.remove(key);
            }
        }
        // Update inherited_defaults with any keys that are not explicitly set
        // check result for any keys that are in defaults
        for (key, value) in &result {
            if let Some(default) = defaults.get(key) {
                if default == value {
                    inherited_defaults.push(key.clone());
                }
            }
        }
    } else {
        // only need to return keys that are explicitly set in the config file
        result.retain(|key, _| explicit_settings.contains_key(key));
    }

    if !cmd_info.input.is_empty() {
        // Filter result based on the keys provided in the input JSON.
        // If a provided key is not found in the result, its value is null.
        result.retain(|key, _| cmd_info.input.contains_key(key));
        inherited_defaults.retain(|key| cmd_info.input.contains_key(key));
        for key in cmd_info.input.keys() {
            result.entry(key.clone()).or_insert(Value::Null);
        }
    }

    if cmd_info.metadata.filepath.is_some() {
        result.insert(CanonicalProperty::Metadata.to_string(), serde_json::to_value(cmd_info.metadata.clone())?);
    }
    if cmd_info.include_defaults && is_get {
        result.insert(CanonicalProperty::InheritedDefaults.to_string(), serde_json::to_value(inherited_defaults)?);
    }
    Ok(result)
}
