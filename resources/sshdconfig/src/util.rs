// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{path::PathBuf, process::Command};
use tracing::{debug, warn, Level};
use tracing_subscriber::{EnvFilter, Layer, prelude::__tracing_subscriber_SubscriberExt};

use crate::args::{TraceFormat, TraceLevel};
use crate::error::SshdConfigError;
use crate::inputs::{CommandInfo, Metadata, SshdCommandArgs};
use crate::metadata::{MULTI_ARG_KEYWORDS_COMMA_SEP, SSHD_CONFIG_DEFAULT_PATH_UNIX, SSHD_CONFIG_DEFAULT_PATH_WINDOWS};
use crate::parser::parse_text_to_map;

#[derive(Debug, Deserialize)]
struct MatchBlockFmt {
    criteria: Map<String, Value>,
    #[serde(flatten)]
    contents: Map<String, Value>,
}

/// Enable tracing.
///
/// # Arguments
///
/// * `trace_level` - The level of information to output
/// * `trace_format` - The format of the output
///
/// # Errors
///
/// This function will return an error if it fails to initialize tracing.
pub fn enable_tracing(trace_level: Option<&TraceLevel>, trace_format: &TraceFormat) {
    let trace_level = match trace_level {
        Some(trace_level) => trace_level,
        None => {
            if let Ok(trace_level) = std::env::var("DSC_TRACE_LEVEL") {
                &match trace_level.to_lowercase().as_str() {
                    "error" => TraceLevel::Error,
                    "warn" => TraceLevel::Warn,
                    "info" => TraceLevel::Info,
                    "debug" => TraceLevel::Debug,
                    "trace" => TraceLevel::Trace,
                    _ => {
                        eprintln!("{}: {trace_level}", t!("main.invalidTraceLevel"));
                        TraceLevel::Info
                    }
                }
            } else {
                &TraceLevel::Info
            }
        }
    };

    let tracing_level = match trace_level {
        TraceLevel::Error => Level::ERROR,
        TraceLevel::Warn => Level::WARN,
        TraceLevel::Info => Level::INFO,
        TraceLevel::Debug => Level::DEBUG,
        TraceLevel::Trace => Level::TRACE,
    };

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("warn"))
        .unwrap_or_default()
        .add_directive(tracing_level.into());
    let layer = tracing_subscriber::fmt::Layer::default().with_writer(std::io::stderr);
    let fmt = match trace_format {
        TraceFormat::Default => {
            layer
                .with_ansi(true)
                .with_level(true)
                .with_line_number(true)
                .boxed()
        },
        TraceFormat::Plaintext => {
            layer
                .with_ansi(false)
                .with_level(true)
                .with_line_number(false)
                .boxed()
        },
        TraceFormat::Json => {
            layer
                .with_ansi(false)
                .with_level(true)
                .with_line_number(true)
                .json()
                .boxed()
        }
    };

    let subscriber = tracing_subscriber::Registry::default().with(fmt).with(filter);

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        eprintln!("{}", t!("util.tracingInitError"));
    }
}

/// Format a JSON value for writing to `sshd_config`.
///
/// # Arguments
///
/// * `key` - The configuration key name (used to determine formatting rules)
/// * `value` - The JSON value to format
///
/// # Returns
///
/// * `Ok(Some(String))` - Formatted value string
/// * `Ok(None)` - Value is null and should be skipped
/// * `Err(SshdConfigError)` - Invalid value type or formatting error
///
/// # Errors
///
/// Returns an error if the value type is not supported or if formatting fails.
pub fn format_sshd_value(key: &str, value: &Value) -> Result<String, SshdConfigError> {
    let key_lower = key.to_lowercase();
    let result = if key_lower == "match" {
        format_match_block(value)?
    } else {
        format_value_as_string(value, MULTI_ARG_KEYWORDS_COMMA_SEP.contains(&key_lower.as_str()))?
    };

    if result.is_empty() {
        return Err(SshdConfigError::ParserError(t!("util.invalidValue", key = key).to_string()))
    }
    Ok(result)
}

fn format_value_as_string(value: &Value, is_comma_separated: bool) -> Result<String, SshdConfigError> {
    match value {
        Value::Array(arr) => {
            if arr.is_empty() {
                return Ok(String::new());
            }

            // Convert array elements to strings
            let mut string_values = Vec::new();
            for item in arr {
                let result = format_value_as_string(item, false)?;
                if !result.is_empty() {
                    string_values.push(result);
                }
            }

            if string_values.is_empty() {
                return Ok(String::new());
            }

            let separator = if is_comma_separated {
                ","
            } else {
                " "
            };

            Ok(string_values.join(separator))
        },
        Value::Bool(b) => {
            let bool_str = if *b { "yes" } else { "no" };
            Ok(bool_str.to_string())
        },
        Value::Number(n) => Ok(n.to_string()),
        Value::String(s) => Ok(s.clone()),
        _ => Ok(String::new())
    }
}

/// Get the `sshd_config` path
/// Uses the input value, if provided.
/// If input value not provided, get default path for the OS.
/// On Windows, uses the `ProgramData` environment variable and standard path.
/// On Unix-like systems, uses the standard path.
pub fn get_default_sshd_config_path(input: Option<PathBuf>) -> Result<PathBuf, SshdConfigError> {
    if let Some(path) = input {
        Ok(path)
    } else if cfg!(windows) {
        let program_data = std::env::var("ProgramData")?;
        Ok(PathBuf::from(format!("{program_data}{SSHD_CONFIG_DEFAULT_PATH_WINDOWS}")))
    } else {
        Ok(PathBuf::from(SSHD_CONFIG_DEFAULT_PATH_UNIX))
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
            command.arg("-f").arg(&filepath);
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

    // on Windows, sshd cannot read from the file if it is still open
    let temp_path = temp_file.path().to_path_buf();
    // do not automatically delete the file when it goes out of scope
    let (file, path) = temp_file.keep()?;
    // close the file handle to allow sshd to read it
    drop(file);

    debug!("{}", t!("util.tempFileCreated", path = temp_path.display()));
    let args = Some(
        SshdCommandArgs {
            filepath: Some(temp_path),
            additional_args: None,
        }
    );

    // Clean up the temporary file regardless of success or failure
    let output = invoke_sshd_config_validation(args);
    if let Err(e) = std::fs::remove_file(&path) {
        debug!("{}", t!("util.cleanupFailed", path = path.display(), error = e));
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
        let clobber = get_bool_or_default(&mut sshd_config, "_clobber", false)?;
        let include_defaults = get_bool_or_default(&mut sshd_config, "_includeDefaults", is_get)?;
        let metadata: Metadata = if let Some(value) = sshd_config.remove("_metadata") {
            serde_json::from_value(value)?
        } else {
            Metadata::new()
        };
        // lowercase keys for case-insensitive comparison later of SSHD -T output
        sshd_config = sshd_config.into_iter().map(|(k, v)| (k.to_lowercase(), v)).collect();
        let sshd_args = metadata.filepath.clone().map(|filepath| {
            SshdCommandArgs {
                filepath: Some(filepath),
                additional_args: None,
            }
        });
        if is_get && !sshd_config.is_empty() {
            warn!("{}", t!("util.getIgnoresInputFilters"));
        }
        return Ok(CommandInfo {
            clobber,
            include_defaults,
            input: sshd_config,
            metadata,
            sshd_args
        })
    }
    Ok(CommandInfo::new(is_get))
}

/// Reads `sshd_config` file.
///
/// # Arguments
///
/// * `input` - Optional `PathBuf` with `sshd_config` filepath.
///
/// # Errors
///
/// This function will return an error if the file cannot be found or read.
pub fn read_sshd_config(input: Option<PathBuf>) -> Result<String, SshdConfigError> {
    let filepath = get_default_sshd_config_path(input)?;

    if filepath.exists() {
        let mut sshd_config_content = String::new();
        if let Ok(mut file) = std::fs::OpenOptions::new().read(true).open(&filepath) {
            use std::io::Read;
            file.read_to_string(&mut sshd_config_content)
                .map_err(|e| SshdConfigError::CommandError(e.to_string()))?;
        } else {
            return Err(SshdConfigError::CommandError(t!("util.sshdConfigReadFailed", path = filepath.display()).to_string()));
        }
        Ok(sshd_config_content)
    } else {
        Err(SshdConfigError::CommandError(t!("util.sshdConfigNotFound", path = filepath.display()).to_string()))
    }
}

/// Helper function to extract a boolean value from a map, or return a default value.
///
/// # Arguments
///
/// * `map` - The map to extract the value from
/// * `key` - The key to look for in the map
/// * `default` - The default value to return if the key is not found
///
/// # Errors
///
/// Returns an error if the value exists but is not a boolean.
fn get_bool_or_default(map: &mut Map<String, Value>, key: &str, default: bool) -> Result<bool, SshdConfigError> {
    if let Some(value) = map.remove(key) {
        if let Value::Bool(b) = value {
            Ok(b)
        } else {
            Err(SshdConfigError::InvalidInput(t!("util.inputMustBeBoolean", input = key).to_string()))
        }
    } else {
        Ok(default)
    }
}

fn format_match_block(match_obj: &Value) -> Result<String, SshdConfigError> {
    let match_block = match serde_json::from_value::<MatchBlockFmt>(match_obj.clone()) {
        Ok(result) => {
            result
        }
        Err(e) => {
            return Err(SshdConfigError::ParserError(t!("util.deserializeFailed", error = e).to_string()));
        }
    };

    if match_block.criteria.is_empty() {
        return Err(SshdConfigError::InvalidInput(
            t!("util.matchBlockMissingCriteria").to_string()
        ));
    }

    let mut match_parts = vec![];
    let mut result = vec![];

    for (key, value) in &match_block.criteria {
        // all match criteria values are comma-separated
        let value_formatted = format_value_as_string(value, true)?;
        match_parts.push(format!("{key} {value_formatted}"));
    }

    // Write the Match line with the formatted criteria(s)
    result.push(match_parts.join(" "));

    // Format other keywords in the match block
    for (key, value) in &match_block.contents {
        let formatted_value = format_sshd_value(key, value)?;
        result.push(format!("    {key} {formatted_value}"));
    }

    Ok(result.join("\n"))
}
