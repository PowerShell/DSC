// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser};
use rust_i18n::i18n;
use schemars::schema_for;

use args::{Args, Command, DefaultShell, Resource};
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

i18n!("locales", fallback = "en-us");

fn main() {
    let args = Args::parse();

    let result = match &args.command {
        Command::Export => invoke_export(),
        Command::Get { resource } => invoke_get(resource),
        Command::Set { input } => invoke_set(input),
        Command::Schema { resource } => {
            let schema = match resource {
                Resource::DefaultShell => {
                    schema_for!(DefaultShell)
                }
                Resource::SshdConfig => {
                    schema_for!(SshdConfigParser)
                }
            };
            println!("{}", serde_json::to_string(&schema).unwrap());
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
