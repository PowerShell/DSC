// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(debug_assertions)]
use crossterm::event;
#[cfg(debug_assertions)]
use std::env;

use args::Arguments;
use atty::Stream;
use clap::Parser;
use schemars::schema_for;
use std::{io::{self, Read}, process::exit};

use crate::config::Registry;

mod args;
#[cfg(onecore)]
mod bcrypt;
mod config;
mod regconfighelper;

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_PARAMETER: i32 = 1;
const EXIT_INVALID_INPUT: i32 = 2;
const EXIT_REGISTRY_ERROR: i32 = 3;
const EXIT_JSON_SERIALIZATION_FAILED: i32 = 4;

#[allow(clippy::too_many_lines)]
fn main() {
    #[cfg(debug_assertions)]
    check_debug();

    let args = Arguments::parse();
    let input: Option<String> = if atty::is(Stream::Stdin) {
        None
    } else {
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        let input = match String::from_utf8(buffer) {
            Ok(input) => input,
            Err(e) => {
                eprintln!("Invalid UTF-8 sequence: {e}");
                exit(EXIT_INVALID_INPUT);
            }
        };
        Some(input)
    };

    let mut config: Registry = Registry::default();
    // check if input is valid for subcommand
    match args.subcommand {
        args::SubCommand::Config { subcommand: _ } => {
            if let Some(input) = input {
                    config = match serde_json::from_str(&input) {
                        Ok(config) => config,
                        Err(err) => {
                            eprintln!("Error JSON does not match schema: {err}");
                            exit(EXIT_INVALID_INPUT);
                        }
                    };
            } else {
                eprintln!("Error: Input JSON via STDIN is required for config subcommand.");
                exit(EXIT_INVALID_PARAMETER);
            }
        }
        _ => {
            if input.is_some() && !input.as_ref().unwrap().is_empty() {
                eprintln!("Error: Input JSON via STDIN is only valid for config subcommand: '{}'", input.unwrap());
                exit(EXIT_INVALID_INPUT);
            }
        }
    }

    match args.subcommand {
        args::SubCommand::Query { key_path, value_name, recurse } => {
            eprintln!("Get key_path: {key_path}, value_name: {value_name:?}, recurse: {recurse}");
        },
        args::SubCommand::Set { key_path, value } => {
            eprintln!("Set key_path: {key_path}, value: {value}");
        },
        args::SubCommand::Test => {
            eprintln!("Test");
        },
        args::SubCommand::Remove { key_path, value_name, recurse } => {
            eprintln!("Remove key_path: {key_path}, value_name: {value_name:?}, recurse: {recurse}");
        },
        args::SubCommand::Find { key_path, find, recurse, keys_only, values_only } => {
            eprintln!("Find key_path: {key_path}, find: {find}, recurse: {recurse:?}, keys_only: {keys_only:?}, values_only: {values_only:?}");
        },
        args::SubCommand::Config { subcommand } => {
            let json: String;
            match regconfighelper::validate_config(&config) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("Error validating config: {err}");
                    exit(EXIT_INVALID_INPUT);
                }
            }

            if config.exist.is_none() {
                config.exist = Some(true);
            }

            match subcommand {
                args::ConfigSubCommand::Get => {
                    match regconfighelper::config_get(&config) {
                        Ok(config) => {
                            json = config;
                        },
                        Err(err) => {
                            eprintln!("Error getting config: {err}");
                            exit(EXIT_REGISTRY_ERROR);
                        }
                    }
                },
                args::ConfigSubCommand::Set => {
                    match regconfighelper::config_set(&config) {
                        Ok(result) => {
                            json = result;
                        },
                        Err(err) => {
                            eprintln!("Error setting config: {err}");
                            exit(EXIT_REGISTRY_ERROR);
                        }
                    }
                },
                args::ConfigSubCommand::Test => {
                    match regconfighelper::config_test(&config) {
                        Ok(result) => {
                            json = result;
                        },
                        Err(err) => {
                            eprintln!("Error testing config: {err}");
                            exit(EXIT_REGISTRY_ERROR);
                        }
                    }
                },
            }

            if json.is_empty() {
                exit(EXIT_JSON_SERIALIZATION_FAILED);
            }

            println!("{json}");
        },
        args::SubCommand::Schema { pretty } => {
            let schema = schema_for!(Registry);
            let json = if pretty {
                serde_json::to_string_pretty(&schema).unwrap()
            }
            else {
                serde_json::to_string(&schema).unwrap()
            };
            println!("{json}");
        },
    }

    exit(EXIT_SUCCESS);
}

#[cfg(debug_assertions)]
fn check_debug() {
    if env::var("DEBUG_REGISTRY").is_ok() {
        eprintln!("attach debugger to pid {} and press any key to continue", std::process::id());
        loop {
            let event = event::read().unwrap();
            if let event::Event::Key(_key) = event {
                break;
            }
            eprintln!("Unexpected event: {event:?}");
        }
    }
}
