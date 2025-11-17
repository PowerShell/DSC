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
use tracing::debug;

use crate::args::DefaultShell;
use crate::error::SshdConfigError;
use crate::util::{invoke_sshd_config_validation, SshdCmdArgs};

/// Invoke the set command.
///
/// # Errors
///
/// This function will return an error if the desired settings cannot be applied.
pub fn invoke_set(input: &str) -> Result<Map<String, Value>, SshdConfigError> {
    match serde_json::from_str::<DefaultShell>(input) {
        Ok(default_shell) => {
            debug!("default_shell: {:?}", default_shell);
            set_default_shell(default_shell.shell, default_shell.cmd_option, default_shell.escape_arguments)?;
            Ok(Map::new())
        },
        Err(_) => {
            match serde_json::from_str::<Map<String, Value>>(input) {
                Ok(sshd_config) => set_sshd_config(sshd_config),
                Err(e) => Err(SshdConfigError::InvalidInput(t!("set.failedToParseInput", error = e).to_string())),
            }
        }
    }
}

#[cfg(windows)]
fn set_default_shell(shell: String, cmd_option: Option<String>, escape_arguments: Option<bool>) -> Result<(), SshdConfigError> {
    debug!("Setting default shell");
    if !shell.is_empty() {
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

fn set_sshd_config(input: Map<String, Value>) -> Result<(), SshdConfigError> {
    // this should be its own helper function that checks that the value makes sense for the key
    debug!("Writing temporary sshd_config file");
    let mut config_text = String::new();
    for (key, value) in &input {
        if let Some(value_str) = value.as_str() {
            config_text.push_str(&format!("{} {}\n", key, value_str));
        } else {
            return Err(SshdConfigError::InvalidInput(t!("set.valueMustBeString", key = key).to_string()));
        }
    }

    // this should also be a helper function potentially
    let temp_file = tempfile::Builder::new()
        .prefix("sshd_config_temp_")
        .suffix(".tmp")
        .tempfile()?;
    let temp_path = temp_file.path().to_string_lossy().into_owned();
    let (file, path) = temp_file.keep()?;
    debug!("temporary file created at: {}", temp_path);
    std::fs::write(&temp_path, &config_text)
        .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
    drop(file);

    let args = Some(
        SshdCmdArgs {
            filepath: Some(temp_path.clone()),
            additional_args: None,
        }
    );

    debug!("Validating temporary sshd_config file");
    invoke_sshd_config_validation(args)?;

    // sshd_config path should be defined based on the system, typically at /etc/ssh/sshd_config or C:\ProgramData\ssh\sshd_config
    let sshd_config_path = if cfg!(windows) {
        "C:\\ProgramData\\ssh\\sshd_config"
    } else {
        "/etc/ssh/sshd_config"
    };
    let sshd_config_path = Path::new(sshd_config_path);

    if sshd_config_path.exists() {
        let mut sshd_config_content = String::new();
        if let Ok(mut file) = std::fs::OpenOptions::new().read(true).open(sshd_config_path) {
            use std::io::Read;
            file.read_to_string(&mut sshd_config_content)
                .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
        } else {
            return Err(SshdConfigError::CommandError(t!("set.sshdConfigReadFailed", path = sshd_config_path.display()).to_string()));
        }
        // Check if the first line contains "managed by dsc sshdconfig resource"
        if !sshd_config_content.starts_with("# managed by dsc sshdconfig resource") {
            // If not, create a backup of the existing file
            debug!("Backing up existing sshd_config file");
            let backup_path = format!("{}.bak", sshd_config_path.display());
            std::fs::write(&backup_path, &sshd_config_content)
                .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
            debug!("Backup created at: {}", backup_path);
        }
    }

    std::fs::write(sshd_config_path, &config_text)
        .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;

    if let Err(e) = std::fs::remove_file(&path) {
        debug!("Failed to clean up temporary file {}: {}", path.display(), e);
    }

    Ok(())
}
