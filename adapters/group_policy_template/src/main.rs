// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
mod admx;
#[cfg(windows)]
mod registry;

use rust_i18n::t;
use serde_json::json;
use std::process::exit;

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_ARGS: i32 = 1;
const EXIT_INVALID_INPUT: i32 = 2;
const EXIT_RESOURCE_ERROR: i32 = 3;

fn write_error(message: &str) {
    eprintln!("{}", json!({ "error": message }));
}

#[cfg(not(windows))]
fn main() {
    write_error(&t!("main.windowsOnly"));
    exit(EXIT_RESOURCE_ERROR);
}

#[cfg(windows)]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(operation) = args.first().map(String::as_str) else {
        write_error(&t!("main.missingOperation"));
        exit(EXIT_INVALID_ARGS);
    };

    let result = match operation {
        "list" => admx::list_resources(),
        "get" | "set" => {
            let Some(input) = argument_value(&args, "--input") else {
                write_error(&t!("main.missingArgument", argument = "--input"));
                exit(EXIT_INVALID_ARGS);
            };
            let Some(resource_type) = argument_value(&args, "--resource-type") else {
                write_error(&t!("main.missingArgument", argument = "--resource-type"));
                exit(EXIT_INVALID_ARGS);
            };
            let Some(resource_path) = argument_value(&args, "--resource-path") else {
                write_error(&t!("main.missingArgument", argument = "--resource-path"));
                exit(EXIT_INVALID_ARGS);
            };
            if operation == "get" {
                registry::get(input, resource_type, resource_path)
            } else {
                registry::set(input, resource_type, resource_path)
            }
        }
        unknown => {
            write_error(&t!("main.unknownOperation", operation = unknown));
            exit(EXIT_INVALID_ARGS);
        }
    };

    match result {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            exit(EXIT_SUCCESS);
        }
        Err(error) => {
            write_error(&error.to_string());
            let code = if error.is_input_error() {
                EXIT_INVALID_INPUT
            } else {
                EXIT_RESOURCE_ERROR
            };
            exit(code);
        }
    }
}

#[cfg(windows)]
fn argument_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}
