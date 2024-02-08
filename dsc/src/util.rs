// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::args::{DscType, OutputFormat, TraceFormat, TraceLevel};

use atty::Stream;
use dsc_lib::{
    configure::{
        config_doc::Configuration,
        config_result::{ConfigurationGetResult, ConfigurationSetResult, ConfigurationTestResult}
    },
    dscerror::DscError,
    dscresources::{
        dscresource::DscResource,
        invoke_result::{GetResult, SetResult, TestResult},
        resource_manifest::ResourceManifest
    }
};
use schemars::{schema_for, schema::RootSchema};
use serde_yaml::Value;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::exit;
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings}
};
use tracing::{Level, debug, error};
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, Layer};

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_INVALID_ARGS: i32 = 1;
pub const EXIT_DSC_ERROR: i32 = 2;
pub const EXIT_JSON_ERROR: i32 = 3;
pub const EXIT_INVALID_INPUT: i32 = 4;
pub const EXIT_VALIDATION_FAILED: i32 = 5;
pub const EXIT_CTRL_C: i32 = 6;

/// Get string representation of JSON value.
///
/// # Arguments
///
/// * `json` - The JSON to convert
///
/// # Returns
///
/// * `String` - The JSON as a string
#[must_use]
pub fn serde_json_value_to_string(json: &serde_json::Value) -> String
{
    match serde_json::to_string(&json) {
        Ok(json_string) => json_string,
        Err(err) => {
            error!("Error: Failed to convert JSON to string: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

/// Add fields to the JSON.
///
/// # Arguments
///
/// * `json` - The JSON to add the fields to
/// * `fields_to_add` - The fields to add
///
/// # Returns
///
/// * `String` - The JSON with the fields added
///
/// # Errors
///
/// * `DscError` - The JSON is invalid
#[allow(clippy::implicit_hasher)]
pub fn add_fields_to_json(json: &str, fields_to_add: &HashMap<String, String>) -> Result<String, DscError>
{
    let mut v = serde_json::from_str::<serde_json::Value>(json)?;

    if let serde_json::Value::Object(ref mut map) = v {
        for (k, v) in fields_to_add {
            map.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
    }

    let result = serde_json::to_string(&v)?;
    Ok(result)
}

/// Add the type property value to the JSON.
///
/// # Arguments
///
/// * `json` - The JSON to add the type property to
/// * `type_name` - The type name to add
///
/// # Returns
///
/// * `String` - The JSON with the type property added
#[must_use]
pub fn add_type_name_to_json(json: String, type_name: String) -> String
{
    let mut map:HashMap<String,String> = HashMap::new();
    map.insert(String::from("type"), type_name);

    let mut j = json;
    if j.is_empty()
    {
        j = String::from("{}");
    }

    match add_fields_to_json(&j, &map) {
        Ok(json) => json,
        Err(err) => {
            error!("JSON Error: {err}");
            exit(EXIT_JSON_ERROR);
        }
    }
}

/// Get the JSON schema for requested type.
///
/// # Arguments
///
/// * `dsc_type` - The type of schema to get
///
/// # Returns
///
/// * `RootSchema` - The schema
#[must_use]
pub fn get_schema(dsc_type: DscType) -> RootSchema {
    match dsc_type {
        DscType::GetResult => {
            schema_for!(GetResult)
        },
        DscType::SetResult => {
            schema_for!(SetResult)
        },
        DscType::TestResult => {
            schema_for!(TestResult)
        },
        DscType::DscResource => {
            schema_for!(DscResource)
        },
        DscType::ResourceManifest => {
            schema_for!(ResourceManifest)
        },
        DscType::Configuration => {
            schema_for!(Configuration)
        },
        DscType::ConfigurationGetResult => {
            schema_for!(ConfigurationGetResult)
        },
        DscType::ConfigurationSetResult => {
            schema_for!(ConfigurationSetResult)
        },
        DscType::ConfigurationTestResult => {
            schema_for!(ConfigurationTestResult)
        },
    }
}

/// Write the output to the console
///
/// # Arguments
///
/// * `json` - The JSON to write
/// * `format` - The format to use
pub fn write_output(json: &str, format: &Option<OutputFormat>) {
    let mut is_json = true;
    let mut output_format = format.clone();
    let mut syntax_color = false;
    if atty::is(Stream::Stdout) {
        syntax_color = true;
        if output_format.is_none() {
            output_format = Some(OutputFormat::Yaml);
        }
    }
    else if output_format.is_none() {
        output_format = Some(OutputFormat::Json);
    }

    let output = match output_format {
        Some(OutputFormat::Json) => json.to_string(),
        Some(OutputFormat::PrettyJson) => {
            let value: serde_json::Value = match serde_json::from_str(json) {
                Ok(value) => value,
                Err(err) => {
                    error!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            match serde_json::to_string_pretty(&value) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            }
        },
        Some(OutputFormat::Yaml) | None => {
            is_json = false;
            let value: serde_json::Value = match serde_json::from_str(json) {
                Ok(value) => value,
                Err(err) => {
                    error!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            match serde_yaml::to_string(&value) {
                Ok(yaml) => yaml,
                Err(err) => {
                    error!("YAML Error: {err}");
                    exit(EXIT_JSON_ERROR);
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
            return;
        };

        let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

        for line in LinesWithEndings::from(&output) {
            let Ok(ranges) = h.highlight_line(line, &ps) else {
                continue;
            };
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            print!("{escaped}");
        }
    }
    else {
        println!("{output}");
    }
}

pub fn enable_tracing(trace_level: &TraceLevel, trace_format: &TraceFormat) {
    let tracing_level = match trace_level {
        TraceLevel::Error => Level::ERROR,
        TraceLevel::Warning => Level::WARN,
        TraceLevel::Info => Level::INFO,
        TraceLevel::Debug => Level::DEBUG,
        TraceLevel::Trace => Level::TRACE,
    };

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("warning"))
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
        eprintln!("Unable to set global default tracing subscriber.  Tracing is diabled.");
    }
}

pub fn parse_input_to_json(value: &str) -> String {
    match serde_json::from_str(value) {
        Ok(json) => json,
        Err(_) => {
            match serde_yaml::from_str::<Value>(value) {
                Ok(yaml) => {
                    match serde_json::to_value(yaml) {
                        Ok(json) => json.to_string(),
                        Err(err) => {
                            error!("Error: Failed to convert YAML to JSON: {err}");
                            exit(EXIT_DSC_ERROR);
                        }
                    }
                },
                Err(err) => {
                    error!("Error: Input is not valid JSON or YAML: {err}");
                    exit(EXIT_INVALID_INPUT);
                }
            }
        }
    }
}

pub fn get_input(input: &Option<String>, stdin: &Option<String>, path: &Option<String>) -> String {
    let value = match (input, stdin, path) {
        (Some(_), Some(_), None) | (None, Some(_), Some(_)) => {
            error!("Error: Cannot specify both stdin and --input or --path");
            exit(EXIT_INVALID_ARGS);
        },
        (Some(input), None, None) => {
            debug!("Reading input from command line parameter");
            input.clone()
        },
        (None, Some(stdin), None) => {
            debug!("Reading input from stdin");
            stdin.clone()
        },
        (None, None, Some(path)) => {
            debug!("Reading input from file {}", path);
            match std::fs::read_to_string(path) {
                Ok(input) => {
                    input.clone()
                },
                Err(err) => {
                    error!("Error: Failed to read input file: {err}");
                    exit(EXIT_INVALID_INPUT);
                }
            }
        },
        (None, None, None) => {
            debug!("No input provided via stdin, file, or command line");
            return String::new();
        },
        _default => {
            /* clap should handle these cases via conflicts_with so this should not get reached */
            error!("Error: Invalid input");
            exit(EXIT_INVALID_ARGS);
        }
    };

    if value.trim().is_empty() {
        debug!("Provided input is empty");
        return String::new();
    }

    parse_input_to_json(&value)
}

pub fn process_macros(json_string: &str, config_path: &str) -> String
{
    let path = Path::new(config_path);
    let config_root = match path.parent()
    {
        Some(dir_path) => { dir_path.to_str().unwrap_or_default().to_string()},
        _ => String::new()
    };

    // Set env var so child processes (of resources) can use it
    debug!("Setting 'DSCConfigRoot' env var as '{}'", config_root);
    env::set_var("DSCConfigRoot", config_root.clone());

    // we are replacing a substring in json, so the new substring must be properly escaped per json rules
    let v = json!(config_root);
    let mut json_escaped_config_root = v.to_string();
    json_escaped_config_root.pop(); // remove last double quote
    if !json_escaped_config_root.is_empty() {
        json_escaped_config_root.remove(0); // remove first double quote
    }

    debug!("Escaped json 'DSCConfigRoot' is '{}'", json_escaped_config_root);
    json_string.replace("_DSCConfigRoot_", &json_escaped_config_root)
}
