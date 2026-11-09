// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod process_info;

use process_info::{ProcessInfo, TerminateInput, matching, snapshot, terminate};
use rust_i18n::t;
use serde::{Serialize, de::DeserializeOwned};
use std::{env, process::ExitCode};

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: u8 = 0;
const EXIT_INVALID_ARGS: u8 = 1;
const EXIT_INVALID_INPUT: u8 = 2;
const EXIT_RESOURCE_ERROR: u8 = 3;

fn write_error(message: &str) {
    eprintln!("{}", serde_json::json!({ "error": message }));
}

fn require_input<T: DeserializeOwned>(input: &str) -> Result<T, u8> {
    serde_json::from_str(input).map_err(|error| {
        write_error(&t!("input.invalidJson", error = error));
        EXIT_INVALID_INPUT
    })
}

fn print_json(value: &impl Serialize) -> Result<(), u8> {
    let json = serde_json::to_string(value).map_err(|error| {
        write_error(&t!("main.serializationFailed", error = error));
        EXIT_RESOURCE_ERROR
    })?;
    println!("{json}");
    Ok(())
}

fn invalid_input(message: &str) -> u8 {
    write_error(message);
    EXIT_INVALID_INPUT
}

fn resource_error(message: &str) -> u8 {
    write_error(message);
    EXIT_RESOURCE_ERROR
}

fn run() -> Result<(), u8> {
    let mut args = env::args().skip(1);
    let operation = args.next().ok_or_else(|| {
        write_error(&t!("main.usage"));
        EXIT_INVALID_ARGS
    })?;
    if !matches!(operation.as_str(), "get" | "export" | "terminate") {
        write_error(&t!("main.unknownOperation", operation = operation));
        return Err(EXIT_INVALID_ARGS);
    }
    let input = match (args.next(), args.next(), args.next()) {
        (None, None, None) if operation == "export" => "{}".to_string(),
        (Some(flag), Some(input), None) if flag == "--input" => input,
        _ => {
            write_error(&t!("main.usage"));
            return Err(EXIT_INVALID_ARGS);
        }
    };

    if operation == "terminate" {
        let input: TerminateInput = require_input(&input)?;
        input
            .validate()
            .map_err(|message| invalid_input(&message))?;
        let system = snapshot().map_err(|message| resource_error(&message))?;
        let terminated = terminate(&system, &input).map_err(|message| resource_error(&message))?;
        return print_json(&serde_json::json!({ "terminated": terminated }));
    }

    let input: ProcessInfo = require_input(&input)?;
    if operation == "get" {
        input.validate_identity()
    } else {
        input.validate()
    }
    .map_err(|message| invalid_input(&message))?;
    let system = snapshot().map_err(|message| resource_error(&message))?;
    let processes = matching(&system, &input).map_err(|message| resource_error(&message))?;
    if operation == "get" {
        if processes.len() != 1 {
            return Err(resource_error(&if processes.is_empty() {
                t!("process.notFound").to_string()
            } else {
                t!("process.ambiguous").to_string()
            }));
        }
        print_json(&processes[0])
    } else {
        for process in processes {
            print_json(&process)?;
        }
        Ok(())
    }
}

fn main() -> ExitCode {
    ExitCode::from(run().err().unwrap_or(EXIT_SUCCESS))
}
