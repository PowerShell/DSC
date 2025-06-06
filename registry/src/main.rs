// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(debug_assertions)]
use crossterm::event;
#[cfg(debug_assertions)]
use std::env;

// Init translations
use rust_i18n::t;
rust_i18n::i18n!("locales", fallback = "en-us");

use args::Arguments;
use clap::Parser;
use registry_helper::RegistryHelper;
use schemars::schema_for;
use std::process::exit;
use tracing::{debug, error};
use tracing_subscriber::{filter::LevelFilter, prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Layer};
use crate::config::Registry;

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

    enable_tracing();

    let args = Arguments::parse();
    match args.subcommand {
        args::SubCommand::Query { key_path, value_name, recurse } => {
            debug!("Get key_path: {key_path}, value_name: {value_name:?}, recurse: {recurse}");
        },
        args::SubCommand::Set { key_path, value } => {
            debug!("Set key_path: {key_path}, value: {value}");
        },
        args::SubCommand::Remove { key_path, value_name, recurse } => {
            debug!("Remove key_path: {key_path}, value_name: {value_name:?}, recurse: {recurse}");
        },
        args::SubCommand::Find { key_path, find, recurse, keys_only, values_only } => {
            debug!("Find key_path: {key_path}, find: {find}, recurse: {recurse:?}, keys_only: {keys_only:?}, values_only: {values_only:?}");
        },
        args::SubCommand::Config { subcommand } => {
            match subcommand {
                args::ConfigSubCommand::Get{input} => {
                    debug!("Get input: {input}");
                    let reg_helper = match RegistryHelper::new(&input) {
                        Ok(reg_helper) => reg_helper,
                        Err(err) => {
                            error!("{err}");
                            exit(EXIT_INVALID_INPUT);
                        }
                    };
                    match reg_helper.get() {
                        Ok(reg_config) => {
                            let json = serde_json::to_string(&reg_config).unwrap();
                            println!("{json}");
                        },
                        Err(err) => {
                            error!("{err}");
                            exit(EXIT_REGISTRY_ERROR);
                        }
                    }
                },
                args::ConfigSubCommand::Set{input, what_if} => {
                    debug!("Set input: {input}, what_if: {what_if}");
                    let mut reg_helper = match RegistryHelper::new(&input) {
                        Ok(reg_helper) => reg_helper,
                        Err(err) => {
                            error!("{err}");
                            exit(EXIT_INVALID_INPUT);
                        }
                    };
                    if what_if {
                        reg_helper.enable_what_if();
                    }
                    match reg_helper.set() {
                        Ok(reg_config) => {
                            if let Some(config) = reg_config {
                                let json = serde_json::to_string(&config).unwrap();
                                println!("{json}");
                            }
                        },
                        Err(err) => {
                            error!("{err}");
                            exit(EXIT_REGISTRY_ERROR);
                        }
                    }
                },
                args::ConfigSubCommand::Delete{input} => {
                    debug!("Delete input: {input}");
                    let reg_helper = match RegistryHelper::new(&input) {
                        Ok(reg_helper) => reg_helper,
                        Err(err) => {
                            error!("{err}");
                            exit(EXIT_INVALID_INPUT);
                        }
                    };
                    match reg_helper.remove() {
                        Ok(()) => {},
                        Err(err) => {
                            error!("{err}");
                            exit(EXIT_REGISTRY_ERROR);
                        }
                    }
                },
            }
        },
        args::SubCommand::Schema => {
            let schema = schema_for!(Registry);
            let json =serde_json::to_string(&schema).unwrap();
            println!("{json}");
        },
    }

    exit(EXIT_SUCCESS);
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
                continue;
            }
        }
    }
}
