// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(debug_assertions)]
use crossterm::event;
#[cfg(debug_assertions)]
use std::env;

use args::Arguments;
use clap::Parser;
use registry_helper::RegistryHelper;
use schemars::schema_for;
use std::process::exit;

use crate::config::RegistryConfig;

mod args;
pub mod config;
mod error;
mod registry_helper;

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_INPUT: i32 = 2;
const EXIT_REGISTRY_ERROR: i32 = 3;

#[allow(clippy::too_many_lines)]
fn main() {
    #[cfg(debug_assertions)]
    check_debug();

    let args = Arguments::parse();
    match args.subcommand {
        args::SubCommand::Query { key_path, value_name, recurse } => {
            eprintln!("Get key_path: {key_path}, value_name: {value_name:?}, recurse: {recurse}");
        },
        args::SubCommand::Set { key_path, value } => {
            eprintln!("Set key_path: {key_path}, value: {value}");
        },
        args::SubCommand::Remove { key_path, value_name, recurse } => {
            eprintln!("Remove key_path: {key_path}, value_name: {value_name:?}, recurse: {recurse}");
        },
        args::SubCommand::Find { key_path, find, recurse, keys_only, values_only } => {
            eprintln!("Find key_path: {key_path}, find: {find}, recurse: {recurse:?}, keys_only: {keys_only:?}, values_only: {values_only:?}");
        },
        args::SubCommand::Config { subcommand } => {
            match subcommand {
                args::ConfigSubCommand::Get{input} => {
                    let reg_helper = match RegistryHelper::new(&input) {
                        Ok(reg_helper) => reg_helper,
                        Err(err) => {
                            eprintln!("Error: {err}");
                            exit(EXIT_INVALID_INPUT);
                        }
                    };
                    match reg_helper.get() {
                        Ok(reg_config) => {
                            let json = serde_json::to_string(&reg_config).unwrap();
                            println!("{json}");
                        },
                        Err(err) => {
                            eprintln!("Error: {err}");
                            exit(EXIT_REGISTRY_ERROR);
                        }
                    }
                },
                args::ConfigSubCommand::Set{input} => {
                    let reg_helper = match RegistryHelper::new(&input) {
                        Ok(reg_helper) => reg_helper,
                        Err(err) => {
                            eprintln!("Error: {err}");
                            exit(EXIT_INVALID_INPUT);
                        }
                    };
                    match reg_helper.set() {
                        Ok(_) => {},
                        Err(err) => {
                            eprintln!("Error: {err}");
                            exit(EXIT_REGISTRY_ERROR);
                        }
                    }
                },
                args::ConfigSubCommand::Remove{input} => {
                    let reg_helper = match RegistryHelper::new(&input) {
                        Ok(reg_helper) => reg_helper,
                        Err(err) => {
                            eprintln!("Error: {err}");
                            exit(EXIT_INVALID_INPUT);
                        }
                    };
                    match reg_helper.remove() {
                        Ok(_) => {},
                        Err(err) => {
                            eprintln!("Error: {err}");
                            exit(EXIT_REGISTRY_ERROR);
                        }
                    }
                },
            }
        },
        args::SubCommand::Schema => {
            let schema = schema_for!(RegistryConfig);
            let json =serde_json::to_string(&schema).unwrap();
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
            let event = match event::read() {
                Ok(event) => event,
                Err(err) => {
                    eprintln!("Error: Failed to read event: {err}");
                    break;
                }
            };
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
