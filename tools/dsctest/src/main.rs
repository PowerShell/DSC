// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod args;
mod delete;
mod echo;
mod exist;
mod exit_code;
mod metadata;
mod sleep;
mod trace;
mod whatif;

use args::{Args, Schemas, SubCommand};
use clap::Parser;
use schemars::schema_for;
use crate::delete::Delete;
use crate::echo::Echo;
use crate::exist::{Exist, State};
use crate::exit_code::ExitCode;
use crate::metadata::Metadata;
use crate::sleep::Sleep;
use crate::trace::Trace;
use crate::whatif::WhatIf;
use std::{thread, time::Duration};

#[allow(clippy::too_many_lines)]
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
        SubCommand::ExitCode { input } => {
            let exit_code = match serde_json::from_str::<ExitCode>(&input) {
                Ok(exit_code) => exit_code,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            if exit_code.exit_code != 0 {
                eprintln!("Exiting with code: {}", exit_code.exit_code);
                std::process::exit(exit_code.exit_code);
            }
            input
        },
        SubCommand::Metadata { input } => {
            let metadata = match serde_json::from_str::<Metadata>(&input) {
                Ok(metadata) => metadata,
                Err(err) => {
                    eprintln!("Error JSON does not match schema: {err}");
                    std::process::exit(1);
                }
            };
            serde_json::to_string(&metadata).unwrap()
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
                Schemas::ExitCode => {
                    schema_for!(ExitCode)
                },
                Schemas::Metadata => {
                    schema_for!(Metadata)
                },
                Schemas::Sleep => {
                    schema_for!(Sleep)
                },
                Schemas::Trace => {
                    schema_for!(Trace)
                },
                Schemas::WhatIf => {
                    schema_for!(WhatIf)
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
        SubCommand::Trace => {
            // get level from DSC_TRACE_LEVEL env var
            let level = match std::env::var("DSC_TRACE_LEVEL") {
                Ok(level) => level,
                Err(_) => "warn".to_string(),
            };
            let trace = trace::Trace {
                level,
            };
            serde_json::to_string(&trace).unwrap()
        },
        SubCommand::WhatIf { what_if } => {
            let result: WhatIf = if what_if {
                WhatIf { execution_type: "WhatIf".to_string() }
            } else {
                WhatIf { execution_type: "Actual".to_string() }
            };
            serde_json::to_string(&result).unwrap()
        },
    };

    println!("{json}");
}
