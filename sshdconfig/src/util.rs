// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde_json::{Map, Value};
use std::{path::Path, process::Command};
use tracing::debug;
use tracing_subscriber::{filter::LevelFilter, prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Layer};

use crate::error::SshdConfigError;
use crate::inputs::{CommandInfo, Metadata, SshdCommandArgs};
use crate::parser::parse_text_to_map;

/// Enable tracing.
///
/// # Errors
///
/// This function will return an error if it fails to initialize tracing.
pub fn enable_tracing() {
    // default filter to trace level
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::TRACE.into())
        .parse("")
        .unwrap_or_default();
    let layer = tracing_subscriber::fmt::Layer::default().with_writer(std::io::stderr);
    let fmt = layer
        .with_ansi(false)
        .with_level(true)
        .with_line_number(true)
        .json()
        .boxed();

    let subscriber = tracing_subscriber::Registry::default().with(fmt).with(filter);

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        eprintln!("{}", t!("util.tracingInitError"));
    }
}

/// Invoke sshd -T.
///
/// # Errors
///
/// This function will return an error if sshd -T fails to validate `sshd_config`.
pub fn invoke_sshd_config_validation(args: Option<SshdCommandArgs>) -> Result<String, SshdConfigError> {
    let mut command = Command::new("sshd");
    command.arg("-T");

    if let Some(args) = args {
        if let Some(filepath) = args.filepath {
            command.arg("-f").arg(filepath);
        }
        if let Some(additional_args) = args.additional_args {
            command.args(additional_args);
        }
    }

    let output = command
        .output()
        .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
        Ok(stdout)
    } else {
        let stderr = String::from_utf8(output.stderr).map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
        if stderr.contains("sshd: no hostkeys available") || stderr.contains("Permission denied") {
            return Err(SshdConfigError::CommandError(t!("util.sshdElevation").to_string()));
        }
        Err(SshdConfigError::CommandError(stderr))
    }
}

/// Extract SSH server defaults by running sshd -T with an empty configuration file.
///
/// # Errors
///
/// This function will return an error if it fails to extract the defaults from sshd.
pub fn extract_sshd_defaults() -> Result<Map<String, Value>, SshdConfigError> {
    let temp_file = tempfile::Builder::new()
        .prefix("sshd_config_empty_")
        .suffix(".tmp")
        .tempfile()?;

    // on Windows, sshd cannot read from the file if it is still open
    let temp_path = temp_file.path().to_string_lossy().into_owned();
    // do not automatically delete the file when it goes out of scope
    let (file, path) = temp_file.keep()?;
    // close the file handle to allow sshd to read it
    drop(file);

    debug!("temporary file created at: {}", temp_path);
    let args = Some(SshdCommandArgs {
        filepath: Some(temp_path.clone()),
        additional_args: None,
    });

    // Clean up the temporary file regardless of success or failure
    let output = invoke_sshd_config_validation(args);
    if let Err(e) = std::fs::remove_file(&path) {
        debug!("Failed to clean up temporary file {}: {}", path.display(), e);
    }
    let result = output?;
    let sshd_config: Map<String, Value> = parse_text_to_map(&result)?;
    Ok(sshd_config)
}

/// Extract _metadata field from the input string, if it can be parsed as JSON.
///
/// # Errors
///
/// This function will return an error if it fails to parse the input string and if the _metadata field exists, extract it.
pub fn build_command_info(input: Option<&String>, is_get: bool) -> Result<CommandInfo, SshdConfigError> {
    if let Some(inputs) = input {
        let mut sshd_config: Map<String, Value> = serde_json::from_str(inputs.as_str())?;
        let metadata: Metadata = if let Some(value) = sshd_config.remove("_metadata") {
            serde_json::from_value(value)?
        } else {
            Metadata::new()
        };
        let sshd_args = metadata.filepath.as_ref().map(|filepath| SshdCommandArgs {
            filepath: Some(filepath.clone()),
            additional_args: None,
        });
        let include_defaults: bool = if let Some(value) = sshd_config.remove("_includeDefaults") {
            if let Value::Bool(b) = value {
                b
            } else {
                return Err(SshdConfigError::InvalidInput(
                    t!("util.includeDefaultsMustBeBoolean").to_string(),
                ));
            }
        } else {
            is_get
        };
        if is_get && !sshd_config.is_empty() {
            return Err(SshdConfigError::InvalidInput(t!("util.inputMustBeEmpty").to_string()));
        }
        return Ok(CommandInfo {
            include_defaults,
            input: sshd_config,
            metadata,
            sshd_args,
        });
    }
    Ok(CommandInfo::new(is_get))
}

/// Reads `sshd_config` file.
///
/// # Arguments
///
/// * `input` - Optional string with `sshd_config` filepath.
///
/// # Errors
///
/// This function will return an error if the file cannot be found or read.
pub fn read_sshd_config(input: Option<String>) -> Result<String, SshdConfigError> {
    let sshd_config_path = if let Some(input) = input {
        input
    } else if cfg!(windows) {
        let program_data = std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".into());
        format!("{program_data}\\ssh\\sshd_config")
    } else {
        "/etc/ssh/sshd_config".to_string()
    };
    let filepath = Path::new(&sshd_config_path);

    if filepath.exists() {
        let mut sshd_config_content = String::new();
        if let Ok(mut file) = std::fs::OpenOptions::new().read(true).open(filepath) {
            use std::io::Read;
            file.read_to_string(&mut sshd_config_content)
                .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
        } else {
            return Err(SshdConfigError::CommandError(
                t!("util.sshdConfigReadFailed", path = filepath.display()).to_string(),
            ));
        }
        Ok(sshd_config_content)
    } else {
        Err(SshdConfigError::CommandError(
            t!("util.sshdConfigNotFound", path = filepath.display()).to_string(),
        ))
    }
}
