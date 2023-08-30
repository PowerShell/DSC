// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::args::{DscType, OutputFormat};

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
use std::collections::HashMap;
use std::process::exit;
use syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings}
};

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_INVALID_ARGS: i32 = 1;
pub const EXIT_DSC_ERROR: i32 = 2;
pub const EXIT_JSON_ERROR: i32 = 3;
pub const EXIT_INVALID_INPUT: i32 = 4;
pub const EXIT_VALIDATION_FAILED: i32 = 5;
pub const EXIT_CTRL_C: i32 = 6;

pub fn serde_json_value_to_string(json: &serde_json::Value) -> String
{
    match serde_json::to_string(&json) {
        Ok(json_string) => json_string,
        Err(err) => {
            eprintln!("Error: Failed to convert JSON to string: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

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
            eprintln!("JSON Error: {err}");
            exit(EXIT_JSON_ERROR);
        }
    }
}

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

pub fn write_output(json: &str, format: &Option<OutputFormat>) {
    let mut is_json = true;
    if atty::is(Stream::Stdout) {
        let output = match format {
            Some(OutputFormat::Json) => json.to_string(),
            Some(OutputFormat::PrettyJson) => {
                let value: serde_json::Value = match serde_json::from_str(json) {
                    Ok(value) => value,
                    Err(err) => {
                        eprintln!("JSON Error: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                };
                match serde_json::to_string_pretty(&value) {
                    Ok(json) => json,
                    Err(err) => {
                        eprintln!("JSON Error: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                }
            },
            Some(OutputFormat::Yaml) | None => {
                is_json = false;
                let value: serde_json::Value = match serde_json::from_str(json) {
                    Ok(value) => value,
                    Err(err) => {
                        eprintln!("JSON Error: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                };
                match serde_yaml::to_string(&value) {
                    Ok(yaml) => yaml,
                    Err(err) => {
                        eprintln!("YAML Error: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                }
            }
        };

        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let syntax = if is_json {
            ps.find_syntax_by_extension("json").unwrap()
        } else {
            ps.find_syntax_by_extension("yaml").unwrap()
        };

        let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

        for line in LinesWithEndings::from(&output) {
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            print!("{escaped}");
        }
    } else {
        println!("{json}");
    }
}
