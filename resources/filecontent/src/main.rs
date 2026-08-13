// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod file;
mod types;

use crate::file::{export, get, set, test};
use crate::types::FileContent;
use rust_i18n::t;
use serde::Serialize;
use serde_json::json;
use std::env;
use std::process::exit;

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_ARGS: i32 = 1;
const EXIT_INVALID_INPUT: i32 = 2;
const EXIT_RESOURCE_ERROR: i32 = 3;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(operation) = args.first() else {
        fail(EXIT_INVALID_ARGS, &t!("main.missingOperation"));
    };

    let input = parse_input_arg(&args[1..]);
    match operation.as_str() {
        "get" => handle_result(get(&require_input(input))),
        "set" => handle_result(set(&require_input(input))),
        "test" => handle_result(test(&require_input(input))),
        "export" => handle_result(export(&require_input(input))),
        _ => fail(
            EXIT_INVALID_ARGS,
            &t!("main.unknownOperation", operation = operation),
        ),
    }
}

fn parse_input_arg(args: &[String]) -> Option<&str> {
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--input" {
            return args.get(index + 1).map(String::as_str);
        }
        index += 1;
    }
    None
}

fn require_input(input: Option<&str>) -> FileContent {
    let Some(input) = input else {
        fail(EXIT_INVALID_ARGS, &t!("main.missingInput"));
    };

    serde_json::from_str(input).unwrap_or_else(|error| {
        fail(
            EXIT_INVALID_INPUT,
            &t!("main.invalidJson", error = error.to_string()),
        )
    })
}

fn handle_result<T: Serialize>(result: Result<T, String>) -> ! {
    let value = match result {
        Ok(value) => value,
        Err(error) => fail(EXIT_RESOURCE_ERROR, &error),
    };

    match serde_json::to_string(&value) {
        Ok(json) => {
            println!("{json}");
            exit(EXIT_SUCCESS);
        }
        Err(error) => fail(
            EXIT_RESOURCE_ERROR,
            &t!("main.serializeError", error = error.to_string()),
        ),
    }
}

fn fail(exit_code: i32, message: &str) -> ! {
    eprintln!("{}", json!({ "error": message }));
    exit(exit_code);
}
