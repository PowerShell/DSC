// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib_osinfo::{perform_test, OsInfo};
use std::io::Read;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("export") => {
            let json = serde_json::to_string(&OsInfo::new(true))
                .map_err(|e| format!("Failed to serialize OS info as JSON: {e}"))?;
            println!("{json}");
        },
        Some("test") => {
            let mut input = String::new();
            std::io::stdin()
                .read_to_string(&mut input)
                .map_err(|e| format!("Failed to read stdin: {e}"))?;
            let result = perform_test(&input)?;
            let json = serde_json::to_string(&result)
                .map_err(|e| format!("Failed to serialize test result as JSON: {e}"))?;
            println!("{json}");
        },
        _ => {
            let json = serde_json::to_string(&OsInfo::new(false))
                .map_err(|e| format!("Failed to serialize OS info as JSON: {e}"))?;
            println!("{json}");
        },
    }
    Ok(())
}
