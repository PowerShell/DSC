// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(debug_assertions)]
use crossterm::event;
#[cfg(debug_assertions)]
use std::env;

use args::Arguments;
use clap::Parser;
use dsc_lib_registry::{config::{Registry, RegistryKey, RegistryList}, RegistryHelper};
use rust_i18n::t;
use schemars::schema_for;
use std::process::exit;
use tracing::{debug, error};
use tracing_subscriber::{filter::LevelFilter, prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Layer};

fn parse_input(input: &str) -> Result<(Vec<RegistryKey>, bool), String> {
    match serde_json::from_str::<Registry>(input) {
        Ok(Registry::Single(rk)) => Ok((vec![rk], true)),
        Ok(Registry::List(rl)) => {
            if rl.registry_keys.is_empty() {
                Err(t!("main.emptyRegistryKeysArray").to_string())
            } else {
                Ok((rl.registry_keys, false))
            }
        },
        Err(e) => Err(e.to_string()),
    }
}

fn emit_json<T: serde::Serialize>(value: &T) {
    match serde_json::to_string(value) {
        Ok(json) => println!("{json}"),
        Err(err) => {
            error!("{}", t!("main.jsonSerializationError", err = err));
            exit(EXIT_JSON_SERIALIZATION);
        }
    }
}

fn emit_results(results: Vec<RegistryKey>, was_single: bool) {
    if was_single {
        if let Some(rk) = results.into_iter().next() {
            emit_json(&rk);
        }
    } else {
        let list = RegistryList {
            registry_keys: results,
            metadata: None,
        };
        emit_json(&list);
    }
}

fn make_helper(rk: RegistryKey, what_if: bool) -> RegistryHelper {
    let mut helper = match RegistryHelper::new_from_key(rk) {
        Ok(h) => h,
        Err(err) => {
            error!("{err}");
            exit(EXIT_INVALID_INPUT);
        }
    };
    if what_if { helper.enable_what_if(); }
    helper
}

mod args;

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_INPUT: i32 = 2;
const EXIT_REGISTRY_ERROR: i32 = 3;
const EXIT_JSON_SERIALIZATION: i32 = 4;

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
                    let (items, was_single) = match parse_input(&input) {
                        Ok(v) => v,
                        Err(err) => { error!("{err}"); exit(EXIT_INVALID_INPUT); }
                    };
                    let mut results: Vec<RegistryKey> = Vec::with_capacity(items.len());
                    for rk in items {
                        let reg_helper = make_helper(rk, false);
                        match reg_helper.get() {
                            Ok(out) => results.push(out),
                            Err(err) => { error!("{err}"); exit(EXIT_REGISTRY_ERROR); }
                        }
                    }
                    emit_results(results, was_single);
                },
                args::ConfigSubCommand::Set{input, what_if} => {
                    debug!("Set input: {input}, what_if: {what_if}");
                    let (items, was_single) = match parse_input(&input) {
                        Ok(v) => v,
                        Err(err) => { error!("{err}"); exit(EXIT_INVALID_INPUT); }
                    };
                    let mut results: Vec<RegistryKey> = Vec::new();
                    for rk in items {
                        // In what-if, if the desired state is _exist: false, route to delete
                        let route_to_delete = what_if && matches!(rk.exist, Some(false));
                        let reg_helper = make_helper(rk, what_if);
                        let outcome = if route_to_delete {
                            reg_helper.remove()
                        } else {
                            reg_helper.set()
                        };
                        match outcome {
                            Ok(Some(out)) => results.push(out),
                            Ok(None) => {},
                            Err(err) => { error!("{err}"); exit(EXIT_REGISTRY_ERROR); }
                        }
                    }
                    if was_single {
                        if let Some(rk) = results.into_iter().next() {
                            emit_json(&rk);
                        }
                    } else if !results.is_empty() {
                        emit_results(results, false);
                    }
                },
                args::ConfigSubCommand::Delete{input, what_if} => {
                    debug!("Delete input: {input}, what_if: {what_if}");
                    let (items, was_single) = match parse_input(&input) {
                        Ok(v) => v,
                        Err(err) => { error!("{err}"); exit(EXIT_INVALID_INPUT); }
                    };
                    let mut results: Vec<RegistryKey> = Vec::new();
                    for rk in items {
                        let reg_helper = make_helper(rk, what_if);
                        match reg_helper.remove() {
                            Ok(Some(out)) => results.push(out),
                            Ok(None) => {},
                            Err(err) => { error!("{err}"); exit(EXIT_REGISTRY_ERROR); }
                        }
                    }
                    if was_single {
                        if let Some(rk) = results.into_iter().next() {
                            emit_json(&rk);
                        }
                    } else if !results.is_empty() {
                        emit_results(results, false);
                    }
                },
            }
        },
        args::SubCommand::Schema => {
            let schema = schema_for!(Registry);
            emit_json(&schema);
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
            }
        }
    }
}
