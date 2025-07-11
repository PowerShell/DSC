// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use std::process::Command;
use tracing_subscriber::{EnvFilter, filter::LevelFilter, Layer, prelude::__tracing_subscriber_SubscriberExt};

use crate::error::SshdConfigError;


/// Enable tracing.
///
/// # Errors
///
/// This function will return an error if it fails to initialize tracing.
pub fn enable_tracing() {
    // default filter to trace level
    let filter = EnvFilter::builder().with_default_directive(LevelFilter::TRACE.into()).from_env();
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
pub fn invoke_sshd_config_validation() -> Result<String, SshdConfigError> {
    let sshd_command = if cfg!(target_os = "windows") {
        "sshd.exe"
    } else {
        "sshd"
    };

    let output = Command::new(sshd_command)
        .arg("-T")
        .output()
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
