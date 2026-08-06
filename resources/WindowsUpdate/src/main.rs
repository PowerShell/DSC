// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
mod windows_update;

use rust_i18n::t;
use std::io::{self, Read, IsTerminal};

rust_i18n::i18n!("locales", fallback = "en-us");

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Error: {}", t!("main.missingOperation"));
        eprintln!("{}", t!("main.usage"));
        std::process::exit(1);
    }

    let operation = args[1].as_str();

    match operation {
        "export" => {
            // Read optional input from stdin (only if stdin is not a terminal/TTY)
            let mut buffer = String::new();
            if !io::stdin().is_terminal() {
                let _ = io::stdin().read_to_string(&mut buffer);
            }

            #[cfg(windows)]
            match windows_update::handle_export(&buffer) {
                Ok(output) => {
                    println!("{}", output);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }

            #[cfg(not(windows))]
            {
                eprintln!("Error: {}", t!("main.windowsUpdateOnlySupported"));
                std::process::exit(1);
            }
        }
        "get" => {
            // Read input from stdin
            let mut buffer = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buffer) {
                eprintln!("{}", t!("main.errorReadingInput", err = e));
                std::process::exit(1);
            }

            #[cfg(windows)]
            match windows_update::handle_get(&buffer) {
                Ok(output) => {
                    println!("{}", output);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }

            #[cfg(not(windows))]
            {
                eprintln!("Error: {}", t!("main.windowsUpdateOnlySupported"));
                std::process::exit(1);
            }
        }
        "set" => {
            // Read input from stdin
            let mut buffer = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buffer) {
                eprintln!("{}", t!("main.errorReadingInput", err = e));
                std::process::exit(1);
            }

            #[cfg(windows)]
            match windows_update::handle_set(&buffer, parse_what_if_arg(&args)) {
                Ok(output) => {
                    println!("{}", output);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }

            #[cfg(not(windows))]
            {
                eprintln!("Error: {}", t!("main.windowsUpdateOnlySupported"));
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("{}", t!("main.unknownOperation", operation = operation));
            eprintln!("{}", t!("main.usage"));
            std::process::exit(1);
        }
    }
}

#[cfg(windows)]
fn parse_what_if_arg(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-w" || arg == "--what-if")
}

#[cfg(all(test, windows))]
mod tests {
    use super::parse_what_if_arg;

    fn to_args(args: &[&str]) -> Vec<String> {
        args.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn detects_short_what_if_flag() {
        assert!(parse_what_if_arg(&to_args(&["windows_update", "set", "-w"])));
    }

    #[test]
    fn detects_long_what_if_flag() {
        assert!(parse_what_if_arg(&to_args(&["windows_update", "set", "--what-if"])));
    }

    #[test]
    fn returns_false_without_what_if_flag() {
        assert!(!parse_what_if_arg(&to_args(&["windows_update", "set"])));
    }

    #[test]
    fn does_not_match_similar_arguments() {
        assert!(!parse_what_if_arg(&to_args(&["windows_update", "set", "-what-if", "--w", "what-if"])));
    }
}
