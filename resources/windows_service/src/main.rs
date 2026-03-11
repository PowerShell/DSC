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
            let json = match input_json {
                Some(j) => j,
                None => {
                    write_error(&t!("main.missingInput"));
                    exit(EXIT_INVALID_ARGS);
                }
            };

            let input: WindowsService = match serde_json::from_str(&json) {
                Ok(s) => s,
                Err(e) => {
                    write_error(&t!("main.invalidJson", error = e.to_string()));
                    exit(EXIT_INVALID_INPUT);
                }
            };

            #[cfg(windows)]
            {
                match service::get_service(&input) {
                    Ok(result) => {
                        println!("{}", serde_json::to_string(&result).unwrap());
                        exit(EXIT_SUCCESS);
                    }
                    Err(e) => {
                        write_error(&e.to_string());
                        exit(EXIT_SERVICE_ERROR);
                    }
                }
            }

            #[cfg(not(windows))]
            {
                let _ = input;
                write_error(&t!("main.windowsOnly"));
                exit(EXIT_SERVICE_ERROR);
            }
        }
        "set" => {
            let json = match input_json {
                Some(j) => j,
                None => {
                    write_error(&t!("main.missingInput"));
                    exit(EXIT_INVALID_ARGS);
                }
            };

            let input: WindowsService = match serde_json::from_str(&json) {
                Ok(s) => s,
                Err(e) => {
                    write_error(&t!("main.invalidJson", error = e.to_string()));
                    exit(EXIT_INVALID_INPUT);
                }
            };

            #[cfg(windows)]
            {
                match service::set_service(&input) {
                    Ok(result) => {
                        println!("{}", serde_json::to_string(&result).unwrap());
                        exit(EXIT_SUCCESS);
                    }
                    Err(e) => {
                        write_error(&e.to_string());
                        exit(EXIT_SERVICE_ERROR);
                    }
                }
            }

            #[cfg(not(windows))]
            {
                let _ = input;
                write_error(&t!("main.windowsOnly"));
                exit(EXIT_SERVICE_ERROR);
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

            #[cfg(windows)]
            {
                match service::export_services(filter.as_ref()) {
                    Ok(services) => {
                        for svc in &services {
                            println!("{}", serde_json::to_string(svc).unwrap());
                        }
                        exit(EXIT_SUCCESS);
                    }
                    Err(e) => {
                        write_error(&e.to_string());
                        exit(EXIT_SERVICE_ERROR);
                    }
                }
            }

            #[cfg(not(windows))]
            {
                let _ = filter;
                write_error(&t!("main.windowsOnly"));
                exit(EXIT_SERVICE_ERROR);
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
