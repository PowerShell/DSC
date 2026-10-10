// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod file;
mod types;

use crate::file::{get, set, test};
use crate::types::FileContent;
use rust_i18n::t;
use serde::Serialize;
use serde_json::json;
use std::env;
use std::process::ExitCode;

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: u8 = 0;
const EXIT_INVALID_ARGS: u8 = 1;
const EXIT_INVALID_INPUT: u8 = 2;
const EXIT_RESOURCE_ERROR: u8 = 3;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(operation) = args.first() else {
        return fail(EXIT_INVALID_ARGS, &t!("main.missingOperation"));
    };

    let input = match parse_input_arg(&args[1..]) {
        Ok(input) => input,
        Err(code) => return code,
    };
    match operation.as_str() {
        "get" | "set" | "test" => {
            let input = match require_input(input) {
                Ok(input) => input,
                Err(code) => return code,
            };
            match operation.as_str() {
                "get" => handle_result(get(&input)),
                "set" => handle_result(set(&input)),
                "test" => handle_result(test(&input)),
                _ => unreachable!(),
            }
        }
        _ => fail(
            EXIT_INVALID_ARGS,
            &t!("main.unknownOperation", operation = operation),
        ),
    }
}

fn parse_input_arg(args: &[String]) -> Result<Option<&str>, ExitCode> {
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--input" || args[index] == "-i" {
            let Some(input) = args.get(index + 1) else {
                return Err(fail(EXIT_INVALID_ARGS, &t!("main.missingInputValue")));
            };
            return Ok(Some(input));
        }
        index += 1;
    }
    Ok(None)
}

fn require_input(input: Option<&str>) -> Result<FileContent, ExitCode> {
    let Some(input) = input else {
        return Err(fail(EXIT_INVALID_ARGS, &t!("main.missingInput")));
    };

    serde_json::from_str(input).map_err(|error| {
        fail(
            EXIT_INVALID_INPUT,
            &t!("main.invalidJson", error = error.to_string()),
        )
    })
}

fn handle_result<T: Serialize>(result: Result<T, String>) -> ExitCode {
    let value = match result {
        Ok(value) => value,
        Err(error) => return fail(EXIT_RESOURCE_ERROR, &error),
    };

    match serde_json::to_string(&value) {
        Ok(json) => {
            println!("{json}");
            ExitCode::from(EXIT_SUCCESS)
        }
        Err(error) => fail(
            EXIT_RESOURCE_ERROR,
            &t!("main.serializeError", error = error.to_string()),
        ),
    }
}

fn fail(exit_code: u8, message: &str) -> ExitCode {
    eprintln!("{}", json!({ "error": message }));
    ExitCode::from(exit_code)
}
