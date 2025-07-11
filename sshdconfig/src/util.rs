// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::process::Command;
use tracing::debug;
use tracing_subscriber::{EnvFilter, filter::LevelFilter, Layer, prelude::__tracing_subscriber_SubscriberExt};

use crate::error::SshdConfigError;
use crate::parser::parse_text_to_map;

// create a struct for sshdconfig arguments
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SshdCmdArgs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filepath: Option<String>,
    #[serde(rename = "additionalArgs", skip_serializing_if = "Option::is_none")]
    pub additional_args: Option<Vec<String>>,
}

/// Enable tracing.
///
/// # Errors
///
/// This function will return an error if it fails to initialize tracing.
pub fn enable_tracing() {
    // default filter to trace level
    let filter = EnvFilter::builder().with_default_directive(LevelFilter::TRACE.into()).parse("").unwrap_or_default();
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
pub fn invoke_sshd_config_validation(args: Option<SshdCmdArgs>) -> Result<String, SshdConfigError> {
    let sshd_command = if cfg!(target_os = "windows") {
        "sshd.exe"
    } else {
        "sshd"
    };

    let mut command = Command::new(sshd_command);
    command.arg("-T");

    if let Some(args) = args {
        if let Some(filepath) = args.filepath {
            command.arg("-f").arg(filepath);
        }
        if let Some(additional_args) = args.additional_args {
            command.args(additional_args);
        }
    }

    let output = command.output()
        .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
        Ok(stdout)
    } else {
        let stderr = String::from_utf8(output.stderr)
            .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
        if stderr.contains("sshd: no hostkeys available") || stderr.contains("Permission denied") {
            return Err(SshdConfigError::CommandError(
                t!("util.sshdElevation").to_string()
            ));
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

    let temp_path = temp_file.path().to_string_lossy().into_owned();
    let (file, path) = temp_file.keep()?;

    // close file so another process (sshd) can read it
    drop(file);

    debug!("temporary file created at: {}", temp_path);
    let args = Some(
        SshdCmdArgs {
            filepath: Some(temp_path.clone()),
            additional_args: None,
        }
    );

    let output = invoke_sshd_config_validation(args);

    if let Err(e) = std::fs::remove_file(&path) {
        debug!("Failed to clean up temporary file {}: {}", path.display(), e);
    }

    let output = output?;
    let sshd_config: Map<String, Value> = parse_text_to_map(&output)?;
    Ok(sshd_config)
}
