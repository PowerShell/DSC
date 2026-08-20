// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod types;

#[cfg(windows)]
mod environment;

use rust_i18n::t;
use std::process::exit;
use types::{EnvironmentVariable, EnvironmentVariableList, Operation};

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_ARGS: i32 = 1;
const EXIT_INVALID_INPUT: i32 = 2;
const EXIT_RESOURCE_ERROR: i32 = 3;
const EXIT_ELEVATION_REQUIRED: i32 = 4;

fn write_error(message: &str) {
    eprintln!("{}", serde_json::json!({ "error": message }));
}

fn print_json(value: &impl serde::Serialize) {
    match serde_json::to_string(value) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            write_error(&t!("main.serializeError", error = error.to_string()));
            exit(EXIT_RESOURCE_ERROR);
        }
    }
}

fn require_input(
    input_json: Option<String>,
    operation: Operation,
    is_list: bool,
) -> EnvironmentVariableList {
    let Some(json) = input_json else {
        write_error(&t!("main.missingInput"));
        exit(EXIT_INVALID_ARGS);
    };

    let input = match if is_list {
        serde_json::from_str::<EnvironmentVariableList>(&json)
    } else {
        serde_json::from_str::<EnvironmentVariable>(&json).map(EnvironmentVariableList::from)
    } {
        Ok(value) => value,
        Err(error) => {
            write_error(&t!("main.invalidJson", error = error.to_string()));
            exit(EXIT_INVALID_INPUT);
        }
    };

    if let Err(error) = input.validate(operation) {
        write_error(&error);
        exit(EXIT_INVALID_INPUT);
    }

    input
}

fn print_result(mut value: EnvironmentVariableList, is_list: bool) {
    if is_list {
        print_json(&value);
        return;
    }

    let Some(variable) = value.environment_variables.pop() else {
        write_error(&t!("main.missingState"));
        exit(EXIT_RESOURCE_ERROR);
    };
    let mut output = match serde_json::to_value(variable) {
        Ok(value) => value,
        Err(error) => {
            write_error(&t!("main.serializeError", error = error.to_string()));
            exit(EXIT_RESOURCE_ERROR);
        }
    };
    if let Some(in_desired_state) = value.in_desired_state {
        output["_inDesiredState"] = serde_json::Value::Bool(in_desired_state);
    }
    print_json(&output);
}

#[cfg(not(windows))]
fn main() {
    write_error(&t!("main.windowsOnly"));
    exit(EXIT_RESOURCE_ERROR);
}

#[cfg(windows)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        write_error(&t!("main.missingOperation"));
        exit(EXIT_INVALID_ARGS);
    }

    let operation = args[1].as_str();
    let input_json = parse_input_arg(&args);
    let is_list = args.iter().any(|arg| arg == "--list");

    let result = match operation {
        "get" => environment::get_variables(&require_input(input_json, Operation::Get, is_list)),
        "set" => environment::set_variables(&require_input(input_json, Operation::Set, is_list)),
        "test" => environment::test_variables(&require_input(input_json, Operation::Test, is_list)),
        _ => {
            write_error(&t!("main.unknownOperation", operation = operation));
            exit(EXIT_INVALID_ARGS);
        }
    };

    match result {
        Ok(value) => {
            print_result(value, is_list);
            exit(EXIT_SUCCESS);
        }
        Err(error) => {
            write_error(&error.to_string());
            exit(if error.is_elevation_required() {
                EXIT_ELEVATION_REQUIRED
            } else {
                EXIT_RESOURCE_ERROR
            });
        }
    }
}

fn parse_input_arg(args: &[String]) -> Option<String> {
    let mut index = 2;
    while index < args.len() {
        if args[index] == "--input" || args[index] == "-i" {
            if index + 1 < args.len() {
                return Some(args[index + 1].clone());
            }
            write_error(&t!("main.missingInputValue"));
            exit(EXIT_INVALID_ARGS);
        }
        index += 1;
    }
    None
}
