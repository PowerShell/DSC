// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use args::{Args, Command};
use clap::{Parser};
use export::invoke_export;
use get::invoke_get;
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
    };

    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
