// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser};
use rust_i18n::{i18n, t};
use schemars::schema_for;
use std::process::exit;
use tracing::{debug, error};

use args::{Args, Command, DefaultShell, Setting};
use export::invoke_export;
use get::invoke_get;
use parser::SshdConfigParser;
use set::invoke_set;
use util::enable_tracing;

mod args;
mod error;
mod export;
mod get;
mod metadata;
mod parser;
mod set;
mod util;

i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

fn main() {
    enable_tracing();

    let args = Args::parse();

    let result = match &args.command {
        Command::Export => {
            debug!("{}", t!("main.export").to_string());
            invoke_export()
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
                Setting::WindowsGlobal => {
                    schema_for!(DefaultShell)
                }
            };
            println!("{}", serde_json::to_string(&schema).unwrap());
            Ok(())
        },
        Command::Set { input } => {
            debug!("{}", t!("main.set", input = input).to_string());
            invoke_set(input)
        },
    };

    if let Err(e) = result {
        error!("{e}");
        exit(EXIT_FAILURE);
    }

    exit(EXIT_SUCCESS);
}
