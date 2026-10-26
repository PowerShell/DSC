// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
mod bridge;
mod model;

use clap::{Parser, Subcommand};
use rust_i18n::t;
use serde_json::{Value, json};
use std::process::ExitCode;

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: u8 = 0;
const EXIT_INVALID_ARGS: u8 = 1;
const EXIT_INVALID_INPUT: u8 = 2;
const EXIT_RESOURCE_ERROR: u8 = 3;

#[derive(Parser)]
#[command(about = t!("main.about").to_string(), version)]
struct Args {
    #[command(subcommand)]
    operation: Operation,
}

#[derive(Subcommand)]
enum Operation {
    #[command(about = t!("main.list").to_string())]
    List,
    #[command(about = t!("main.get").to_string())]
    Get(ResourceArgs),
    #[command(about = t!("main.set").to_string())]
    Set(ResourceArgs),
    #[command(about = t!("main.test").to_string())]
    Test(ResourceArgs),
    #[command(about = t!("main.export").to_string())]
    Export {
        #[arg(long, help = t!("main.resourceType").to_string())]
        resource_type: String,
        #[arg(long, help = t!("main.input").to_string())]
        input: Option<String>,
    },
}

#[derive(clap::Args)]
struct ResourceArgs {
    #[arg(long, help = t!("main.resourceType").to_string())]
    resource_type: String,
    #[arg(long, help = t!("main.input").to_string())]
    input: String,
}

fn write_error(message: &str) {
    eprintln!("{}", json!({"error": message}));
}

fn print_json(value: &Value) -> Result<(), model::AdapterError> {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    serde_json::to_writer(&mut output, value)
        .map_err(|error| model::AdapterError::Resource(error.to_string()))?;
    writeln!(output).map_err(|error| model::AdapterError::Resource(error.to_string()))
}

fn main() -> ExitCode {
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(error) if error.use_stderr() => {
            write_error(&error.to_string());
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
        Err(error) => {
            print!("{error}");
            return ExitCode::from(EXIT_SUCCESS);
        }
    };
    match run(args.operation) {
        Ok(()) => ExitCode::from(EXIT_SUCCESS),
        Err(error) => {
            write_error(&error.to_string());
            ExitCode::from(match error {
                model::AdapterError::Input(_) => EXIT_INVALID_INPUT,
                model::AdapterError::Resource(_) => EXIT_RESOURCE_ERROR,
            })
        }
    }
}

#[cfg(not(windows))]
fn run(_operation: Operation) -> Result<(), model::AdapterError> {
    Err(model::AdapterError::Resource(
        t!("main.windowsOnly").to_string(),
    ))
}

#[cfg(windows)]
fn run(operation: Operation) -> Result<(), model::AdapterError> {
    let bridge = bridge::Bridge::new()?;
    match operation {
        Operation::List => {
            for resource in bridge.list()? {
                print_json(&resource)?;
            }
        }
        Operation::Get(args) => invoke_instance(&bridge, "get", &args)?,
        Operation::Set(args) => invoke_instance(&bridge, "set", &args)?,
        Operation::Test(args) => invoke_instance(&bridge, "test", &args)?,
        Operation::Export {
            resource_type,
            input,
        } => {
            let class = bridge.class(&resource_type)?;
            let input = class.parse_input(input.as_deref().unwrap_or("{}"), true)?;
            bridge::Bridge::check_context(&class)?;
            for state in bridge.export(&class, &input)? {
                print_json(&Value::Object(state))?;
            }
        }
    }
    Ok(())
}

#[cfg(windows)]
fn invoke_instance(
    bridge: &bridge::Bridge,
    operation: &str,
    args: &ResourceArgs,
) -> Result<(), model::AdapterError> {
    let class = bridge.class(&args.resource_type)?;
    let input = class.parse_input(&args.input, false)?;
    class.require_keys(&input)?;
    bridge::Bridge::check_context(&class)?;
    let mut actual = if operation == "set" {
        bridge.set(&class, &input)?
    } else {
        bridge.get(&class, &input)?
    };
    if operation == "test" {
        let desired = bridge::normalize_input(&class, &input)?;
        let diff = model::diff(&desired, &actual);
        actual.insert("_inDesiredState".to_string(), Value::Bool(diff.is_empty()));
        print_json(&Value::Object(actual))?;
        print_json(&json!(diff))
    } else {
        print_json(&Value::Object(actual))
    }
}
