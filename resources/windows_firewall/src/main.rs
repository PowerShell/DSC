// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod types;

#[cfg(windows)]
mod firewall;

use rust_i18n::t;
use std::process::ExitCode;

use types::FirewallRuleList;

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: u8 = 0;
const EXIT_INVALID_ARGS: u8 = 1;
const EXIT_INVALID_INPUT: u8 = 2;
const EXIT_FIREWALL_ERROR: u8 = 3;

pub(crate) fn write_error(message: &str) {
    eprintln!("{}", serde_json::json!({ "error": message }));
}

fn print_json(value: &impl serde::Serialize) -> Result<(), ExitCode> {
    match serde_json::to_string(value) {
        Ok(json) => {
            println!("{json}");
            Ok(())
        }
        Err(error) => {
            write_error(&t!("main.invalidJson", error = error.to_string()));
            Err(ExitCode::from(EXIT_FIREWALL_ERROR))
        }
    }
}

fn require_input(input_json: Option<String>) -> Result<FirewallRuleList, ExitCode> {
    let json = match input_json {
        Some(json) => json,
        None => {
            write_error(&t!("main.missingInput"));
            return Err(ExitCode::from(EXIT_INVALID_ARGS));
        }
    };

    match serde_json::from_str(&json) {
        Ok(value) => Ok(value),
        Err(error) => {
            write_error(&t!("main.invalidJson", error = error.to_string()));
            Err(ExitCode::from(EXIT_INVALID_INPUT))
        }
    }
}

#[cfg(not(windows))]
fn main() -> ExitCode {
    write_error(&t!("main.windowsOnly"));
    ExitCode::from(EXIT_FIREWALL_ERROR)
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

    match operation {
        "get" => {
            let input = match require_input(input_json) {
                Ok(input) => input,
                Err(code) => return code,
            };
            match firewall::get_rules(&input) {
                Ok(result) => {
                    if let Err(code) = print_json(&result) {
                        return code;
                    }
                    ExitCode::from(EXIT_SUCCESS)
                }
                Err(error) => {
                    write_error(&error.to_string());
                    ExitCode::from(EXIT_FIREWALL_ERROR)
                }
            }
        }
        "set" => {
            let what_if = parse_what_if_arg(&args);
            let input = match require_input(input_json) {
                Ok(input) => input,
                Err(code) => return code,
            };
            match firewall::set_rules(&input, what_if) {
                Ok(result) => {
                    if let Err(code) = print_json(&result) {
                        return code;
                    }
                    ExitCode::from(EXIT_SUCCESS)
                }
                Err(error) => {
                    write_error(&error.to_string());
                    ExitCode::from(EXIT_FIREWALL_ERROR)
                }
            }
        }
        "export" => {
            match firewall::export_rules() {
                Ok(result) => {
                    if let Err(code) = print_json(&result) {
                        return code;
                    }
                    ExitCode::from(EXIT_SUCCESS)
                }
                Err(error) => {
                    write_error(&error.to_string());
                    ExitCode::from(EXIT_FIREWALL_ERROR)
                }
            }
        }
        _ => {
            write_error(&t!("main.unknownOperation", operation = operation));
            ExitCode::from(EXIT_INVALID_ARGS)
        }
    }
}

fn parse_input_arg(args: &[String]) -> Result<Option<String>, ExitCode> {
    let mut index = 2;
    while index < args.len() {
        if args[index] == "--input" || args[index] == "-i" {
            if index + 1 < args.len() {
                return Ok(Some(args[index + 1].clone()));
            }
            write_error(&t!("main.missingInputValue"));
            return Err(ExitCode::from(EXIT_INVALID_ARGS));
        }
        index += 1;
    }
    Ok(None)
}

fn parse_what_if_arg(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-w" || arg == "--what-if")
}
