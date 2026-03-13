// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
mod optional_feature;

use rust_i18n::t;
use std::io::{self, Read, IsTerminal};

rust_i18n::i18n!("locales", fallback = "en-us");

fn read_stdin(required: bool) -> Result<String, String> {
    let mut buffer = String::new();
    if required || !io::stdin().is_terminal() {
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| t!("main.errorReadingInput", err = e).to_string())?;
    }
    Ok(buffer)
}

fn dispatch(handler: impl FnOnce(&str) -> Result<String, String>, stdin_required: bool) {
    let buffer = match read_stdin(stdin_required) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    #[cfg(windows)]
    match handler(&buffer) {
        Ok(output) => {
            println!("{output}");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }

    #[cfg(not(windows))]
    {
        let _ = buffer;
        eprintln!("Error: {}", t!("main.windowsOnly"));
        std::process::exit(1);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: {}", t!("main.missingOperation"));
        eprintln!("{}", t!("main.usage"));
        std::process::exit(1);
    }

    let operation = args[1].as_str();

    match operation {
        "export" => dispatch(optional_feature::handle_export, false),
        "get" => dispatch(optional_feature::handle_get, true),
        "set" => dispatch(optional_feature::handle_set, true),
        _ => {
            eprintln!("{}", t!("main.unknownOperation", operation = operation));
            eprintln!("{}", t!("main.usage"));
            std::process::exit(1);
        }
    }
}
