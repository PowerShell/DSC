// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use args::{Args, ConfigSubCommand, DscType, OutputFormat, ResourceSubCommand, SubCommand};
use atty::Stream;
use clap::Parser;
use dsc_lib::{
    configure::{Configurator, ErrorAction,
        config_result::{ConfigurationGetResult, ConfigurationSetResult, ConfigurationTestResult}},
    configure::config_doc::Configuration,
    DscManager,
    dscresources::dscresource::{DscResource, Invoke},
    dscresources::invoke_result::{GetResult, SetResult, TestResult},
    dscresources::resource_manifest::ResourceManifest,
    dscerror::DscError};
use schemars::{schema_for, schema::RootSchema};
use serde_yaml::Value;
use std::collections::HashMap;
use std::io::{self, Read};
use std::process::exit;
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

#[cfg(debug_assertions)]
use crossterm::event;
#[cfg(debug_assertions)]
use std::env;

pub mod args;

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_ARGS: i32 = 1;
const EXIT_DSC_ERROR: i32 = 2;
const EXIT_JSON_ERROR: i32 = 3;
const EXIT_INVALID_INPUT: i32 = 4;

fn main() {
    #[cfg(debug_assertions)]
    check_debug();

    let args = Args::parse();

    let stdin: Option<String> = if atty::is(Stream::Stdin) {
        None
    } else {
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        let input = match String::from_utf8(buffer) {
            Ok(input) => input,
            Err(e) => {
                eprintln!("Invalid UTF-8 sequence: {e}");
                exit(EXIT_INVALID_ARGS);
            },
        };
        Some(input)
    };

    match args.subcommand {
        SubCommand::Config { subcommand } => {
            handle_config_subcommand(&subcommand, &args.format, &stdin);
        },
        SubCommand::Resource { subcommand } => {
            handle_resource_subcommand(&subcommand, &args.format, &stdin);
        },
        SubCommand::Schema { dsc_type } => {
            let schema = get_schema(dsc_type);
            let json = match serde_json::to_string(&schema) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, &args.format);
        },
    }

    exit(EXIT_SUCCESS);
}

fn serde_json_value_to_string(json: &serde_json::Value) -> String
{
    match serde_json::to_string(&json) {
        Ok(json_string) => json_string,
        Err(err) => {
            eprintln!("Error: Failed to convert JSON to string: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

fn new_configurator(json_string: &str) -> Configurator
{
    match Configurator::new(json_string) {
        Ok(configurator) => configurator,
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

fn handle_config_subcommand(subcommand: &ConfigSubCommand, format: &Option<OutputFormat>, stdin: &Option<String>) {
    if stdin.is_none() {
        eprintln!("Configuration must be piped to STDIN");
        exit(EXIT_INVALID_ARGS);
    }

    let json: serde_json::Value = match serde_json::from_str(stdin.as_ref().unwrap()) {
        Ok(json) => json,
        Err(_) => {
            match serde_yaml::from_str::<Value>(stdin.as_ref().unwrap()) {
                Ok(yaml) => {
                    match serde_json::to_value(yaml) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("Error: Failed to convert YAML to JSON: {err}");
                            exit(EXIT_DSC_ERROR);
                        }
                    }
                },
                Err(err) => {
                    eprintln!("Error: Input is not valid JSON or YAML: {err}");
                    exit(EXIT_INVALID_INPUT);
                }
            }
        }
    };

    let json_string = serde_json_value_to_string(&json);
    let configurator = new_configurator(&json_string);

    match subcommand {
        ConfigSubCommand::Get => {
            match configurator.invoke_get(ErrorAction::Continue, || { /* code */ }) {
                Ok(result) => {
                    let json = match serde_json::to_string(&result) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("JSON Error: {err}");
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    write_output(&json, format);
                    if result.had_errors {
                        exit(EXIT_DSC_ERROR);
                    }
                },
                Err(err) => {
                    eprintln!("Error: {err}");
                    exit(EXIT_DSC_ERROR);
                }
            }
        },
        ConfigSubCommand::Set => {
            match configurator.invoke_set(ErrorAction::Continue, || { /* code */ }) {
                Ok(result) => {
                    let json = match serde_json::to_string(&result) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("JSON Error: {err}");
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    write_output(&json, format);
                    if result.had_errors {
                        exit(EXIT_DSC_ERROR);
                    }
                },
                Err(err) => {
                    eprintln!("Error: {err}");
                    exit(EXIT_DSC_ERROR);
                }
            }
        },
        ConfigSubCommand::Test => {
            match configurator.invoke_test(ErrorAction::Continue, || { /* code */ }) {
                Ok(result) => {
                    let json = match serde_json::to_string(&result) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("JSON Error: {err}");
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    write_output(&json, format);
                    if result.had_errors {
                        exit(EXIT_DSC_ERROR);
                    }
                },
                Err(err) => {
                    eprintln!("Error: {err}");
                    exit(EXIT_DSC_ERROR);
                }
            }
        },
        ConfigSubCommand::Validate => {
            eprintln!("Validate configuration.. NOT IMPLEMENTED YET");
            exit(EXIT_DSC_ERROR);
        }
    }
}

fn handle_resource_subcommand(subcommand: &ResourceSubCommand, format: &Option<OutputFormat>, stdin: &Option<String>) {
    let mut dsc = match DscManager::new() {
        Ok(dsc) => dsc,
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    match subcommand {
        ResourceSubCommand::List { resource_name } => {
            match dsc.initialize_discovery() {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                    exit(EXIT_DSC_ERROR);
                }
            };
            for resource in dsc.find_resource(&resource_name.clone().unwrap_or_default()) {
                // convert to json
                let json = match serde_json::to_string(&resource) {
                    Ok(json) => json,
                    Err(err) => {
                        eprintln!("JSON Error: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                };
                write_output(&json, format);
                // insert newline separating instances if writing to console
                if atty::is(Stream::Stdout) {
                    println!();
                }
            }
        },
       ResourceSubCommand::Get { resource, input } => {
            handle_resource_get(&mut dsc, resource, input, stdin, format);
        },
        ResourceSubCommand::Set { resource, input } => {
            handle_resource_set(&mut dsc, resource, input, stdin, format);
        },
        ResourceSubCommand::Test { resource, input } => {
            handle_resource_test(&mut dsc, resource, input, stdin, format);
        },
        ResourceSubCommand::Schema { resource } => {
            handle_resource_schema(&mut dsc, resource, format);
        },
    }
}

fn add_fields_to_json(json: &str, fields_to_add: &HashMap<String, String>) -> Result<String, DscError>
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

fn add_type_name_to_json(json: String, type_name: String) -> String
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

fn handle_resource_get(dsc: &mut DscManager, resource: &str, input: &Option<String>, stdin: &Option<String>, format: &Option<OutputFormat>) {
    // TODO: support streaming stdin which includes resource and input
    let mut input = get_input(input, stdin);
    let mut resource = get_resource(dsc, resource);
    //TODO: add to debug stream: println!("handle_resource_get - {} implemented_as - {:?}", resource.type_name, resource.implemented_as);
    if resource.requires.is_some()
    {
        input = add_type_name_to_json(input, resource.type_name);
        resource = get_resource(dsc, &resource.requires.clone().unwrap());
    }

    //TODO: add to debug stream: println!("handle_resource_get - input - {}", input);

    match resource.get(input.as_str()) {
        Ok(result) => {
            // convert to json
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
        }
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

fn handle_resource_set(dsc: &mut DscManager, resource: &str, input: &Option<String>, stdin: &Option<String>, format: &Option<OutputFormat>) {
    let mut input = get_input(input, stdin);
    let mut resource = get_resource(dsc, resource);

    //TODO: add to debug stream: println!("handle_resource_set - {} implemented_as - {:?}", resource.type_name, resource.implemented_as);

    if resource.requires.is_some()
    {
        input = add_type_name_to_json(input, resource.type_name);
        resource = get_resource(dsc, &resource.requires.clone().unwrap());
    }

    //TODO: add to debug stream: println!("handle_resource_get - input - {}", input);

    match resource.set(input.as_str()) {
        Ok(result) => {
            // convert to json
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
        }
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

fn handle_resource_test(dsc: &mut DscManager, resource: &str, input: &Option<String>, stdin: &Option<String>, format: &Option<OutputFormat>) {
    let mut input = get_input(input, stdin);
    let mut resource = get_resource(dsc, resource);

    //TODO: add to debug stream: println!("handle_resource_test - {} implemented_as - {:?}", resource.type_name, resource.implemented_as);

    if resource.requires.is_some()
    {
        input = add_type_name_to_json(input, resource.type_name);
        resource = get_resource(dsc, &resource.requires.clone().unwrap());
    }

    //TODO: add to debug stream: println!("handle_resource_test - input - {}", input);

    match resource.test(input.as_str()) {
        Ok(result) => {
            // convert to json
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
        }
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

fn handle_resource_schema(dsc: &mut DscManager, resource: &str, format: &Option<OutputFormat>) {
    let resource = get_resource(dsc, resource);
    match resource.schema() {
        Ok(json) => {
            // verify is json
            match serde_json::from_str::<serde_json::Value>(json.as_str()) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
        }
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

fn get_schema(dsc_type: DscType) -> RootSchema {
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

fn write_output(json: &str, format: &Option<OutputFormat>) {
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

fn get_resource(dsc: &mut DscManager, resource: &str) -> DscResource {
    // check if resource is JSON or just a name
    match serde_json::from_str(resource) {
        Ok(resource) => resource,
        Err(err) => {
            if resource.contains('{') {
                eprintln!("Not valid resource JSON: {err}\nInput was: {resource}");
                exit(EXIT_INVALID_ARGS);
            }

            match dsc.initialize_discovery() {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                    exit(EXIT_DSC_ERROR);
                }
            };
            let resources: Vec<DscResource> = dsc.find_resource(resource).collect();
            match resources.len() {
                0 => {
                    eprintln!("Error: Resource not found: '{resource}'");
                    exit(EXIT_INVALID_ARGS);
                }
                1 => resources[0].clone(),
                _ => {
                    eprintln!("Error: Multiple resources found");
                    exit(EXIT_INVALID_ARGS);
                }
            }
        }
    }
}

fn get_input(input: &Option<String>, stdin: &Option<String>) -> String {
    let input = match (input, stdin) {
        (Some(_input), Some(_stdin)) => {
            eprintln!("Error: Cannot specify both --input and stdin");
            exit(EXIT_INVALID_ARGS);
        }
        (Some(input), None) => input.clone(),
        (None, Some(stdin)) => stdin.clone(),
        (None, None) => {
            return String::new();
        },
    };

    if input.is_empty() {
        return String::new();
    }

    match serde_json::from_str::<serde_json::Value>(&input) {
        Ok(_) => input,
        Err(json_err) => {
            match serde_yaml::from_str::<serde_yaml::Value>(&input) {
                Ok(yaml) => {
                    match serde_json::to_string(&yaml) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("Error: Cannot convert YAML to JSON: {err}");
                            exit(EXIT_INVALID_ARGS);
                        }
                    }
                },
                Err(err) => {
                    if input.contains('{') {
                        eprintln!("Error: Input is not valid JSON: {json_err}");
                    }
                    else {
                        eprintln!("Error: Input is not valid YAML: {err}");
                    }
                    exit(EXIT_INVALID_ARGS);
                }
            }
        }
    }
}

#[cfg(debug_assertions)]
fn check_debug() {
    if env::var("DEBUG_DSC").is_ok() {
        eprintln!("attach debugger to pid {} and press a key to continue", std::process::id());
        loop {
            let event = event::read().unwrap();
            if let event::Event::Key(key) = event {
                // workaround bug in 0.26+ https://github.com/crossterm-rs/crossterm/issues/752#issuecomment-1414909095
                if key.kind == event::KeyEventKind::Press {
                    break;
                }
            } else {
                eprintln!("Unexpected event: {event:?}");
                continue;
            }
        }
    }
}
