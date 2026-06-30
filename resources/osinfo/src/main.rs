// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib_osinfo::{perform_test, OsInfo};
use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("export") => {
            let json = serde_json::to_string(&OsInfo::new(true)).unwrap_or_default();
            println!("{json}");
        },
        Some("test") => {
            let mut input = String::new();
            if let Err(e) = std::io::stdin().read_to_string(&mut input) {
                eprintln!("Failed to read stdin: {e}");
                std::process::exit(1);
            }
            match perform_test(&input) {
                Ok(result) => {
                    let json = serde_json::to_string(&result).unwrap_or_default();
                    println!("{json}");
                },
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                },
            }
        },
        _ => {
            let json = serde_json::to_string(&OsInfo::new(false)).unwrap_or_default();
            println!("{json}");
        },
    }
}
