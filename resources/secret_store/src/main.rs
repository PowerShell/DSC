// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod types;

#[cfg(windows)]
mod vault_config;
#[cfg(windows)]
mod secret;

use rust_i18n::t;
use std::io::{self, Read, IsTerminal};
use std::process::exit;

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_ARGS: i32 = 1;
const EXIT_INVALID_INPUT: i32 = 2;
const EXIT_OPERATION_ERROR: i32 = 3;

/// Write a JSON error object to stderr.
fn write_error(message: &str) {
    eprintln!("{}", serde_json::json!({"error": message}));
}

/// Read JSON input from stdin.
fn read_stdin() -> Result<String, String> {
    let mut buffer = String::new();
    if !io::stdin().is_terminal() {
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| t!("main.stdinReadError", error = e).to_string())?;
    }
    Ok(buffer)
}

/// Parse `--input <json>` from command line args, or fall back to stdin.
fn get_input_json(args: &[String]) -> Option<String> {
    // Check for --input arg
    for i in 0..args.len() {
        if args[i] == "--input" {
            if i + 1 < args.len() {
                return Some(args[i + 1].clone());
            }
        }
    }
    // Fall back to stdin
    read_stdin().ok().filter(|s| !s.trim().is_empty())
}

/// Serialize a value to JSON and print it to stdout, or exit with an error.
fn print_json(value: &impl serde::Serialize) {
    match serde_json::to_string(value) {
        Ok(json) => println!("{json}"),
        Err(e) => {
            write_error(&t!("main.invalidJson", error = e.to_string()));
            exit(EXIT_OPERATION_ERROR);
        }
    }
}

#[cfg(not(windows))]
fn main() {
    write_error(&t!("main.windowsOnly"));
    exit(EXIT_OPERATION_ERROR);
}

#[cfg(windows)]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        write_error(&t!("main.missingArguments"));
        exit(EXIT_INVALID_ARGS);
    }

    let operation = args[1].as_str();
    let resource_type = args[2].as_str();
    let input_json = get_input_json(&args);

    match (operation, resource_type) {
        // --- vault-config operations ---
        ("get", "vault-config") => {
            let input: types::VaultConfig = parse_input_or_default(input_json);
            match vault_config::get_config(&input) {
                Ok(result) => {
                    print_json(&result);
                    exit(EXIT_SUCCESS);
                }
                Err(e) => {
                    write_error(&e);
                    exit(EXIT_OPERATION_ERROR);
                }
            }
        }
        ("set", "vault-config") => {
            let input: types::VaultConfig = parse_input_or_default(input_json);
            match vault_config::set_config(&input) {
                Ok(result) => {
                    print_json(&result);
                    exit(EXIT_SUCCESS);
                }
                Err(e) => {
                    write_error(&e);
                    exit(EXIT_OPERATION_ERROR);
                }
            }
        }
        ("test", "vault-config") => {
            let input: types::VaultConfig = parse_input_or_default(input_json);
            match vault_config::test_config(&input) {
                Ok(in_desired_state) => {
                    let mut result = input.clone();
                    // DSC convention: _inDesiredState is communicated via the diff
                    println!("{}", serde_json::json!({
                        "inDesiredState": in_desired_state
                    }));
                    let _ = result;
                    exit(EXIT_SUCCESS);
                }
                Err(e) => {
                    write_error(&e);
                    exit(EXIT_OPERATION_ERROR);
                }
            }
        }

        // --- secret operations ---
        ("get", "secret") => {
            let input: types::Secret = require_input(input_json);
            match secret::get_secret(&input) {
                Ok(result) => {
                    print_json(&result);
                    exit(EXIT_SUCCESS);
                }
                Err(e) => {
                    write_error(&e);
                    exit(EXIT_OPERATION_ERROR);
                }
            }
        }
        ("set", "secret") => {
            let input: types::Secret = require_input(input_json);
            match secret::set_secret(&input) {
                Ok(result) => {
                    print_json(&result);
                    exit(EXIT_SUCCESS);
                }
                Err(e) => {
                    write_error(&e);
                    exit(EXIT_OPERATION_ERROR);
                }
            }
        }
        ("export", "secret") => {
            let filter: Option<types::Secret> = input_json.and_then(|json| {
                serde_json::from_str(&json).ok()
            });
            match secret::export_secrets(filter.as_ref()) {
                Ok(secrets) => {
                    for s in &secrets {
                        print_json(s);
                    }
                    exit(EXIT_SUCCESS);
                }
                Err(e) => {
                    write_error(&e);
                    exit(EXIT_OPERATION_ERROR);
                }
            }
        }

        // --- error handling ---
        ("get" | "set" | "test" | "export", _) => {
            write_error(&t!("main.unknownResourceType", resource_type = resource_type));
            exit(EXIT_INVALID_ARGS);
        }
        _ => {
            write_error(&t!("main.unknownOperation", operation = operation));
            exit(EXIT_INVALID_ARGS);
        }
    }
}

/// Parse JSON input into a typed struct, or exit with an error.
#[cfg(windows)]
fn require_input<T: serde::de::DeserializeOwned>(input_json: Option<String>) -> T {
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

/// Parse JSON input into a typed struct, or return Default if no input provided.
#[cfg(windows)]
fn parse_input_or_default<T: serde::de::DeserializeOwned + Default>(input_json: Option<String>) -> T {
    match input_json {
        Some(json) if !json.trim().is_empty() => {
            match serde_json::from_str(&json) {
                Ok(v) => v,
                Err(e) => {
                    write_error(&t!("main.invalidJson", error = e.to_string()));
                    exit(EXIT_INVALID_INPUT);
                }
            }
        }
        _ => T::default(),
    }
}
