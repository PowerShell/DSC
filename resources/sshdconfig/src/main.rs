// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser};
use rust_i18n::{i18n, t};
use schemars::schema_for;
use serde_json::Map;
use std::process::exit;
use tracing::{debug, error};

use args::{Args, Command, DefaultShell, Setting};
use export::invoke_export;
use get::{get_sshd_settings, invoke_get};
use parser::SshdConfigParser;
use repeat_keyword::{RepeatInput, RepeatListInput};
use set::invoke_set;
use util::{build_command_info, enable_tracing};

mod args;
mod canonical_properties;
mod error;
mod export;
mod formatter;
mod get;
mod inputs;
mod metadata;
mod parser;
mod repeat_keyword;
mod set;
mod util;

i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

fn main() {
    let args = Args::parse();

    enable_tracing(args.trace_level.as_ref(), &args.trace_format);

    let result = match &args.command {
        Command::Export { input, compare } => {
            debug!("{}: {:?}", t!("main.export").to_string(), input);
            invoke_export(input.as_ref(), *compare)
        },
        Command::Get { input, setting } => {
            invoke_get(input.as_ref(), setting)
        },
        Command::Schema { setting } => {
            debug!("{}; {:?}", t!("main.schema").to_string(), setting);
            let schema = match setting {
                Setting::SshdConfig => {
                    schema_for!(SshdConfigParser)
                },
                Setting::SshdConfigRepeat => {
                    schema_for!(RepeatInput)
                },
                Setting::SshdConfigRepeatList => {
                    schema_for!(RepeatListInput)
                },
                Setting::WindowsGlobal => {
                    schema_for!(DefaultShell)
                }
            };
            println!("{}", serde_json::to_string(&schema).unwrap());
            Ok(Map::new())
        },
        Command::Set { input, setting } => {
            debug!("{}", t!("main.set", input = input).to_string());
            invoke_set(input, setting)
        },
    };

    match result {
        Ok(output) => {
            if !output.is_empty() {
                match serde_json::to_string(&output) {
                    Ok(json) => println!("{json}"),
                    Err(e) => {
                        error!("{}", e);
                        exit(EXIT_FAILURE);
                    }
                }
            }
            exit(EXIT_SUCCESS);
        },
        Err(e) => {
            error!("{}", e);
            exit(EXIT_FAILURE);
        }
    }
}
