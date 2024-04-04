// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod args;
mod delete;
mod echo;
mod exist;
mod sleep;

use args::{Args, Schemas, SubCommand};
use clap::Parser;
use schemars::schema_for;
use crate::delete::Delete;
use crate::echo::Echo;
use crate::exist::{Exist, State};
use crate::sleep::Sleep;
use std::{thread, time::Duration};

fn main() {
    let args = Args::parse();
    let json = match args.subcommand {
        SubCommand::Delete { input } => {
            let mut delete = match serde_json::from_str::<Delete>(&input) {
                Ok(delete) => delete,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            delete.delete_called = Some(true);
            serde_json::to_string(&delete).unwrap()
        },
        SubCommand::Echo { input } => {
            let echo = match serde_json::from_str::<Echo>(&input) {
                Ok(echo) => echo,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            serde_json::to_string(&echo).unwrap()
        },
        SubCommand::Exist { input } => {
            let mut exist = match serde_json::from_str::<Exist>(&input) {
                Ok(exist) => exist,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            if exist.exist {
                exist.state = Some(State::Present);
            } else {
                exist.state = Some(State::Absent);
            }

            serde_json::to_string(&exist).unwrap()
        },
        SubCommand::Schema { subcommand } => {
            let schema = match subcommand {
                Schemas::Delete => {
                    schema_for!(Delete)
                },
                Schemas::Echo => {
                    schema_for!(Echo)
                },
                Schemas::Exist => {
                    schema_for!(Exist)
                },
                Schemas::Sleep => {
                    schema_for!(Sleep)
                },
            };
            serde_json::to_string(&schema).unwrap()
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
