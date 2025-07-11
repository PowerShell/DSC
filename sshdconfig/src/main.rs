// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser};
use rust_i18n::i18n;
use schemars::schema_for;
use std::process::exit;
use tracing::{debug, error};

use args::{Args, Command, DefaultShell, Setting};
use export::invoke_export;
use get::invoke_get;
use parser::SshdConfigParser;
use set::invoke_set;
use util::{enable_tracing, extract_sshd_defaults};

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

    let test = extract_sshd_defaults();
    println!("Extracted defaults: {:?}", test);

    let result = match &args.command {
        Command::Export => {
            debug!("Export command");
            match invoke_export() {
                Ok(output) => {
                    println!("{:?}", serde_json::to_string(&output));
                    Ok(())
                },
                Err(e) => Err(e),
            }
        },
        Command::Get { input, setting } => {
            debug!("Get command: setting={:?}", setting);
            invoke_get(input.as_ref(), setting)
        },
        Command::Schema { setting } => {
            debug!("Schema command: setting={:?}", setting);
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
            debug!("Set command: input={}", input);
            invoke_set(input)
        },
    };

    if let Err(e) = result {
        error!("{e}");
        exit(EXIT_FAILURE);
    }

    exit(EXIT_SUCCESS);
}
