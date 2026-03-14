// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
mod util;
#[cfg(windows)]
mod optional_feature;
#[cfg(windows)]
mod feature_on_demand;

use rust_i18n::t;
use std::io::{self, Read, IsTerminal};

rust_i18n::i18n!("locales", fallback = "en-us");

fn read_stdin() -> Result<String, String> {
    let mut buffer = String::new();
    if !io::stdin().is_terminal() {
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| t!("main.errorReadingInput", err = e).to_string())?;
    }
    Ok(buffer)
}

fn dispatch(handler: impl FnOnce(&str) -> Result<String, String>) {
    let buffer = match read_stdin() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

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
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Error: {}", t!("main.windowsOnly"));
    std::process::exit(1);
}

#[cfg(windows)]
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Error: {}", t!("main.missingArguments"));
        eprintln!("{}", t!("main.usage"));
        std::process::exit(1);
    }

    let operation = args[1].as_str();
    let resource_type = args[2].as_str();

    match (operation, resource_type) {
        ("get", "optional-feature") => dispatch(optional_feature::handle_get),
        ("set", "optional-feature") => dispatch(optional_feature::handle_set),
        ("export", "optional-feature") => dispatch(optional_feature::handle_export),
        ("get", "feature-on-demand") => dispatch(feature_on_demand::handle_get),
        ("set", "feature-on-demand") => dispatch(feature_on_demand::handle_set),
        ("export", "feature-on-demand") => dispatch(feature_on_demand::handle_export),
        ("get" | "set" | "export", _) => {
            eprintln!("{}", t!("main.unknownResourceType", resource_type = resource_type));
            eprintln!("{}", t!("main.usage"));
            std::process::exit(1);
        }
        _ => {
            eprintln!("{}", t!("main.unknownOperation", operation = operation));
            eprintln!("{}", t!("main.usage"));
            std::process::exit(1);
        }
    }
}
