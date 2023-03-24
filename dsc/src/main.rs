use args::*;
use atty::Stream;
use clap::Parser;
use dsc_lib::{configure::{Configurator, ErrorAction}, configure::config_doc::Configuration, DscManager, dscresources::dscresource::{DscResource, Invoke}, dscresources::invoke_result::{GetResult, SetResult, TestResult}, dscresources::resource_manifest::ResourceManifest};
use schemars::schema_for;
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
                eprintln!("Invalid UTF-8 sequence: {}", e);
                exit(EXIT_INVALID_ARGS);
            },
        };
        Some(input)
    };

    let mut dsc = match DscManager::new() {
        Ok(dsc) => dsc,
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(EXIT_DSC_ERROR);
        }
    };

    match args.subcommand {
        SubCommand::Config { subcommand } => {
            if stdin.is_none() {
                eprintln!("Configuration JSON must be piped to stdin");
                exit(EXIT_INVALID_ARGS);
            }
            let configurator = match Configurator::new(&stdin.unwrap()) {
                Ok(configurator) => configurator,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    exit(EXIT_DSC_ERROR);
                }
            };
            match subcommand {
                ConfigSubCommand::Get => {
                    match configurator.invoke_get(ErrorAction::Continue, || { /* code */ }) {
                        Ok(result) => {
                            let json = match serde_json::to_string(&result) {
                                Ok(json) => json,
                                Err(err) => {
                                    eprintln!("JSON Error: {}", err);
                                    exit(EXIT_JSON_ERROR);
                                }
                            };
                            write_output(&json, &args.format);
                        },
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            exit(EXIT_DSC_ERROR);
                        }
                    }
                },
                ConfigSubCommand::Set => {
                    eprintln!("Setting configuration... NOT IMPLEMENTED YET");
                    exit(EXIT_DSC_ERROR);
                },
                ConfigSubCommand::Test => {
                    eprintln!("Testing configuration... NOT IMPLEMENTED YET");
                    exit(EXIT_DSC_ERROR);
                },
            }
        },
        SubCommand::Resource { subcommand } => {
            match subcommand {
                ResourceSubCommand::List { resource_name } => {
                    match dsc.initialize_discovery() {
                        Ok(_) => (),
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            exit(EXIT_DSC_ERROR);
                        }
                    };
                    for resource in dsc.find_resource(&resource_name.unwrap_or_default()) {
                        // convert to json
                        let json = match serde_json::to_string(&resource) {
                            Ok(json) => json,
                            Err(err) => {
                                eprintln!("JSON Error: {}", err);
                                exit(EXIT_JSON_ERROR);
                            }
                        };
                        write_output(&json, &args.format);
                        // insert newline separating instances if writing to console
                        if atty::is(Stream::Stdout) {
                            println!();
                        }
                    }
                },
                ResourceSubCommand::Get { resource, input } => {
                    // TODO: support streaming stdin which includes resource and input

                    let input = get_input(&input, &stdin);
                    let resource = get_resource(&mut dsc, resource.as_str());
                    match resource.get(input.as_str()) {
                        Ok(result) => {
                            // convert to json
                            let json = match serde_json::to_string(&result) {
                                Ok(json) => json,
                                Err(err) => {
                                    eprintln!("JSON Error: {}", err);
                                    exit(EXIT_JSON_ERROR);
                                }
                            };
                            write_output(&json, &args.format);
                        }
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            exit(EXIT_DSC_ERROR);
                        }
                    }
                },
                ResourceSubCommand::Set { resource, input: _ } => {
                    let input = get_input(&None, &stdin);
                    let resource = get_resource(&mut dsc, resource.as_str());
                    match resource.set(input.as_str()) {
                        Ok(result) => {
                            // convert to json
                            let json = match serde_json::to_string(&result) {
                                Ok(json) => json,
                                Err(err) => {
                                    eprintln!("JSON Error: {}", err);
                                    exit(EXIT_JSON_ERROR);
                                }
                            };
                            write_output(&json, &args.format);
                        }
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            exit(EXIT_DSC_ERROR);
                        }
                    }
                },
                ResourceSubCommand::Test { resource, input: _ } => {
                    let input = get_input(&None, &stdin);
                    let resource = get_resource(&mut dsc, resource.as_str());
                    match resource.test(input.as_str()) {
                        Ok(result) => {
                            // convert to json
                            let json = match serde_json::to_string(&result) {
                                Ok(json) => json,
                                Err(err) => {
                                    eprintln!("JSON Error: {}", err);
                                    exit(EXIT_JSON_ERROR);
                                }
                            };
                            write_output(&json, &args.format);
                        }
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            exit(EXIT_DSC_ERROR);
                        }
                    }
                },
                ResourceSubCommand::Schema { resource } => {
                    let resource = get_resource(&mut dsc, resource.as_str());
                    match resource.schema() {
                        Ok(json) => {
                            // verify is json
                            match serde_json::from_str::<serde_json::Value>(json.as_str()) {
                                Ok(_) => (),
                                Err(err) => {
                                    eprintln!("Error: {}", err);
                                    exit(EXIT_JSON_ERROR);
                                }
                            };
                            write_output(&json, &args.format);
                        }
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            exit(EXIT_DSC_ERROR);
                        }
                    }
                },
            }
        },
        SubCommand::Schema { dsc_type } => {
            match dsc_type {
                DscType::GetResult => {
                    let schema = schema_for!(GetResult);
                    // convert to json
                    let json = match serde_json::to_string(&schema) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("JSON Error: {}", err);
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    write_output(&json, &args.format);
                },
                DscType::SetResult => {
                    let schema = schema_for!(SetResult);
                    // convert to json
                    let json = match serde_json::to_string(&schema) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("JSON Error: {}", err);
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    write_output(&json, &args.format);
                },
                DscType::TestResult => {
                    let schema = schema_for!(TestResult);
                    // convert to json
                    let json = match serde_json::to_string(&schema) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("JSON Error: {}", err);
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    write_output(&json, &args.format);
                },
                DscType::DscResource => {
                    let schema = schema_for!(DscResource);
                    // convert to json
                    let json = match serde_json::to_string(&schema) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("JSON Error: {}", err);
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    write_output(&json, &args.format);
                },
                DscType::ResourceManifest => {
                    let schema = schema_for!(ResourceManifest);
                    // convert to json
                    let json = match serde_json::to_string(&schema) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("JSON Error: {}", err);
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    write_output(&json, &args.format);
                },
                DscType::Configuration => {
                    let schema = schema_for!(Configuration);
                    // convert to json
                    let json = match serde_json::to_string(&schema) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("JSON Error: {}", err);
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    write_output(&json, &args.format);
                },
            }
        },
    }

    exit(EXIT_SUCCESS);
}

fn write_output(json: &str, format: &Option<OutputFormat>) {
    let mut is_json = true;
    match atty::is(Stream::Stdout) {
        true => {
            let output = match format {
                Some(OutputFormat::Json) => json.to_string(),
                Some(OutputFormat::PrettyJson) => {
                    let value: serde_json::Value = match serde_json::from_str(json) {
                        Ok(value) => value,
                        Err(err) => {
                            eprintln!("JSON Error: {}", err);
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    match serde_json::to_string_pretty(&value) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("JSON Error: {}", err);
                            exit(EXIT_JSON_ERROR);
                        }
                    }
                },
                Some(OutputFormat::Yaml) | None => {
                    is_json = false;
                    let value: serde_json::Value = match serde_json::from_str(json) {
                        Ok(value) => value,
                        Err(err) => {
                            eprintln!("JSON Error: {}", err);
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    match serde_yaml::to_string(&value) {
                        Ok(yaml) => yaml,
                        Err(err) => {
                            eprintln!("YAML Error: {}", err);
                            exit(EXIT_JSON_ERROR);
                        }
                    }
                }
            };

            let ps = SyntaxSet::load_defaults_newlines();
            let ts = ThemeSet::load_defaults();
            let syntax = match is_json {
                true => ps.find_syntax_by_extension("json").unwrap(),
                false => ps.find_syntax_by_extension("yaml").unwrap(),
            };
    
            let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    
            for line in LinesWithEndings::from(&output) {
                let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                print!("{}", escaped);
            }
        },
        false => {
            println!("{}", json);
        }
    };
}

fn get_resource(dsc: &mut DscManager, resource: &str) -> DscResource {
    // check if resource is JSON or just a name
    match serde_json::from_str(resource) {
        Ok(resource) => resource,
        Err(err) => {
            if resource.contains('{') {
                eprintln!("Not valid resource JSON: {}\nInput was: {}", err, resource);
                exit(EXIT_INVALID_ARGS);
            }

            match dsc.initialize_discovery() {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("Error: {}", err);
                    exit(EXIT_DSC_ERROR);
                }
            };
            let resources: Vec<DscResource> = dsc.find_resource(resource).collect();
            match resources.len() {
                0 => {
                    eprintln!("Error: Resource not found");
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
                            eprintln!("Error: Cannot convert YAML to JSON: {}", err);
                            exit(EXIT_INVALID_ARGS);
                        }
                    }
                },
                Err(err) => {
                    if input.contains('{') {
                        eprintln!("Error: Input is not valid JSON: {}", json_err);
                    }
                    else {
                        eprintln!("Error: Input is not valid YAML: {}", err);
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
            match event {
                event::Event::Key(key) => {
                    // workaround bug in 0.26+ https://github.com/crossterm-rs/crossterm/issues/752#issuecomment-1414909095
                    if key.kind == event::KeyEventKind::Press {
                        break;
                    }
                }
                _ => {
                    eprintln!("Unexpected event: {:?}", event);
                    continue;
                }
            }
        }
    }
}
