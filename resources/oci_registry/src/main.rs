// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod oci;
mod rpc;
mod settings;

use clap::{Parser, Subcommand};
use rust_i18n::t;
use serde_json::json;
use std::{
    io::{self, Write},
    process::ExitCode,
};

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_INVALID_ARGS: u8 = 1;
const EXIT_INVALID_INPUT: u8 = 2;
const EXIT_RESOURCE_ERROR: u8 = 3;

#[derive(Parser)]
#[command(about = t!("main.about").to_string())]
struct Args {
    #[command(subcommand)]
    operation: Operation,
}

#[derive(Subcommand)]
enum Operation {
    Discover,
    Schema,
    Get {
        #[arg(long, help = t!("main.input").to_string())]
        input: Option<String>,
    },
    Set {
        #[arg(long, help = t!("main.input").to_string())]
        input: String,
    },
    Export {
        #[arg(long, help = t!("main.input").to_string())]
        input: Option<String>,
    },
}

fn print_json(value: &impl serde::Serialize) -> Result<(), String> {
    let mut output = io::stdout().lock();
    serde_json::to_writer(&mut output, value).map_err(|error| error.to_string())?;
    writeln!(output).map_err(|error| error.to_string())
}

fn write_error(error: &str) {
    eprintln!("{}", json!({"error": error}));
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(error) => {
            let code = if error.use_stderr() {
                EXIT_INVALID_ARGS
            } else {
                0
            };
            let _ = error.print();
            return ExitCode::from(code);
        }
    };
    let result = match args.operation {
        Operation::Discover => rpc::serve(&mut io::stdin().lock(), &mut io::stdout().lock()).await,
        Operation::Schema => print_json(&schemars::schema_for!(settings::RegistryList)),
        operation => {
            let export = matches!(&operation, Operation::Export { .. });
            let (input, set) = match operation {
                Operation::Get { input } | Operation::Export { input } => (input, false),
                Operation::Set { input } => (Some(input), true),
                _ => unreachable!(),
            };
            let input: settings::RegistryList =
                match serde_json::from_str(input.as_deref().unwrap_or("{}")) {
                    Ok(input) => input,
                    Err(error) => {
                        write_error(&t!("main.invalidInput", error = error));
                        return ExitCode::from(EXIT_INVALID_INPUT);
                    }
                };
            if let Some(registries) = &input.registries
                && let Err(error) = (settings::Settings {
                    registries: registries.clone(),
                })
                .entries()
            {
                write_error(&t!("main.invalidInput", error = error));
                return ExitCode::from(EXIT_INVALID_INPUT);
            }
            if (input.policy && input.scope != settings::Scope::AllUsers)
                || (set && input.registries.is_none())
            {
                write_error(&t!(
                    "main.invalidInput",
                    error = if input.policy {
                        t!("settings.scope")
                    } else {
                        t!("settings.required")
                    }
                ));
                return ExitCode::from(EXIT_INVALID_INPUT);
            }
            match if set {
                settings::set(&input)
            } else {
                settings::get(&input)
            } {
                Ok(mut state) => {
                    if export {
                        state.name = Some(t!("main.exportName").to_string());
                    }
                    print_json(&state)
                }
                Err(error) => Err(error),
            }
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            write_error(&t!("main.failed", error = error));
            ExitCode::from(EXIT_RESOURCE_ERROR)
        }
    }
}
