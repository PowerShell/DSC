// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
use {
    crate::metadata::windows::{
        DEFAULT_SHELL, DEFAULT_SHELL_CMD_OPTION, DEFAULT_SHELL_ESCAPE_ARGS, REGISTRY_PATH,
    },
    dsc_lib_registry::{RegistryHelper, config::RegistryValueData},
};

use rust_i18n::t;
use serde_json::{Map, Value};
use std::{
    path::{Path, PathBuf},
    string::String,
};
use tracing::{debug, info, warn};

use crate::args::{DefaultShell, Setting};
use crate::canonical_properties::CanonicalProperties;
use crate::error::SshdConfigError;
use crate::formatter::write_config_map_to_text;
use crate::get::get_sshd_settings;
use crate::inputs::{CommandInfo, SshdCommandArgs};
use crate::metadata::{SSHD_CONFIG_HEADER, SSHD_CONFIG_HEADER_VERSION, SSHD_CONFIG_HEADER_WARNING};
use crate::repeat_keyword::{
    NameValueEntry, RepeatInput, RepeatListInput, add_or_update_entry, extract_single_keyword,
    parse_and_validate_entries, remove_entry,
};
use crate::util::{
    build_command_info, ensure_sshd_config_exists, get_default_sshd_config_path,
    invoke_sshd_config_validation,
};

/// Invoke the set command.
///
/// # Errors
///
/// This function will return an error if the desired settings cannot be applied.
pub fn invoke_set(
    input: &str,
    setting: &Setting,
    what_if: bool,
) -> Result<Map<String, Value>, SshdConfigError> {
    match setting {
        Setting::SshdConfig => {
            debug!("{} {:?}", t!("set.settingSshdConfig").to_string(), setting);
            let mut cmd_info = build_command_info(Some(&input.to_string()), false)?;
            let state = set_sshd_config(&mut cmd_info, what_if)?;
            if what_if { Ok(state) } else { Ok(Map::new()) }
        }
        Setting::SshdConfigRepeat => {
            debug!("{} {:?}", t!("set.settingSshdConfig").to_string(), setting);
            let cmd_info = build_command_info(Some(&input.to_string()), false)?;
            set_sshd_config_repeat(input, &cmd_info, what_if)
        }
        Setting::SshdConfigRepeatList => {
            debug!("{} {:?}", t!("set.settingSshdConfig").to_string(), setting);
            let cmd_info = build_command_info(Some(&input.to_string()), false)?;
            set_sshd_config_repeat_list(input, &cmd_info, what_if)
        }
        Setting::WindowsGlobal => {
            debug!(
                "{} {:?}",
                t!("set.settingDefaultShell").to_string(),
                setting
            );
            match serde_json::from_str::<DefaultShell>(input) {
                Ok(default_shell) => {
                    debug!(
                        "{}",
                        t!(
                            "set.defaultShellDebug",
                            shell = format!("{:?}", default_shell)
                        )
                    );
                    let desired_state = get_default_shell_desired_state(default_shell)?;
                    if what_if {
                        if !cfg!(windows) {
                            return Err(SshdConfigError::InvalidInput(
                                t!("get.windowsOnly").to_string(),
                            ));
                        }
                        default_shell_to_map(&desired_state)
                    } else {
                        set_default_shell(&desired_state)?;
                        Ok(Map::new())
                    }
                }
                Err(e) => Err(SshdConfigError::InvalidInput(
                    t!("set.failedToParseDefaultShell", error = e).to_string(),
                )),
            }
        }
    }
}

/// Handle single name-value keyword entry operations (add or remove).
fn set_sshd_config_repeat(
    input: &str,
    cmd_info: &CommandInfo,
    what_if: bool,
) -> Result<Map<String, Value>, SshdConfigError> {
    let keyword_input: RepeatInput = serde_json::from_str(input).map_err(|e| {
        SshdConfigError::InvalidInput(t!("set.failedToParse", input = e.to_string()).to_string())
    })?;

    let (keyword, entry_value) = extract_single_keyword(keyword_input.additional_properties)?;

    let mut existing_config = get_existing_config(cmd_info, what_if)?;

    // parses entry for name-value keywords, like subsystem, for now
    // different keywords will likely need to be serialized into different structs
    // and likely need to have different add/update/remove functions
    let entry: NameValueEntry = serde_json::from_value(entry_value).map_err(|e| {
        SshdConfigError::InvalidInput(t!("set.failedToParse", input = e.to_string()).to_string())
    })?;

    if keyword_input.exist {
        add_or_update_entry(&mut existing_config, &keyword, &entry)?;
    } else {
        remove_entry(&mut existing_config, &keyword, &entry.name);
    }

    write_and_validate_config(
        &mut existing_config,
        cmd_info.metadata.filepath.as_ref(),
        what_if,
    )?;
    if what_if {
        Ok(existing_config)
    } else {
        Ok(Map::new())
    }
}

/// Handle list name-value keyword operations with purge support.
fn set_sshd_config_repeat_list(
    input: &str,
    cmd_info: &CommandInfo,
    what_if: bool,
) -> Result<Map<String, Value>, SshdConfigError> {
    let list_input: RepeatListInput = serde_json::from_str(input).map_err(|e| {
        SshdConfigError::InvalidInput(t!("set.failedToParse", input = e.to_string()).to_string())
    })?;

    let (keyword, entries_value) = extract_single_keyword(list_input.additional_properties)?;
    let mut existing_config = get_existing_config(cmd_info, what_if)?;
    // Ensure it's an array
    let Value::Array(ref entries_array) = entries_value else {
        return Err(SshdConfigError::InvalidInput(
            t!("set.expectedArrayForKeyword", keyword = keyword).to_string(),
        ));
    };

    // Apply the changes based on _purge flag
    if list_input.purge {
        if entries_array.is_empty() {
            existing_config.remove(&keyword);
        } else {
            existing_config.insert(keyword, entries_value);
        }
    } else {
        let entries = parse_and_validate_entries(entries_array)?;
        for entry in entries {
            add_or_update_entry(&mut existing_config, &keyword, &entry)?;
        }
    }
    write_and_validate_config(
        &mut existing_config,
        cmd_info.metadata.filepath.as_ref(),
        what_if,
    )?;
    if what_if {
        Ok(existing_config)
    } else {
        Ok(Map::new())
    }
}

fn get_default_shell_desired_state(
    default_shell: DefaultShell,
) -> Result<DefaultShell, SshdConfigError> {
    if let Some(shell) = default_shell.shell.as_deref()
        && !shell.is_empty()
    {
        let shell_path = Path::new(shell);
        if shell_path.is_relative()
            && shell_path
                .components()
                .any(|c| c == std::path::Component::ParentDir)
        {
            return Err(SshdConfigError::InvalidInput(
                t!("set.shellPathMustNotBeRelative").to_string(),
            ));
        }
        if !shell_path.exists() {
            return Err(SshdConfigError::InvalidInput(
                t!("set.shellPathDoesNotExist", shell = shell).to_string(),
            ));
        }
    }

    Ok(DefaultShell {
        shell: default_shell.shell.filter(|shell| !shell.is_empty()),
        cmd_option: default_shell.cmd_option,
        escape_arguments: default_shell.escape_arguments,
    })
}

fn default_shell_to_map(
    default_shell: &DefaultShell,
) -> Result<Map<String, Value>, SshdConfigError> {
    let value = serde_json::to_value(default_shell)?;
    match value {
        Value::Object(map) => Ok(map),
        _ => Ok(Map::new()),
    }
}

#[cfg(windows)]
fn set_default_shell(default_shell: &DefaultShell) -> Result<(), SshdConfigError> {
    debug!("{}", t!("set.settingDefaultShell"));
    if let Some(shell) = &default_shell.shell {
        set_registry(DEFAULT_SHELL, RegistryValueData::String(shell.clone()))?;
    } else {
        remove_registry(DEFAULT_SHELL)?;
    }

    if let Some(cmd_option) = &default_shell.cmd_option {
        set_registry(
            DEFAULT_SHELL_CMD_OPTION,
            RegistryValueData::String(cmd_option.clone()),
        )?;
    } else {
        remove_registry(DEFAULT_SHELL_CMD_OPTION)?;
    }

    if let Some(escape_args) = default_shell.escape_arguments {
        let mut escape_data = 0;
        if escape_args {
            escape_data = 1;
        }
        set_registry(
            DEFAULT_SHELL_ESCAPE_ARGS,
            RegistryValueData::DWord(escape_data),
        )?;
    } else {
        remove_registry(DEFAULT_SHELL_ESCAPE_ARGS)?;
    }

    Ok(())
}

#[cfg(not(windows))]
fn set_default_shell(_default_shell: &DefaultShell) -> Result<(), SshdConfigError> {
    Err(SshdConfigError::InvalidInput(
        t!("get.windowsOnly").to_string(),
    ))
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

fn set_sshd_config(
    cmd_info: &mut CommandInfo,
    what_if: bool,
) -> Result<Map<String, Value>, SshdConfigError> {
    // this should be its own helper function that checks that the value makes sense for the key type
    // i.e. if the key can be repeated or have multiple values, etc.
    // or if the value is something besides a string (like an object to convert back into a comma-separated list)
    let mut config_to_write = if cmd_info.purge {
        cmd_info.input.clone()
    } else {
        let mut existing_config = get_existing_config(cmd_info, what_if)?;
        for (key, value) in &cmd_info.input {
            if value.is_null() {
                existing_config.remove(key);
            } else {
                existing_config.insert(key.clone(), value.clone());
            }
        }
        existing_config
    };

    write_and_validate_config(
        &mut config_to_write,
        cmd_info.metadata.filepath.as_ref(),
        what_if,
    )?;
    Ok(config_to_write)
}

/// Write configuration to file after validation.
fn write_and_validate_config(
    config: &mut Map<String, Value>,
    filepath: Option<&PathBuf>,
    what_if: bool,
) -> Result<(), SshdConfigError> {
    debug!("{}", t!("set.writingTempConfig"));
    CanonicalProperties::remove_all(config);
    let mut config_text = SSHD_CONFIG_HEADER.to_string()
        + "\n"
        + SSHD_CONFIG_HEADER_VERSION
        + "\n"
        + SSHD_CONFIG_HEADER_WARNING
        + "\n";
    config_text.push_str(&write_config_map_to_text(config)?);

    // Write input to a temporary file and validate it with SSHD -T
    let temp_file = tempfile::Builder::new()
        .prefix("sshd_config_temp_")
        .suffix(".tmp")
        .tempfile()?;
    let temp_path = temp_file.path().to_path_buf();
    let (file, path) = temp_file.keep()?;
    debug!("{}", t!("set.tempFileCreated", path = temp_path.display()));
    std::fs::write(&temp_path, &config_text)
        .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
    drop(file);

    let args = Some(SshdCommandArgs {
        filepath: Some(temp_path),
        additional_args: None,
    });

    debug!("{}", t!("set.validatingTempConfig"));
    let result = invoke_sshd_config_validation(args);
    // Always cleanup temp file, regardless of result success or failure
    if let Err(e) = std::fs::remove_file(&path) {
        warn!(
            "{}",
            t!("set.cleanupFailed", path = path.display(), error = e)
        );
    }
    // Propagate failure, if any
    result?;

    if what_if {
        return Ok(());
    }

    let sshd_config_path = get_default_sshd_config_path(filepath.cloned())?;

    if sshd_config_path.exists() {
        let mut sshd_config_content = String::new();
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .read(true)
            .open(&sshd_config_path)
        {
            use std::io::Read;
            file.read_to_string(&mut sshd_config_content)
                .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
        } else {
            return Err(SshdConfigError::CommandError(
                t!(
                    "set.sshdConfigReadFailed",
                    path = sshd_config_path.display()
                )
                .to_string(),
            ));
        }
        if !sshd_config_content.starts_with(SSHD_CONFIG_HEADER) {
            // If config file is not already managed by this resource, create a backup of the existing file
            debug!("{}", t!("set.backingUpConfig"));
            let backup_path = format!("{}_backup", sshd_config_path.display());
            std::fs::write(&backup_path, &sshd_config_content)
                .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
            info!("{}", t!("set.backupCreated", path = backup_path));
        }
    } else {
        debug!("{}", t!("set.configDoesNotExist"));
    }

    std::fs::write(&sshd_config_path, &config_text)
        .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;

    Ok(())
}

/// Get existing config from file or return empty map if file doesn't exist.
fn get_existing_config(
    cmd_info: &CommandInfo,
    what_if: bool,
) -> Result<Map<String, Value>, SshdConfigError> {
    let mut get_cmd_info = cmd_info.clone();
    get_cmd_info.include_defaults = false;
    get_cmd_info.input = Map::new();
    if what_if {
        // In what-if (preview) mode, do not seed/copy a default sshd_config to
        // disk. If the config file does not yet exist, treat the existing
        // config as empty so no system mutation occurs.
        let config_path = get_default_sshd_config_path(get_cmd_info.metadata.filepath.clone())?;
        if !config_path.exists() {
            return Ok(Map::new());
        }
    } else {
        ensure_sshd_config_exists(get_cmd_info.metadata.filepath.clone())?;
    }
    get_sshd_settings(&get_cmd_info, false)
}
