// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser};
use schemars::schema_for;

use args::{Args, Command, DefaultShell};
use export::invoke_export;
use get::invoke_get;
use parser::SshdConfigParser;
use set::invoke_set;

mod args;
mod error;
mod export;
mod get;
mod metadata;
mod parser;
mod set;
mod util;

fn main() {
    let args = Args::parse();

    let result = match &args.command {
        Command::Export => invoke_export(),
        Command::Get => invoke_get(),
        Command::Set { input } => invoke_set(input),
        Command::Schema { as_global } => {
            let schema = if *as_global {
                schema_for!(DefaultShell)
            } else {
                schema_for!(SshdConfigParser)
            };
            println!("{}", serde_json::to_string_pretty(&schema).unwrap());
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
