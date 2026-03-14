// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod types;

#[cfg(windows)]
mod service;

use rust_i18n::t;
use std::process::exit;

use types::WindowsService;

/// Write a JSON error object to stderr: `{"error":"<message>"}`
fn write_error(message: &str) {
    eprintln!("{}", serde_json::json!({"error": message}));
}

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_ARGS: i32 = 1;
const EXIT_INVALID_INPUT: i32 = 2;
const EXIT_SERVICE_ERROR: i32 = 3;

/// Deserialize the required JSON input into a `WindowsService`, or exit with an error.
fn require_input(input_json: Option<String>) -> WindowsService {
    let json = match input_json {
        Some(j) => j,
        None => {
            write_error(&t!("main.missingInput"));
            exit(EXIT_INVALID_ARGS);
        }
    };
    match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(e) => {
            write_error(&t!("main.invalidJson", error = e.to_string()));
            exit(EXIT_INVALID_INPUT);
        }
    }
}

/// Serialize a value to JSON and print it to stdout, or exit with an error.
fn print_json(value: &impl serde::Serialize) {
    match serde_json::to_string(value) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            write_error(&t!("main.invalidJson", error = e.to_string()));
            exit(EXIT_SERVICE_ERROR);
        }
    }
}

#[cfg(not(windows))]
fn main() {
    write_error(&t!("main.windowsOnly"));
    exit(EXIT_SERVICE_ERROR);
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

    match operation {
        "get" => {
            let input = require_input(input_json);

            match service::get_service(&input) {
                Ok(result) => {
                    print_json(&result);
                    exit(EXIT_SUCCESS);
                }
                Err(e) => {
                    write_error(&e.to_string());
                    exit(EXIT_SERVICE_ERROR);
                }
            }
        }
        "set" => {
            let input = require_input(input_json);

            match service::set_service(&input) {
                Ok(result) => {
                    print_json(&result);
                    exit(EXIT_SUCCESS);
                }
                Err(e) => {
                    write_error(&e.to_string());
                    exit(EXIT_SERVICE_ERROR);
                }
            }
        }
        "export" => {
            let filter: Option<WindowsService> = match input_json {
                Some(json) => match serde_json::from_str(&json) {
                    Ok(s) => Some(s),
                    Err(e) => {
                        write_error(&t!("main.invalidJson", error = e.to_string()));
                        exit(EXIT_INVALID_INPUT);
                    }
                },
                None => None,
            };

            match service::export_services(filter.as_ref()) {
                Ok(services) => {
                    for svc in &services {
                        print_json(svc);
                    }
                    exit(EXIT_SUCCESS);
                }
                Err(e) => {
                    write_error(&e.to_string());
                    exit(EXIT_SERVICE_ERROR);
                }
            }
        }
        _ => {
            write_error(&t!("main.unknownOperation", operation = operation));
            exit(EXIT_INVALID_ARGS);
        }
    }
}

/// Parse the `--input <json>` argument from the command-line args.
fn parse_input_arg(args: &[String]) -> Option<String> {
    let mut i = 2; // skip binary name and operation
    while i < args.len() {
        if args[i] == "--input" || args[i] == "-i" {
            if i + 1 < args.len() {
                return Some(args[i + 1].clone());
            }
            write_error(&t!("main.missingInputValue"));
            exit(EXIT_INVALID_ARGS);
        }
        i += 1;
    }
    None
}
