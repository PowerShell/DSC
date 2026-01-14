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
use std::string::String;
use tracing::{debug, info, warn};

use crate::args::{DefaultShell, Setting};
use crate::error::SshdConfigError;
use crate::formatter::write_config_map_to_text;
use crate::get::get_sshd_settings;
use crate::inputs::{CommandInfo, SshdCommandArgs};
use crate::metadata::{MULTI_ARG_KEYWORDS_COMMA_SEP, MULTI_ARG_KEYWORDS_SPACE_SEP, REPEATABLE_KEYWORDS, SSHD_CONFIG_HEADER, SSHD_CONFIG_HEADER_VERSION, SSHD_CONFIG_HEADER_WARNING};
use crate::util::{build_command_info, get_default_sshd_config_path, invoke_sshd_config_validation};

/// Invoke the set command.
///
/// # Errors
///
/// This function will return an error if the desired settings cannot be applied.
pub fn invoke_set(input: &str, setting: &Setting) -> Result<Map<String, Value>, SshdConfigError> {
    match setting {
        Setting::SshdConfig => {
            debug!("{} {:?}", t!("set.settingSshdConfig").to_string(), setting);
            let mut cmd_info = build_command_info(Some(&input.to_string()), false)?;
            match set_sshd_config(&mut cmd_info) {
                Ok(()) => Ok(Map::new()),
                Err(e) => Err(e),
            }
        },
        Setting::WindowsGlobal => {
            debug!("{} {:?}", t!("set.settingDefaultShell").to_string(), setting);
            match serde_json::from_str::<DefaultShell>(input) {
                Ok(default_shell) => {
                    debug!("{}", t!("set.defaultShellDebug", shell = format!("{:?}", default_shell)));
                    // if default_shell.shell is Some, we should pass that into set default shell
                    // otherwise pass in an empty string
                    let shell: String = default_shell.shell.clone().unwrap_or_default();
                    set_default_shell(shell, default_shell.cmd_option, default_shell.escape_arguments)?;
                    Ok(Map::new())
                },
                Err(e) => Err(SshdConfigError::InvalidInput(t!("set.failedToParseDefaultShell", error = e).to_string())),
            }
        }
    }
}

#[cfg(windows)]
fn set_default_shell(shell: String, cmd_option: Option<String>, escape_arguments: Option<bool>) -> Result<(), SshdConfigError> {
    debug!("{}", t!("set.settingDefaultShell"));
    if shell.is_empty() {
        remove_registry(DEFAULT_SHELL)?;
    } else {
        // TODO: if shell contains quotes, we need to remove them
        let shell_path = Path::new(&shell);
        if shell_path.is_relative() && shell_path.components().any(|c| c == std::path::Component::ParentDir) {
            return Err(SshdConfigError::InvalidInput(t!("set.shellPathMustNotBeRelative").to_string()));
        }
        if !shell_path.exists() {
            return Err(SshdConfigError::InvalidInput(t!("set.shellPathDoesNotExist", shell = shell).to_string()));
        }
        set_registry(DEFAULT_SHELL, RegistryValueData::String(shell))?;
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
fn set_default_shell(_shell: String, _cmd_option: Option<String>, _escape_arguments: Option<bool>) -> Result<(), SshdConfigError> {
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

fn set_sshd_config(cmd_info: &mut CommandInfo) -> Result<(), SshdConfigError> {
    // this should be its own helper function that checks that the value makes sense for the key type
    // i.e. if the key can be repeated or have multiple values, etc.
    // or if the value is something besides a string (like an object to convert back into a comma-separated list)
    debug!("{}", t!("set.writingTempConfig"));
    let mut config_text = SSHD_CONFIG_HEADER.to_string() + "\n" + SSHD_CONFIG_HEADER_VERSION + "\n" + SSHD_CONFIG_HEADER_WARNING + "\n";
    if cmd_info.purge {
        config_text.push_str(&write_config_map_to_text(&cmd_info.input)?);
    } else {
        let mut get_cmd_info = cmd_info.clone();
        get_cmd_info.include_defaults = false;
        get_cmd_info.input = Map::new();

        let mut existing_config = match get_sshd_settings(&get_cmd_info, true) {
            Ok(config) => config,
            Err(SshdConfigError::FileNotFound(_)) => {
                return Err(SshdConfigError::InvalidInput(
                    t!("set.purgeFalseRequiresExistingFile").to_string()
                ));
            }
            Err(e) => return Err(e),
        };
        for (key, value) in &cmd_info.input {
            let key_contains = key.as_str();

            // TODO: remove when design for handling repeatable and multi-arg keywords is finalized
            // and consider using SshdConfigValue instead of any remaining contains() checks
            if REPEATABLE_KEYWORDS.contains(&key_contains)
                || MULTI_ARG_KEYWORDS_COMMA_SEP.contains(&key_contains)
                || MULTI_ARG_KEYWORDS_SPACE_SEP.contains(&key_contains) {
                return Err(SshdConfigError::InvalidInput(t!("set.purgeFalseUnsupported").to_string()));
            }

            if value.is_null() {
                existing_config.remove(key);
            } else {
                existing_config.insert(key.clone(), value.clone());
            }
        }
        existing_config.remove("_metadata");
        existing_config.remove("_inheritedDefaults");
        config_text.push_str(&write_config_map_to_text(&existing_config)?);
    }

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

    let args = Some(
        SshdCommandArgs {
            filepath: Some(temp_path),
            additional_args: None,
        }
    );

    debug!("{}", t!("set.validatingTempConfig"));
    let result = invoke_sshd_config_validation(args);
    // Always cleanup temp file, regardless of result success or failure
    if let Err(e) = std::fs::remove_file(&path) {
        warn!("{}", t!("set.cleanupFailed", path = path.display(), error = e));
    }
    // Propagate failure, if any
    result?;

    let sshd_config_path = get_default_sshd_config_path(cmd_info.metadata.filepath.clone())?;

    if sshd_config_path.exists() {
        let mut sshd_config_content = String::new();
        if let Ok(mut file) = std::fs::OpenOptions::new().read(true).open(&sshd_config_path) {
            use std::io::Read;
            file.read_to_string(&mut sshd_config_content)
                .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
        } else {
            return Err(SshdConfigError::CommandError(t!("set.sshdConfigReadFailed", path = sshd_config_path.display()).to_string()));
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
