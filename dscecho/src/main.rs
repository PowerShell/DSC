// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod args;
mod echo;

use args::Args;
use clap::Parser;
use schemars::schema_for;
use crate::echo::Echo;

fn main() {
    let args = Args::parse();
    match args.input {
        Some(input) => {
            let echo = match serde_json::from_str::<Echo>(&input) {
                Ok(echo) => echo,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            let json = serde_json::to_string(&echo).unwrap();
            println!("{json}");
            return;
        },
        None => {
            eprintln!("No input provided.");
        }
    }

    let schema = schema_for!(Echo);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    println!("{json}");
}
