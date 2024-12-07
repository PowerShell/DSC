// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod args;
mod echo;

use args::Args;
use clap::Parser;
use rust_i18n::{i18n, t};
use schemars::schema_for;
use crate::echo::Echo;

i18n!("locales", fallback = "en-us");

fn main() {
    let args = Args::parse();
    match args.input {
        Some(input) => {
            let echo = match serde_json::from_str::<Echo>(&input) {
                Ok(echo) => echo,
                Err(err) => {
                    eprintln!("{}: {err}", t!("main.invalidJson"));
                    std::process::exit(1);
                }
            };
            let json = serde_json::to_string(&echo).unwrap();
            println!("{json}");
            return;
        },
        None => {
            eprintln!("{}", t!("main.noInput"));
        }
    }

    let schema = schema_for!(Echo);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    println!("{json}");
}
