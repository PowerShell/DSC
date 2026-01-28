// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::args::{SchemaType, OutputFormat, TraceFormat};
use crate::resolve::Include;
use dsc_lib::{
    configure::{
        config_doc::{
            Configuration,
            Resource,
            RestartRequired,
        },
        config_result::{
            ConfigurationGetResult,
            ConfigurationSetResult,
            ConfigurationTestResult,
            ResourceTestResult,
        },
    },
    discovery::{
        command_discovery::ManifestList,
        Discovery,
    },
    dscerror::DscError,
    dscresources::{
        command_resource::TraceLevel,
        dscresource::DscResource,
        invoke_result::{
            GetResult,
            SetResult,
            TestResult,
            ResolveResult,
        },
        resource_manifest::ResourceManifest
    },
    extensions::{
        discover::DiscoverResult,
        dscextension::Capability,
        extension_manifest::ExtensionManifest,
    },
    functions::FunctionDefinition,
    util::{
        get_setting,
        parse_input_to_json,
    },
};
use path_absolutize::Absolutize;
use rust_i18n::t;
use schemars::{Schema, schema_for};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::io::{IsTerminal, Read, stdout, Write};
use std::path::Path;
use std::process::exit;
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings}
};
use tracing::{Level, debug, error, info, warn, trace};
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, Layer};
use tracing_indicatif::IndicatifLayer;

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_INVALID_ARGS: i32 = 1;
pub const EXIT_DSC_ERROR: i32 = 2;
pub const EXIT_JSON_ERROR: i32 = 3;
pub const EXIT_INVALID_INPUT: i32 = 4;
pub const EXIT_VALIDATION_FAILED: i32 = 5;
pub const EXIT_CTRL_C: i32 = 6;
pub const EXIT_DSC_RESOURCE_NOT_FOUND: i32 = 7;
pub const EXIT_DSC_ASSERTION_FAILED: i32 = 8;
pub const EXIT_MCP_FAILED: i32 = 9;
pub const EXIT_BICEP_FAILED: i32 = 10;

pub const DSC_CONFIG_ROOT: &str = "DSC_CONFIG_ROOT";
pub const DSC_TRACE_LEVEL: &str = "DSC_TRACE_LEVEL";

#[derive(Deserialize)]
pub struct TracingSetting {
    /// Trace level to use - see pub enum `TraceLevel` in `dsc_lib\src\dscresources\command_resource.rs`
    level:  TraceLevel,
    /// Trace format to use - see pub enum `TraceFormat` in `dsc\src\args.rs`
    format: TraceFormat,
    /// Whether the 'level' can be overrridden by `DSC_TRACE_LEVEL` environment variable
    #[serde(rename = "allowOverride")]
    allow_override: bool
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
            error!("{}: {err}", t!("util.failedToConvertJsonToString"));
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

/// Get the JSON schema for requested type.
///
/// # Arguments
///
/// * `dsc_type` - The type of schema to get
///
/// # Returns
///
/// * `Schema` - The schema
#[must_use]
pub fn get_schema(schema: SchemaType) -> Schema {
    match schema {
        SchemaType::Configuration => {
            schema_for!(Configuration)
        },
        SchemaType::ConfigurationGetResult => {
            schema_for!(ConfigurationGetResult)
        },
        SchemaType::ConfigurationSetResult => {
            schema_for!(ConfigurationSetResult)
        },
        SchemaType::ConfigurationTestResult => {
            schema_for!(ConfigurationTestResult)
        },
        SchemaType::DscResource => {
            schema_for!(DscResource)
        },
        SchemaType::ExtensionDiscoverResult => {
            schema_for!(DiscoverResult)
        },
        SchemaType::ExtensionManifest => {
            schema_for!(ExtensionManifest)
        },
        SchemaType::FunctionDefinition => {
            schema_for!(FunctionDefinition)
        },
        SchemaType::GetResult => {
            schema_for!(GetResult)
        },
        SchemaType::Include => {
            schema_for!(Include)
        },
        SchemaType::ManifestList => {
            schema_for!(ManifestList)
        },
        SchemaType::ResolveResult => {
            schema_for!(ResolveResult)
        },
        SchemaType::Resource => {
            schema_for!(Resource)
        },
        SchemaType::ResourceManifest => {
            schema_for!(ResourceManifest)
        },
        SchemaType::RestartRequired => {
            schema_for!(RestartRequired)
        },
        SchemaType::SetResult => {
            schema_for!(SetResult)
        },
        SchemaType::TestResult => {
            schema_for!(TestResult)
        },
    }
}

/// Write the JSON object to the console
///
/// # Arguments
///
/// * `json` - The JSON to write
/// * `format` - The format to use
/// * `include_separator` - Whether to include a separator for YAML before the object
pub fn write_object(json: &str, format: Option<&OutputFormat>, include_separator: bool) {
    let mut is_json = true;
    let mut output_format = format;
    let mut syntax_color = false;
    if std::io::stdout().is_terminal() {
        syntax_color = true;
        if output_format.is_none() {
            output_format = Some(&OutputFormat::Yaml);
        }
    }
    else if output_format.is_none() {
        output_format = Some(&OutputFormat::Json);
    }

    let output = match output_format {
        Some(OutputFormat::Json) => json.to_string(),
        Some(OutputFormat::PrettyJson) => {
            let value: serde_json::Value = match serde_json::from_str(json) {
                Ok(value) => value,
                Err(err) => {
                    error!("JSON: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            match serde_json::to_string_pretty(&value) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            }
        },
        Some(OutputFormat::Yaml) | None => {
            is_json = false;
            if include_separator {
                println!("---");
            }

            let value: serde_json::Value = match serde_json::from_str(json) {
                Ok(value) => value,
                Err(err) => {
                    error!("JSON: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            match serde_yaml::to_string(&value) {
                Ok(yaml) => yaml,
                Err(err) => {
                    error!("YAML: {err}");
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
        let mut stdout_lock = stdout().lock();
        if writeln!(stdout_lock, "{output}").is_err() {
            // likely caused by a broken pipe (e.g. 'head' command closed early)
            exit(EXIT_SUCCESS);
        }
    }
}

#[allow(clippy::too_many_lines)]
pub fn enable_tracing(trace_level_arg: Option<&TraceLevel>, trace_format_arg: Option<&TraceFormat>) {

    let mut policy_is_used = false;
    let mut tracing_setting = TracingSetting::default();

    let default_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("warn"))
        .unwrap_or_default()
        .add_directive(Level::WARN.into());
    let default_indicatif_layer = IndicatifLayer::new();
    let default_layer = tracing_subscriber::fmt::Layer::default().with_writer(default_indicatif_layer.get_stderr_writer());
    let default_fmt = default_layer
                .with_ansi(true)
                .with_level(true)
                .boxed();
    let default_subscriber = tracing_subscriber::Registry::default().with(default_fmt).with(default_filter).with(default_indicatif_layer);
    let default_guard = tracing::subscriber::set_default(default_subscriber);

    // read setting/policy from files
    if let Ok(v) = get_setting("tracing") {
        if v.policy != serde_json::Value::Null {
            match serde_json::from_value::<TracingSetting>(v.policy) {
                Ok(v) => {
                    tracing_setting = v;
                    policy_is_used = true;
                },
                Err(e) => { error!("{e}"); }
            }
        } else if v.setting != serde_json::Value::Null {
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
    if tracing_setting.allow_override {
        if let Ok(level) = env::var(DSC_TRACE_LEVEL) {
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

    // enable tracing
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("warn"))
        .unwrap_or_default()
        .add_directive(tracing_level.into());
    let indicatif_layer = IndicatifLayer::new();
    let layer = tracing_subscriber::fmt::Layer::default().with_writer(indicatif_layer.get_stderr_writer());
    let with_source = tracing_level == Level::DEBUG || tracing_level == Level::TRACE;
    let fmt = match tracing_setting.format {
        TraceFormat::Default => {
            layer
                .with_ansi(true)
                .with_level(true)
                .with_target(with_source)
                .with_line_number(with_source)
                .boxed()
        },
        TraceFormat::Plaintext => {
            layer
                .with_ansi(false)
                .with_level(true)
                .with_target(with_source)
                .with_line_number(with_source)
                .boxed()
        },
        TraceFormat::Json | TraceFormat::PassThrough => {
            layer
                .with_ansi(false)
                .with_level(true)
                .with_target(with_source)
                .with_line_number(with_source)
                .json()
                .boxed()
        },
    };

    let subscriber = tracing_subscriber::Registry::default().with(fmt).with(filter).with(indicatif_layer);

    drop(default_guard);
    if tracing::subscriber::set_global_default(subscriber).is_err() {
        eprintln!("{}", t!("util.failedToSetTracing"));
    }

    // set DSC_TRACE_LEVEL for child processes
    env::set_var(DSC_TRACE_LEVEL, tracing_level.to_string().to_ascii_lowercase());
    info!("Trace-level is {:?}", tracing_setting.level);
}

pub fn get_input(input: Option<&String>, file: Option<&String>) -> String {
    trace!("Input: {input:?}, File: {file:?}");
    let value = if let Some(input) = input {
        debug!("{}", t!("util.readingInput"));

        // see if user accidentally passed in a file path
        if Path::new(input).exists() {
            error!("{}", t!("util.inputIsFile"));
            exit(EXIT_INVALID_INPUT);
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
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                },
                Err(err) => {
                    error!("{}: {err}", t!("util.failedToReadStdin"));
                    exit(EXIT_INVALID_INPUT);
                }
            }
        } else {
            // see if an extension should handle this file
            let mut discovery = Discovery::new();
            let path_buf = Path::new(path);
            for extension in discovery.get_extensions(&Capability::Import) {
                if let Ok(content) = extension.import(path_buf) {
                    return content;
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
                    exit(EXIT_INVALID_INPUT);
                }
            }
        }
    } else {
        debug!("{}", t!("util.noInput"));
        return String::new();
    };

    if value.trim().is_empty() {
        error!("{}", t!("util.emptyInput"));
        exit(EXIT_INVALID_INPUT);
    }

    match parse_input_to_json(&value) {
        Ok(json) => json,
        Err(err) => {
            error!("{}: {err}", t!("util.failedToParseInput"));
            exit(EXIT_INVALID_INPUT);
        }
    }
}

/// Sets `DSC_CONFIG_ROOT` env var and makes path absolute.
///
/// # Arguments
///
/// * `config_path` - Full path to the config file or directory.
///
/// # Returns
///
/// Absolute full path to the config file.
/// If a directory is provided, the path returned is the directory path.
pub fn set_dscconfigroot(config_path: &str) -> String
{
    let path = Path::new(config_path);

    // make path absolute
    let Ok(full_path) = path.absolutize() else {
            error!("{}", t!("util.failedToAbsolutizePath"));
            exit(EXIT_DSC_ERROR);
    };

    let config_root_path = if full_path.is_file() {
        let Some(config_root_path) = full_path.parent() else {
            // this should never happen because path was made absolute
            error!("{}", t!("util.failedToGetParentPath"));
            exit(EXIT_DSC_ERROR);
        };
        config_root_path.to_string_lossy().into_owned()
    } else {
        config_path.to_string()
    };

    // warn if env var is already set/used
    if env::var(DSC_CONFIG_ROOT).is_ok() {
        warn!("{}", t!("util.dscConfigRootAlreadySet"));
    }

    // Set env var so child processes (of resources) can use it
    debug!("{} '{config_root_path}'", t!("util.settingDscConfigRoot"));
    env::set_var(DSC_CONFIG_ROOT, config_root_path);

    full_path.to_string_lossy().into_owned()
}


/// Check if the test result is in the desired state.
///
/// # Arguments
///
/// * `test_result` - The test result to check
///
/// # Returns
///
/// * `bool` - True if the test result is in the desired state, false otherwise
#[must_use]
pub fn in_desired_state(test_result: &ResourceTestResult) -> bool {
    match &test_result.result {
        TestResult::Resource(result) => {
            result.in_desired_state
        },
        TestResult::Group(results) => {
            for result in results {
                if !in_desired_state(result) {
                    return false;
                }
            }
            true
        }
    }
}

/// Parse input string as JSON or YAML and return a serde_json::Value.
///
/// # Arguments
///
/// * `input` - The input string to parse (JSON or YAML format)
/// * `context` - Context string for error messages (e.g., "file parameters", "inline parameters")
///
/// # Returns
///
/// * `Result<serde_json::Value, DscError>` - Parsed JSON value
///
/// # Errors
///
/// This function will return an error if the input cannot be parsed as valid JSON or YAML
fn parse_input_to_json_value(input: &str, context: &str) -> Result<serde_json::Value, DscError> {
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

/// Convert parameter input to a map, handling different formats.
///
/// # Arguments
///
/// * `params` - Parameter string to convert (JSON or YAML format)
/// * `context` - Context string for error messages
///
/// # Returns
///
/// * `Result<serde_json::Map<String, serde_json::Value>, DscError>` - Parameter map
///
/// # Errors
///
/// Returns an error if the input cannot be parsed or is not an object
fn params_to_map(params: &str, context: &str) -> Result<serde_json::Map<String, serde_json::Value>, DscError> {
    let value = parse_input_to_json_value(params, context)?;
    
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
/// * `Result<String, DscError>` - Merged parameters as JSON string
///
/// # Errors
///
/// This function will return an error if:
/// - Either parameter set cannot be parsed as valid JSON or YAML
/// - The merged result cannot be serialized to JSON
pub fn merge_parameters(file_params: &str, inline_params: &str) -> Result<String, DscError> {
    use serde_json::Value;
    
    // Convert both parameter inputs to maps
    let mut file_map = params_to_map(file_params, "FileParameters")?;
    let inline_map = params_to_map(inline_params, "InlineParameters")?;

    // Merge top-level keys
    for (key, inline_value) in &inline_map {
        if key == "parameters" {
            // Special handling for "parameters" key - merge at parameter name level only
            // Within each parameter name, inline replaces (not merges)
            if let Some(file_params_value) = file_map.get_mut("parameters") {
                if let (Some(file_params_obj), Some(inline_params_obj)) = (file_params_value.as_object_mut(), inline_value.as_object()) {
                    // For each parameter in inline, replace (not merge) in file
                    for (param_name, param_value) in inline_params_obj {
                        file_params_obj.insert(param_name.clone(), param_value.clone());
                    }
                } else {
                    // If one is not an object, inline replaces completely
                    file_map.insert(key.clone(), inline_value.clone());
                }
            } else {
                // "parameters" key doesn't exist in file, add it
                file_map.insert(key.clone(), inline_value.clone());
            }
        } else {
            // For other top-level keys, inline value replaces file value
            file_map.insert(key.clone(), inline_value.clone());
        }
    }

    let merged = Value::Object(file_map);
    Ok(serde_json::to_string(&merged)?)
}
