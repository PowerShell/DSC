// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod types;
mod util;

#[cfg(windows)]
mod firewall;

use rust_i18n::t;
use std::process::exit;

use types::FirewallRuleList;

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_ARGS: i32 = 1;
const EXIT_INVALID_INPUT: i32 = 2;
const EXIT_FIREWALL_ERROR: i32 = 3;

pub(crate) fn write_error(message: &str) {
    eprintln!("{}", serde_json::json!({ "error": message }));
}

fn print_json(value: &impl serde::Serialize) {
    match serde_json::to_string(value) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            write_error(&t!("main.invalidJson", error = error.to_string()));
            exit(EXIT_FIREWALL_ERROR);
        }
    }
}

fn require_input(input_json: Option<String>) -> FirewallRuleList {
    let json = match input_json {
        Some(json) => json,
        None => {
            write_error(&t!("main.missingInput"));
            exit(EXIT_INVALID_ARGS);
        }
    };

    match serde_json::from_str(&json) {
        Ok(value) => value,
        Err(error) => {
            write_error(&t!("main.invalidJson", error = error.to_string()));
            exit(EXIT_INVALID_INPUT);
        }
    }
}

#[cfg(not(windows))]
fn main() {
    write_error(&t!("main.windowsOnly"));
    exit(EXIT_FIREWALL_ERROR);
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
            match firewall::get_rules(&input) {
                Ok(result) => {
                    print_json(&result);
                    exit(EXIT_SUCCESS);
                }
                Err(error) => {
                    write_error(&error.to_string());
                    exit(EXIT_FIREWALL_ERROR);
                }
            }
        }
        "set" => {
            let input = require_input(input_json);
            match firewall::set_rules(&input) {
                Ok(result) => {
                    print_json(&result);
                    exit(EXIT_SUCCESS);
                }
                Err(error) => {
                    write_error(&error.to_string());
                    exit(EXIT_FIREWALL_ERROR);
                }
            }
        }
        "export" => {
            let filters: Option<FirewallRuleList> = match input_json {
                Some(json) => match serde_json::from_str(&json) {
                    Ok(value) => Some(value),
                    Err(error) => {
                        write_error(&t!("main.invalidJson", error = error.to_string()));
                        exit(EXIT_INVALID_INPUT);
                    }
                },
                None => None,
            };

            match firewall::export_rules(filters.as_ref()) {
                Ok(result) => {
                    print_json(&result);
                    exit(EXIT_SUCCESS);
                }
                Err(error) => {
                    write_error(&error.to_string());
                    exit(EXIT_FIREWALL_ERROR);
                }
            }
        }
        _ => {
            write_error(&t!("main.unknownOperation", operation = operation));
            exit(EXIT_INVALID_ARGS);
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
