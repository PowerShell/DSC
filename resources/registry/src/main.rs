// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(debug_assertions)]
use crossterm::event;
#[cfg(debug_assertions)]
use std::env;

use adapter::{adapter_export, adapter_get, adapter_set};
use args::{AdapterSubCommand, Arguments, ConfigSubCommand, SubCommand};
use clap::Parser;
use dsc_lib_registry::{config::Registry, RegistryHelper};
use rust_i18n::t;
use schemars::schema_for;
use std::process::exit;
use tracing::{error, trace};
use tracing_subscriber::{filter::LevelFilter, prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Layer};
use types::RegistryList;

mod adapter;
mod args;
mod error;
mod types;

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_INPUT: i32 = 2;
const EXIT_REGISTRY_ERROR: i32 = 3;

#[allow(clippy::too_many_lines)]
fn main() {
    #[cfg(debug_assertions)]
    check_debug();

    enable_tracing();

    let args = Arguments::parse();
    match args.subcommand {
        SubCommand::Adapter { subcommand } => {
            let result = match subcommand {
                AdapterSubCommand::Get { input, adapted_resource } => {
                    adapter_get(&input, &adapted_resource)
                },
                AdapterSubCommand::Set { input, adapted_resource } => {
                    if let Err(e) = adapter_set(&input, &adapted_resource) {
                        error!("{e}");
                        exit(EXIT_REGISTRY_ERROR);
                    }
                    exit(EXIT_SUCCESS);
                },
                AdapterSubCommand::Export { input, adapted_resource } => {
                    adapter_export(&input, &adapted_resource)
                },
            };
            match result {
                Ok(output) => {
                    println!("{output}");
                },
                Err(err) => {
                    error!("{err}");
                    exit(EXIT_INVALID_INPUT);
                }
            }
        },
        SubCommand::Query { key_path, value_name, recurse } => {
            trace!("Get key_path: {key_path}, value_name: {value_name:?}, recurse: {recurse}");
        },
        SubCommand::Set { key_path, value } => {
            trace!("Set key_path: {key_path}, value: {value}");
        },
        SubCommand::Remove { key_path, value_name, recurse } => {
            trace!("Remove key_path: {key_path}, value_name: {value_name:?}, recurse: {recurse}");
        },
        SubCommand::Find { key_path, find, recurse, keys_only, values_only } => {
            trace!("Find key_path: {key_path}, find: {find}, recurse: {recurse:?}, keys_only: {keys_only:?}, values_only: {values_only:?}");
        },
        SubCommand::Config { subcommand } => {
            match subcommand {
                ConfigSubCommand::Get{input, list} => {
                    trace!("Get input: {input}");
                    let mut output = RegistryList { registry_entries: vec![] };
                    let reg_list = import_input(&input, list);
                    for reg in reg_list.registry_entries {
                        let reg_helper = match RegistryHelper::new_from_registry(&reg) {
                            Ok(helper) => helper,
                            Err(err) => {
                                error!("{err}");
                                exit(EXIT_INVALID_INPUT);
                            }
                        };
                        match reg_helper.get() {
                            Ok(reg_config) => {
                                if list {
                                    output.registry_entries.push(reg_config);
                                } else {
                                    let json = serde_json::to_string(&reg_config).unwrap();
                                    println!("{json}");
                                    exit(EXIT_SUCCESS);
                                }
                            },
                            Err(err) => {
                                error!("{err}");
                                exit(EXIT_REGISTRY_ERROR);
                            }
                        }
                    }
                    let json = serde_json::to_string(&output).unwrap();
                    println!("{json}");
                    exit(EXIT_SUCCESS);
                },
                ConfigSubCommand::Set{input, list, what_if} => {
                    trace!("Set input: {input}, what_if: {what_if}");
                    let mut output = RegistryList { registry_entries: vec![] };
                    let reg_list = import_input(&input, list);
                    for reg in reg_list.registry_entries {
                        let mut reg_helper = match RegistryHelper::new_from_registry(&reg) {
                            Ok(helper) => helper,
                            Err(err) => {
                                error!("{err}");
                                exit(EXIT_INVALID_INPUT);
                            }
                        };
                        if what_if { reg_helper.enable_what_if(); }
                        if let Some(exist) = reg.exist && !exist {
                                match reg_helper.remove() {
                                    Ok(Some(reg_config)) => {
                                        if what_if {
                                            if list {
                                                output.registry_entries.push(reg_config);
                                            } else {
                                                let json = serde_json::to_string(&reg_config).unwrap();
                                                println!("{json}");
                                                exit(EXIT_SUCCESS);
                                            }
                                        }
                                    },
                                    Ok(None) => {},
                                    Err(err) => {
                                        error!("{err}");
                                        exit(EXIT_REGISTRY_ERROR);
                                    }
                                }
                                continue;
                            }
                        match reg_helper.set() {
                            Ok(reg_config) => {
                                if what_if && let Some(config) = reg_config {
                                    if list {
                                        output.registry_entries.push(config);
                                    } else {
                                        let json = serde_json::to_string(&config).unwrap();
                                        println!("{json}");
                                        exit(EXIT_SUCCESS);
                                    }
                                }
                                if !list {
                                    exit(EXIT_SUCCESS);
                                }
                            },
                            Err(err) => {
                                error!("{err}");
                                exit(EXIT_REGISTRY_ERROR);
                            }
                        }
                    }
                    if what_if {
                        let json = serde_json::to_string(&output).unwrap();
                        println!("{json}");
                    }
                    exit(EXIT_SUCCESS);
                },
                ConfigSubCommand::Delete{input, what_if} => {
                    trace!("Delete input: {input}, what_if: {what_if}");
                    let mut reg_helper = match RegistryHelper::new_from_json(&input) {
                        Ok(reg_helper) => reg_helper,
                        Err(err) => {
                            error!("{err}");
                            exit(EXIT_INVALID_INPUT);
                        }
                    };
                    if what_if { reg_helper.enable_what_if(); }
                    match reg_helper.remove() {
                        Ok(Some(reg_config)) => {
                            let json = serde_json::to_string(&reg_config).unwrap();
                            println!("{json}");
                        },
                        Ok(None) => {},
                        Err(err) => {
                            error!("{err}");
                            exit(EXIT_REGISTRY_ERROR);
                        }
                    }
                },
            }
        },
        args::SubCommand::Schema{list} => {
            let schema = if list {
                schema_for!(RegistryList)
            } else {
                schema_for!(Registry)
            };
            let json =serde_json::to_string(&schema).unwrap();
            println!("{json}");
        },
    }

    exit(EXIT_SUCCESS);
}

fn import_input(input: &str, list: bool) -> RegistryList {
    if list {
        match serde_json::from_str::<RegistryList>(input) {
            Ok(reg_list) => reg_list,
            Err(err) => {
                error!("{err}");
                exit(EXIT_INVALID_INPUT);
            }
        }
    } else {
        match serde_json::from_str::<Registry>(input) {
            Ok(reg) => RegistryList { registry_entries: vec![reg] },
            Err(err) => {
                error!("{err}");
                exit(EXIT_INVALID_INPUT);
            }
        }
    }
}

pub fn enable_tracing() {
    // default filter to trace level
    let filter = EnvFilter::builder().with_default_directive(LevelFilter::TRACE.into()).parse("").unwrap_or_default();
    let layer = tracing_subscriber::fmt::Layer::default().with_writer(std::io::stderr);
    let fmt = layer
                .with_ansi(false)
                .with_level(true)
                .with_line_number(true)
                .json()
                .boxed();

    let subscriber = tracing_subscriber::Registry::default().with(fmt).with(filter);

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        eprintln!("{}", t!("main.tracingInitError"));
    }
}

#[cfg(debug_assertions)]
fn check_debug() {
    if env::var("DEBUG_REGISTRY").is_ok() {
        eprintln!("{}", t!("main.debugAttach", pid = std::process::id()));

        loop {
            let event = match event::read() {
                Ok(event) => event,
                Err(err) => {
                    eprintln!("{}", t!("main.debugEventReadError", "err" => err));
                    break;
                }
            };
            if let event::Event::Key(key) = event {
                // workaround bug in 0.26+ https://github.com/crossterm-rs/crossterm/issues/752#issuecomment-1414909095
                if key.kind == event::KeyEventKind::Press {
                    break;
                }
            } else {
                eprintln!("{}", t!("main.debugEventUnexpectedError", e = event : {:?}));
            }
        }
    }
}
