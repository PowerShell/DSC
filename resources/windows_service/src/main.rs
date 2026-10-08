// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod types;

#[cfg(windows)]
mod service;

use rust_i18n::t;
use std::process::ExitCode;

use types::WindowsService;

/// Write a JSON error object to stderr: `{"error":"<message>"}`
fn write_error(message: &str) {
    eprintln!("{}", serde_json::json!({"error": message}));
}

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: u8 = 0;
const EXIT_INVALID_ARGS: u8 = 1;
const EXIT_INVALID_INPUT: u8 = 2;
const EXIT_SERVICE_ERROR: u8 = 3;

/// Deserialize the required JSON input into a `WindowsService`, or exit with an error.
fn require_input(input_json: Option<String>) -> Result<WindowsService, ExitCode> {
    let json = match input_json {
        Some(j) => j,
        None => {
            write_error(&t!("main.missingInput"));
            return Err(ExitCode::from(EXIT_INVALID_ARGS));
        }
    };
    match serde_json::from_str(&json) {
        Ok(v) => Ok(v),
        Err(e) => {
            write_error(&t!("main.invalidJson", error = e.to_string()));
            Err(ExitCode::from(EXIT_INVALID_INPUT))
        }
    }
}

/// Serialize a value to JSON and print it to stdout, or exit with an error.
fn print_json(value: &impl serde::Serialize) -> Result<(), ExitCode> {
    match serde_json::to_string(value) {
        Ok(json) => {
            println!("{json}");
            Ok(())
        }
        Err(e) => {
            write_error(&t!("main.invalidJson", error = e.to_string()));
            Err(ExitCode::from(EXIT_SERVICE_ERROR))
        }
    }
}

#[cfg(not(windows))]
fn main() -> ExitCode {
    write_error(&t!("main.windowsOnly"));
    ExitCode::from(EXIT_SERVICE_ERROR)
}

#[cfg(windows)]
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        write_error(&t!("main.missingOperation"));
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    let operation = args[1].as_str();
    let input_json = match parse_input_arg(&args) {
        Ok(input) => input,
        Err(code) => return code,
    };
    let what_if = parse_what_if_flag(&args);

    match operation {
        "get" => {
            let input = match require_input(input_json) {
                Ok(input) => input,
                Err(code) => return code,
            };

            match service::get_service(&input) {
                Ok(result) => {
                    if let Err(code) = print_json(&result) {
                        return code;
                    }
                    ExitCode::from(EXIT_SUCCESS)
                }
                Err(e) => {
                    write_error(&e.to_string());
                    ExitCode::from(EXIT_SERVICE_ERROR)
                }
            }
        }
        "set" => {
            let input = match require_input(input_json) {
                Ok(input) => input,
                Err(code) => return code,
            };

            // In what-if, if the desired state is _exist: false, route to delete
            // so the projected state and metadata describe a delete operation.
            if what_if && matches!(input.exist, Some(false)) {
                match service::what_if_delete_service(&input) {
                    Ok(result) => {
                        if let Err(code) = print_json(&result) {
                            return code;
                        }
                        return ExitCode::from(EXIT_SUCCESS);
                    }
                    Err(e) => {
                        write_error(&e.to_string());
                        return ExitCode::from(EXIT_SERVICE_ERROR);
                    }
                }
            }

            match service::set_service(&input, what_if) {
                Ok(result) => {
                    if let Err(code) = print_json(&result) {
                        return code;
                    }
                    ExitCode::from(EXIT_SUCCESS)
                }
                Err(e) => {
                    write_error(&e.to_string());
                    ExitCode::from(EXIT_SERVICE_ERROR)
                }
            }
        }
        "export" => {
            match service::export_services() {
                Ok(services) => {
                    for svc in &services {
                        if let Err(code) = print_json(svc) {
                            return code;
                        }
                    }
                    ExitCode::from(EXIT_SUCCESS)
                }
                Err(e) => {
                    write_error(&e.to_string());
                    ExitCode::from(EXIT_SERVICE_ERROR)
                }
            }
        }
        _ => {
            write_error(&t!("main.unknownOperation", operation = operation));
            ExitCode::from(EXIT_INVALID_ARGS)
        }
    }
}

/// Parse the `--input <json>` argument from the command-line args.
fn parse_input_arg(args: &[String]) -> Result<Option<String>, ExitCode> {
    let mut i = 2; // skip binary name and operation
    while i < args.len() {
        if args[i] == "--input" || args[i] == "-i" {
            if i + 1 < args.len() {
                return Ok(Some(args[i + 1].clone()));
            }
            write_error(&t!("main.missingInputValue"));
            return Err(ExitCode::from(EXIT_INVALID_ARGS));
        }
        i += 1;
    }
    Ok(None)
}

/// Parse the `--what-if` / `-w` flag from the command-line args.
fn parse_what_if_flag(args: &[String]) -> bool {
    args.iter().skip(2).any(|a| a == "--what-if" || a == "-w")
}
