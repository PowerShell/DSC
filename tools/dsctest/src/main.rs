// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod args;
mod sleep;

use args::{Args, Schemas, SubCommand};
use clap::Parser;
use schemars::schema_for;
use crate::sleep::Sleep;
use std::{thread, time::Duration};

fn main() {
    let args = Args::parse();
    let json = match args.subcommand {
        SubCommand::Schema { subcommand } => {
            match subcommand {
                Schemas::Sleep => {
                    let schema = schema_for!(Sleep);
                    serde_json::to_string(&schema).unwrap()
                },
            }
        },
        SubCommand::Sleep { input } => {
            let sleep = match serde_json::from_str::<Sleep>(&input) {
                Ok(sleep) => sleep,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            thread::sleep(Duration::from_secs(sleep.seconds));
            serde_json::to_string(&sleep).unwrap()
        },
    };

    println!("{json}");
}
