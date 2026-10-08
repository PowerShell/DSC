// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
pub(crate) mod dism;
#[cfg(windows)]
mod feature_on_demand;
#[cfg(windows)]
mod optional_feature;
#[cfg(windows)]
mod util;
#[cfg(windows)]
mod windows_feature;

use rust_i18n::t;

rust_i18n::i18n!("locales", fallback = "en-us");

fn get_input(args: &[String]) -> &str {
    args.windows(2)
        .find(|pair| pair[0] == "--input")
        .map_or("", |pair| pair[1].as_str())
}

fn dispatch(input: &str, handler: impl FnOnce(&str) -> Result<String, String>) {
    match handler(input) {
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
    let input = get_input(&args);

    match (operation, resource_type) {
        ("get", "optional-feature") => dispatch(input, optional_feature::handle_get),
        ("set", "optional-feature") => dispatch(input, optional_feature::handle_set),
        ("export", "optional-feature") => dispatch(input, optional_feature::handle_export),
        ("get", "feature-on-demand") => dispatch(input, feature_on_demand::handle_get),
        ("set", "feature-on-demand") => dispatch(input, feature_on_demand::handle_set),
        ("export", "feature-on-demand") => dispatch(input, feature_on_demand::handle_export),
        ("get", "windows-feature") => dispatch(input, windows_feature::handle_get),
        ("set", "windows-feature") => {
            let what_if = args.iter().any(|arg| arg == "-w" || arg == "--what-if");
            dispatch(input, |input| windows_feature::handle_set(input, what_if));
        }
        ("export", "windows-feature") => dispatch(input, windows_feature::handle_export),
        ("get" | "set" | "export", _) => {
            eprintln!(
                "{}",
                t!("main.unknownResourceType", resource_type = resource_type)
            );
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
