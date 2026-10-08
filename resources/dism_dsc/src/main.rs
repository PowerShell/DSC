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
use std::process::ExitCode;

rust_i18n::i18n!("locales", fallback = "en-us");

fn get_input(args: &[String]) -> &str {
    args.windows(2)
        .find(|pair| pair[0] == "--input")
        .map_or("", |pair| pair[1].as_str())
}

fn dispatch(input: &str, handler: impl FnOnce(&str) -> Result<String, String>) -> ExitCode {
    match handler(input) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(windows))]
fn main() -> ExitCode {
    eprintln!("Error: {}", t!("main.windowsOnly"));
    ExitCode::FAILURE
}

#[cfg(windows)]
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Error: {}", t!("main.missingArguments"));
        eprintln!("{}", t!("main.usage"));
        return ExitCode::FAILURE;
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
            dispatch(input, |input| windows_feature::handle_set(input, what_if))
        }
        ("export", "windows-feature") => dispatch(input, windows_feature::handle_export),
        ("get" | "set" | "export", _) => {
            eprintln!(
                "{}",
                t!("main.unknownResourceType", resource_type = resource_type)
            );
            eprintln!("{}", t!("main.usage"));
            ExitCode::FAILURE
        }
        _ => {
            eprintln!("{}", t!("main.unknownOperation", operation = operation));
            eprintln!("{}", t!("main.usage"));
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{dispatch, get_input};
    use std::process::ExitCode;

    #[test]
    fn input_argument_is_returned() {
        let args = vec![
            "dism_dsc".to_string(),
            "get".to_string(),
            "windows-feature".to_string(),
            "--input".to_string(),
            r#"{"features":[]}"#.to_string(),
        ];

        assert_eq!(get_input(&args), r#"{"features":[]}"#);
    }

    #[test]
    fn missing_input_argument_returns_empty_string() {
        let args = vec![
            "dism_dsc".to_string(),
            "export".to_string(),
            "windows-feature".to_string(),
        ];

        assert_eq!(get_input(&args), "");
    }

    #[test]
    fn dispatch_returns_success_for_handler_output() {
        assert_eq!(
            dispatch("input", |input| Ok(input.to_string())),
            ExitCode::SUCCESS
        );
    }

    #[test]
    fn dispatch_returns_failure_for_handler_error() {
        assert_eq!(
            dispatch("input", |_| Err("failed".to_string())),
            ExitCode::FAILURE
        );
    }
}
