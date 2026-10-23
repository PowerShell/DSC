// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Helpers shared by the command: exit codes, tracing setup, input reading, and output writing.
//!
//! These mirror the equivalents in the `dsc` crate so that `dsclint` behaves the same way as
//! `dsc` for settings, tracing, input, and output.

use crate::args::{OutputFormat, TraceFormat};
use dsc_lib::{
    discovery::Discovery,
    dscerror::DscError,
    dscresources::command_resource::TraceLevel,
    extensions::dscextension::Capability,
    types::SemanticVersion,
    util::get_setting,
};
use dsc_lib_telemetry::{StderrFormat, basic::BasicTracingOptions};
use rust_i18n::t;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::env;
use std::io::{IsTerminal, Read, Write, stdout};
use std::path::Path;
use std::process::ExitCode;
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{LinesWithEndings, as_24_bit_terminal_escaped},
};
use tracing::{Level, debug, error, info, trace, warn};

// The exit codes match the ones `dsc` uses for the same conditions.
pub const EXIT_SUCCESS: u8 = 0;
pub const EXIT_INVALID_ARGS: u8 = 1;
pub const EXIT_DSC_ERROR: u8 = 2;
pub const EXIT_JSON_ERROR: u8 = 3;
pub const EXIT_INVALID_INPUT: u8 = 4;
pub const EXIT_VALIDATION_FAILED: u8 = 5;

pub const DSC_TRACE_LEVEL: &str = "DSC_TRACE_LEVEL";

#[derive(Deserialize)]
pub struct TracingSetting {
    /// Trace level to use - see pub enum `TraceLevel` in `dsc_lib\src\dscresources\command_resource.rs`
    level: TraceLevel,
    /// Trace format to use - see pub enum `TraceFormat` in `dsclint\src\args.rs`
    format: TraceFormat,
    /// Whether the 'level' can be overridden by `DSC_TRACE_LEVEL` environment variable
    #[serde(rename = "allowOverride")]
    allow_override: bool,
}

impl Default for TracingSetting {
    fn default() -> TracingSetting {
        TracingSetting {
            level: TraceLevel::Warn,
            format: TraceFormat::Default,
            allow_override: true,
        }
    }
}

/// Returns the version of `dsclint` as a semantic version.
///
/// The crate version is kept in lockstep with `dsc`, so this is the DSC version that
/// `directives.version` requirements are checked against.
///
/// # Panics
///
/// Panics if the version defined in the cargo manifest isn't a valid semantic version. Cargo
/// rejects such a manifest at build time, so this can't happen for a built binary.
#[must_use]
pub fn current_version() -> SemanticVersion {
    match SemanticVersion::parse(env!("CARGO_PKG_VERSION")) {
        Ok(version) => version,
        Err(err) => unreachable!("{}", t!("util.invalidVersion", error = err)),
    }
}

/// Write the JSON object to the console
///
/// # Arguments
///
/// * `json` - The JSON to write
/// * `format` - The format to use
///
/// # Errors
///
/// Returns `EXIT_JSON_ERROR` when the JSON can't be parsed or converted to the requested format.
pub fn write_object(json: &str, format: Option<&OutputFormat>) -> Result<(), ExitCode> {
    let mut is_json = true;
    let mut output_format = format;
    let mut syntax_color = false;
    if stdout().is_terminal() {
        syntax_color = true;
        if output_format.is_none() {
            output_format = Some(&OutputFormat::Yaml);
        }
    } else if output_format.is_none() {
        output_format = Some(&OutputFormat::Json);
    }

    let output = match output_format {
        Some(OutputFormat::Json) => json.to_string(),
        Some(OutputFormat::PrettyJson) => {
            let value: Value = match serde_json::from_str(json) {
                Ok(value) => value,
                Err(err) => {
                    error!("JSON: {err}");
                    return Err(ExitCode::from(EXIT_JSON_ERROR));
                }
            };
            match serde_json::to_string_pretty(&value) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON: {err}");
                    return Err(ExitCode::from(EXIT_JSON_ERROR));
                }
            }
        },
        Some(OutputFormat::Yaml) | None => {
            is_json = false;
            let value: Value = match serde_json::from_str(json) {
                Ok(value) => value,
                Err(err) => {
                    error!("JSON: {err}");
                    return Err(ExitCode::from(EXIT_JSON_ERROR));
                }
            };
            match serde_yaml::to_string(&value) {
                Ok(yaml) => yaml,
                Err(err) => {
                    error!("YAML: {err}");
                    return Err(ExitCode::from(EXIT_JSON_ERROR));
                }
            }
        }
    };

    if syntax_color {
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let Some(syntax) = (if is_json {
            ps.find_syntax_by_extension("json")
        } else {
            ps.find_syntax_by_extension("yaml")
        }) else {
            println!("{json}");
            return Ok(());
        };

        let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

        for line in LinesWithEndings::from(&output) {
            let Ok(ranges) = h.highlight_line(line, &ps) else {
                continue;
            };
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            print!("{escaped}");
        }
    } else {
        let mut stdout_lock = stdout().lock();
        if writeln!(stdout_lock, "{output}").is_err() {
            // likely caused by a broken pipe (e.g. 'head' command closed early)
            return Ok(());
        }
    }

    Ok(())
}

/// Enable tracing the same way `dsc` does: the `tracing` setting or policy from the settings
/// file next to the executable, overridden by `DSC_TRACE_LEVEL` when allowed, then by the
/// command-line arguments when no policy is in force.
///
/// # Arguments
///
/// * `trace_level_arg` - The trace level from the command line, if given.
/// * `trace_format_arg` - The trace format from the command line, if given.
pub fn enable_tracing(trace_level_arg: Option<&TraceLevel>, trace_format_arg: Option<&TraceFormat>) {
    let mut policy_is_used = false;
    let mut tracing_setting = TracingSetting::default();

    let default_guard = BasicTracingOptions::init_default_guard();

    // read setting/policy from files
    if let Ok(v) = get_setting("tracing") {
        if v.policy != Value::Null {
            match serde_json::from_value::<TracingSetting>(v.policy) {
                Ok(v) => {
                    tracing_setting = v;
                    policy_is_used = true;
                },
                Err(e) => { error!("{e}"); }
            }
        } else if v.setting != Value::Null {
            match serde_json::from_value::<TracingSetting>(v.setting) {
                Ok(v) => {
                    tracing_setting = v;
                },
                Err(e) => { error!("{e}"); }
            }
        }
    } else {
        error!("{}", t!("util.failedToReadTracingSetting"));
    }

    // override with DSC_TRACE_LEVEL env var if permitted
    if tracing_setting.allow_override && let Ok(level) = env::var(DSC_TRACE_LEVEL) {
        tracing_setting.level = match level.to_ascii_uppercase().as_str() {
            "ERROR" => TraceLevel::Error,
            "WARN" => TraceLevel::Warn,
            "INFO" => TraceLevel::Info,
            "DEBUG" => TraceLevel::Debug,
            "TRACE" => TraceLevel::Trace,
            _ => {
                warn!("{}: '{level}'", t!("util.invalidTraceLevel"));
                TraceLevel::Warn
            }
        }
    }

    // command-line args override setting value, but not policy
    if !policy_is_used {
        if let Some(v) = trace_level_arg {
            tracing_setting.level = v.clone();
        }
        if let Some(v) = trace_format_arg {
            tracing_setting.format = v.clone();
        }
    }

    // convert to 'tracing' crate type
    let tracing_level = match tracing_setting.level {
        TraceLevel::Error => Level::ERROR,
        TraceLevel::Warn => Level::WARN,
        TraceLevel::Info => Level::INFO,
        TraceLevel::Debug => Level::DEBUG,
        TraceLevel::Trace => Level::TRACE,
    };
    // convert to 'dsc-lib-telemetry' crate type
    let stderr_format = match tracing_setting.format {
        TraceFormat::Default => StderrFormat::Default,
        TraceFormat::Plaintext => StderrFormat::Plaintext,
        TraceFormat::Json | TraceFormat::PassThrough => StderrFormat::Json,
    };

    // enable tracing
    let subscriber = BasicTracingOptions {
        tracing_level,
        stderr_format
    }.init_subscriber();

    drop(default_guard);
    if tracing::subscriber::set_global_default(subscriber).is_err() {
        eprintln!("{}", t!("util.failedToSetTracing"));
    }

    // set DSC_TRACE_LEVEL for child processes, like the resources run during discovery
    unsafe {
        env::set_var(DSC_TRACE_LEVEL, tracing_level.to_string().to_ascii_lowercase());
    }
    info!("Trace-level is {:?}", tracing_setting.level);
}

/// Read an input document as text without parsing it.
///
/// # Arguments
///
/// * `input` - The inline document, if provided.
/// * `file` - The path to the document, or `-` to read it from STDIN. Extensions with the
///   `import` capability get the first chance to handle a file path.
///
/// # Returns
///
/// The document text, or an empty string when neither argument is provided.
///
/// # Errors
///
/// Returns `EXIT_INVALID_INPUT` when the inline input is actually a path to an existing file, the
/// file can't be read, STDIN isn't valid UTF-8, or the document is empty.
pub fn read_input(input: Option<&String>, file: Option<&String>) -> Result<String, ExitCode> {
    trace!("Input: {input:?}, File: {file:?}");
    let value = if let Some(input) = input {
        debug!("{}", t!("util.readingInput"));

        // see if user accidentally passed in a file path
        if Path::new(input).exists() {
            error!("{}", t!("util.inputIsFile"));
            return Err(ExitCode::from(EXIT_INVALID_INPUT));
        }
        input.clone()
    } else if let Some(path) = file {
        debug!("{} {path}", t!("util.readingInputFromFile"));
        // check if need to read from STDIN
        if path == "-" {
            info!("{}", t!("util.readingInputFromStdin"));
            let mut stdin = Vec::<u8>::new();
            match std::io::stdin().read_to_end(&mut stdin) {
                Ok(_) => {
                    match String::from_utf8(stdin) {
                        Ok(input) => {
                            input
                        },
                        Err(err) => {
                            error!("{}: {err}", t!("util.invalidUtf8"));
                            return Err(ExitCode::from(EXIT_INVALID_INPUT));
                        }
                    }
                },
                Err(err) => {
                    error!("{}: {err}", t!("util.failedToReadStdin"));
                    return Err(ExitCode::from(EXIT_INVALID_INPUT));
                }
            }
        } else {
            // see if an extension should handle this file
            let mut discovery = Discovery::new();
            let path_buf = Path::new(path);
            for extension in discovery.get_extensions(&Capability::Import) {
                if let Ok(content) = extension.import(path_buf) {
                    return Ok(content);
                }
            }
            match std::fs::read_to_string(path) {
                Ok(input) => {
                    // check if it contains UTF-8 BOM and remove it
                    if input.as_bytes().starts_with(&[0xEF, 0xBB, 0xBF]) {
                        info!("{}", t!("util.removingUtf8Bom"));
                        input[3..].to_string()
                    } else {
                        input
                    }
                },
                Err(err) => {
                    error!("{}: {err}", t!("util.failedToReadFile"));
                    return Err(ExitCode::from(EXIT_INVALID_INPUT));
                }
            }
        }
    } else {
        debug!("{}", t!("util.noInput"));
        return Ok(String::new());
    };

    if value.trim().is_empty() {
        error!("{}", t!("util.emptyInput"));
        return Err(ExitCode::from(EXIT_INVALID_INPUT));
    }

    Ok(value)
}

/// Parse parameter input as JSON, falling back to YAML.
///
/// # Arguments
///
/// * `input` - The parameters as JSON or YAML text.
/// * `context` - Which input is being parsed, for the error message: `FileParameters` or
///   `InlineParameters`.
///
/// # Errors
///
/// Returns an error if the input cannot be parsed as valid JSON or YAML.
pub fn parse_parameters(input: &str, context: &str) -> Result<Value, DscError> {
    match serde_json::from_str(input) {
        Ok(json) => Ok(json),
        Err(_) => {
            match serde_yaml::from_str::<serde_yaml::Value>(input) {
                Ok(yaml) => Ok(serde_json::to_value(yaml)?),
                Err(err) => {
                    Err(DscError::Parser(t!(&format!("util.failedToParse{context}"), error = err.to_string()).to_string()))
                }
            }
        }
    }
}

/// Convert parameter input to a map.
///
/// # Arguments
///
/// * `params` - Parameter string to convert (JSON or YAML format)
/// * `context` - Context string for error messages
///
/// # Errors
///
/// Returns an error if the input cannot be parsed or is not an object.
fn params_to_map(params: &str, context: &str) -> Result<Map<String, Value>, DscError> {
    let value = parse_parameters(params, context)?;

    let Some(map) = value.as_object().cloned() else {
        return Err(DscError::Parser(t!("util.parametersNotObject").to_string()));
    };

    Ok(map)
}

/// Merge two parameter sets, with inline parameters taking precedence over file parameters.
/// Top-level keys (like "parameters") are merged recursively, but parameter values themselves
/// are replaced (not merged) when specified inline.
///
/// # Arguments
///
/// * `file_params` - Parameters from file (JSON or YAML format)
/// * `inline_params` - Inline parameters (JSON or YAML format) that take precedence
///
/// # Returns
///
/// The merged parameters.
///
/// # Errors
///
/// Returns an error if either parameter set cannot be parsed as valid JSON or YAML or isn't an
/// object.
pub fn merge_parameters(file_params: &str, inline_params: &str) -> Result<Value, DscError> {
    let mut file_map = params_to_map(file_params, "FileParameters")?;
    let inline_map = params_to_map(inline_params, "InlineParameters")?;

    for (key, inline_value) in &inline_map {
        if key == "parameters" {
            // Special handling for "parameters" key - merge at parameter name level only.
            // Within each parameter name, inline replaces (not merges).
            if let Some(file_params_value) = file_map.get_mut("parameters") {
                if let (Some(file_params_obj), Some(inline_params_obj)) = (file_params_value.as_object_mut(), inline_value.as_object()) {
                    for (param_name, param_value) in inline_params_obj {
                        file_params_obj.insert(param_name.clone(), param_value.clone());
                    }
                } else {
                    // If one is not an object, inline replaces completely
                    file_map.insert(key.clone(), inline_value.clone());
                }
            } else {
                file_map.insert(key.clone(), inline_value.clone());
            }
        } else {
            // For other top-level keys, inline value replaces file value
            file_map.insert(key.clone(), inline_value.clone());
        }
    }

    Ok(Value::Object(file_map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn inline_parameters_replace_file_parameters_by_name() {
        let file = r#"{"parameters": {"a": 1, "b": {"x": 1}}, "other": true}"#;
        let inline = "parameters:\n  b:\n    y: 2\n  c: 3\n";
        let merged = merge_parameters(file, inline).unwrap();
        assert_eq!(merged, json!({"parameters": {"a": 1, "b": {"y": 2}, "c": 3}, "other": true}));
    }

    #[test]
    fn parameters_must_be_an_object() {
        let err = merge_parameters("[1, 2]", "{}").unwrap_err();
        assert!(err.to_string().contains("must be an object"), "{err}");
    }

    #[test]
    fn invalid_parameters_name_the_input() {
        let err = parse_parameters("{not: valid: yaml", "InlineParameters").unwrap_err();
        assert!(err.to_string().contains("inline parameters"), "{err}");
    }
}
