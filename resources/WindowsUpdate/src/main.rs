// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
mod windows_update;

use std::io::{self, Read};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Error: Missing operation argument");
        eprintln!("Usage: wu_dsc <get|set|test>");
        std::process::exit(1);
    }

    let operation = args[1].as_str();

    match operation {
        "get" => {
            // Read input from stdin
            let mut buffer = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut buffer) {
                eprintln!("Error reading input: {}", e);
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
                eprintln!("Error: Windows Update resource is only supported on Windows");
                std::process::exit(1);
            }
        }
        "set" => {
            eprintln!("Error: Set operation is not implemented for Windows Update resource");
            std::process::exit(1);
        }
        "test" => {
            eprintln!("Error: Test operation is not implemented for Windows Update resource");
            std::process::exit(1);
        }
        _ => {
            eprintln!("Error: Unknown operation '{}'", operation);
            eprintln!("Usage: wu_dsc <get|set|test>");
            std::process::exit(1);
        }
    }
}
