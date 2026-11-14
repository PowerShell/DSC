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
    settings::{fields::TracingFormatField, global as global_settings, sources::{CommandLineData, EnvironmentData}},
};
use dsc_lib_telemetry::{StderrFormat, basic::BasicTracingOptions};
use rust_i18n::t;
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
use tracing::{Level, debug, error, info, trace};

// The exit codes match the ones `dsc` uses for the same conditions.
pub const EXIT_SUCCESS: u8 = 0;
pub const EXIT_INVALID_ARGS: u8 = 1;
pub const EXIT_DSC_ERROR: u8 = 2;
pub const EXIT_JSON_ERROR: u8 = 3;
pub const EXIT_INVALID_INPUT: u8 = 4;
pub const EXIT_VALIDATION_FAILED: u8 = 5;

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

/// Enable tracing the same way `dsc` does, from the resolved DSC settings.
///
/// The settings resolve the trace level and format from the policy file, the command-line
/// arguments, the `DSC_TRACE_LEVEL` and `DSC_TRACE_FORMAT` environment variables, and the settings
/// files, in that order of precedence. The environment variables are how `dsc` hands its own
/// options to `dsclint` when it runs as the `dsc lint` plug-in command.
///
/// # Arguments
///
/// * `trace_level_arg` - The trace level from the command line, if given.
/// * `trace_format_arg` - The trace format from the command line, if given.
/// * `ignore_settings_file` - Whether the command line asked to ignore the settings files.
pub fn enable_tracing(trace_level_arg: Option<&TraceLevel>, trace_format_arg: Option<&TraceFormat>, ignore_settings_file: bool) {
    let default_guard = BasicTracingOptions::init_default_guard();

    let command_line = CommandLineData {
        trace_level: trace_level_arg.cloned().map(Into::into),
        trace_format: trace_format_arg.map(|format| match format {
            TraceFormat::Default => TracingFormatField::Default,
            TraceFormat::Plaintext => TracingFormatField::Plaintext,
            TraceFormat::Json | TraceFormat::PassThrough => TracingFormatField::Json,
        }),
        ignore_settings_file: ignore_settings_file.then(|| true.into()),
    };
    let mut settings = global_settings::get_or_init_with_command_line(command_line);
    settings.load();
    settings.resolve_all();
    let tracing_setting = settings.resolved().tracing.clone();
    drop(settings);

    let tracing_level: Level = tracing_setting.level.value.clone().into();
    let stderr_format = match tracing_setting.format.value {
        TracingFormatField::Default => StderrFormat::Default,
        TracingFormatField::Plaintext => StderrFormat::Plaintext,
        TracingFormatField::Json => StderrFormat::Json,
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
        env::set_var(EnvironmentData::DSC_TRACE_LEVEL_ENV_VAR, tracing_setting.level.value.to_string());
    }
    info!("Trace-level is {:?}", tracing_setting.level.value);
}

/// Read an input document as text without parsing it, rejecting an empty document.
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
    let value = read_document(input, file)?;
    if (input.is_some() || file.is_some()) && value.trim().is_empty() {
        error!("{}", t!("util.emptyInput"));
        return Err(ExitCode::from(EXIT_INVALID_INPUT));
    }
    Ok(value)
}

/// Read an input document as text without parsing it, including an empty document.
///
/// The linter reports an empty document as a diagnostic in its result, so the document is
/// returned as read. Use [`read_input`] for input that must not be empty, like parameters.
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
/// file can't be read, or STDIN isn't valid UTF-8.
pub fn read_document(input: Option<&String>, file: Option<&String>) -> Result<String, ExitCode> {
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
        String::new()
    };

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
