// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg_attr(not(windows), allow(dead_code))]
mod subcategories;
#[cfg_attr(not(windows), allow(dead_code))]
mod types;

#[cfg(windows)]
mod audit;
#[cfg(windows)]
mod options;

use rust_i18n::t;
use std::process::ExitCode;

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: u8 = 0;
const EXIT_INVALID_ARGS: u8 = 1;
const EXIT_INVALID_INPUT: u8 = 2;
const EXIT_AUDIT_ERROR: u8 = 3;
const EXIT_ELEVATION_REQUIRED: u8 = 4;

/// Write a JSON error object to stderr: `{"error":"<message>"}`
fn write_error(message: &str) {
    eprintln!("{}", serde_json::json!({"error": message}));
}

#[cfg(not(windows))]
fn main() -> ExitCode {
    write_error(&t!("main.windowsOnly"));
    ExitCode::from(EXIT_AUDIT_ERROR)
}

#[cfg(windows)]
fn main() -> ExitCode {
    use types::{AuditPolicy, AuditPolicyOption};

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        write_error(&t!("main.missingOperation"));
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    let operation = args[1].as_str();
    if !matches!(operation, "get" | "set" | "export") {
        write_error(&t!("main.unknownOperation", operation = operation));
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    let Some(target) = args.get(2) else {
        write_error(&t!("main.missingTarget"));
        return ExitCode::from(EXIT_INVALID_ARGS);
    };

    let Arguments { input: input_json, what_if } = match parse_arguments(&args) {
        Ok(arguments) => arguments,
        Err(code) => return code,
    };

    match (operation, target.as_str()) {
        ("get", "subcategory") => match require_input::<AuditPolicy>(input_json) {
            Ok(input) => finish(audit::get_audit_policy(&input)),
            Err(code) => code,
        },
        ("set", "subcategory") => match require_input::<AuditPolicy>(input_json) {
            Ok(input) => finish(audit::set_audit_policy(&input, what_if)),
            Err(code) => code,
        },
        ("export", "subcategory") => finish_many(audit::export_audit_policies()),
        ("get", "option") => match require_input::<AuditPolicyOption>(input_json) {
            Ok(input) => finish(options::get_option(&input)),
            Err(code) => code,
        },
        ("set", "option") => match require_input::<AuditPolicyOption>(input_json) {
            Ok(input) => finish(options::set_option(&input, what_if)),
            Err(code) => code,
        },
        ("export", "option") => finish_many(options::export_options()),
        (_, target) => {
            write_error(&t!("main.unknownTarget", target = target));
            ExitCode::from(EXIT_INVALID_ARGS)
        }
    }
}

/// Deserialize the required JSON input, or return the exit code to use.
#[cfg(windows)]
fn require_input<T: serde::de::DeserializeOwned>(input_json: Option<String>) -> Result<T, ExitCode> {
    let Some(json) = input_json else {
        write_error(&t!("main.missingInput"));
        return Err(ExitCode::from(EXIT_INVALID_ARGS));
    };
    serde_json::from_str(&json).map_err(|error| {
        write_error(&t!("main.invalidJson", error = error.to_string()));
        ExitCode::from(EXIT_INVALID_INPUT)
    })
}

/// Serialize a value to JSON and print it to stdout, or return the exit code to use.
#[cfg(windows)]
fn print_json(value: &impl serde::Serialize) -> Result<(), ExitCode> {
    match serde_json::to_string(value) {
        Ok(json) => {
            println!("{json}");
            Ok(())
        }
        Err(error) => {
            write_error(&t!("main.serializeFailed", error = error.to_string()));
            Err(ExitCode::from(EXIT_AUDIT_ERROR))
        }
    }
}

/// Map a resource error to its documented exit code.
#[cfg(windows)]
fn exit_code_for(error: &types::AuditError) -> ExitCode {
    use types::AuditError;

    ExitCode::from(match error {
        AuditError::ElevationRequired => EXIT_ELEVATION_REQUIRED,
        AuditError::InvalidInput(_) => EXIT_INVALID_INPUT,
        AuditError::Audit(_) => EXIT_AUDIT_ERROR,
    })
}

/// Print a single result as JSON, or write the error and return its exit code.
#[cfg(windows)]
fn finish<T: serde::Serialize>(result: Result<T, types::AuditError>) -> ExitCode {
    match result {
        Ok(value) => match print_json(&value) {
            Ok(()) => ExitCode::from(EXIT_SUCCESS),
            Err(code) => code,
        },
        Err(error) => {
            write_error(&error.to_string());
            exit_code_for(&error)
        }
    }
}

/// Print each exported instance on its own line, or write the error and return its exit code.
#[cfg(windows)]
fn finish_many<T: serde::Serialize>(result: Result<Vec<T>, types::AuditError>) -> ExitCode {
    match result {
        Ok(values) => {
            for value in &values {
                if let Err(code) = print_json(value) {
                    return code;
                }
            }
            ExitCode::from(EXIT_SUCCESS)
        }
        Err(error) => {
            write_error(&error.to_string());
            exit_code_for(&error)
        }
    }
}

/// The options that may follow the operation and the target.
#[cfg(windows)]
#[derive(Debug, Default, PartialEq, Eq)]
struct Arguments {
    /// The JSON given with `--input` / `-i`.
    input: Option<String>,
    /// Whether `--what-if` / `-w` was given.
    what_if: bool,
}

/// Parse every argument after the operation and the target (arguments 1 and 2). Only
/// `--input <json>` and `--what-if` are accepted; anything else is rejected before any operation
/// is dispatched, so a misspelled preview flag can never turn into a real change.
///
/// # Errors
///
/// Returns the exit code to use when `--input` has no value or an argument is unknown.
#[cfg(windows)]
fn parse_arguments(args: &[String]) -> Result<Arguments, ExitCode> {
    let mut arguments = Arguments::default();
    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--input" | "-i" => {
                let Some(value) = args.get(index + 1) else {
                    write_error(&t!("main.missingInputValue"));
                    return Err(ExitCode::from(EXIT_INVALID_ARGS));
                };
                arguments.input = Some(value.clone());
                index += 2;
            }
            "--what-if" | "-w" => {
                arguments.what_if = true;
                index += 1;
            }
            unknown => {
                write_error(&t!("main.unknownArgument", argument = unknown));
                return Err(ExitCode::from(EXIT_INVALID_ARGS));
            }
        }
    }
    Ok(arguments)
}

#[cfg(all(test, windows))]
mod tests {
    use super::{Arguments, EXIT_INVALID_ARGS, parse_arguments};
    use std::process::ExitCode;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    fn parsed(input: Option<&str>, what_if: bool) -> Arguments {
        Arguments {
            input: input.map(ToString::to_string),
            what_if,
        }
    }

    #[test]
    fn finds_long_and_short_input_arguments_after_the_target() {
        assert_eq!(
            parse_arguments(&args(&["exe", "get", "subcategory", "--input", "{}"])),
            Ok(parsed(Some("{}"), false))
        );
        assert_eq!(
            parse_arguments(&args(&["exe", "set", "option", "-w", "-i", "[]"])),
            Ok(parsed(Some("[]"), true))
        );
        assert_eq!(parse_arguments(&args(&["exe", "export", "subcategory"])), Ok(parsed(None, false)));
        assert_eq!(
            parse_arguments(&args(&["exe", "get", "subcategory", "--input"])),
            Err(ExitCode::from(EXIT_INVALID_ARGS))
        );
    }

    #[test]
    fn finds_what_if_flag_after_the_target() {
        assert_eq!(
            parse_arguments(&args(&["exe", "set", "subcategory", "--what-if", "--input", "{}"])),
            Ok(parsed(Some("{}"), true))
        );
        assert_eq!(
            parse_arguments(&args(&["exe", "set", "option", "--input", "{}", "-w"])),
            Ok(parsed(Some("{}"), true))
        );
        assert_eq!(
            parse_arguments(&args(&["exe", "set", "subcategory", "--input", "{}"])),
            Ok(parsed(Some("{}"), false))
        );
        // The operation and target positions are never read as flags.
        assert_eq!(parse_arguments(&args(&["exe", "-w", "subcategory"])), Ok(parsed(None, false)));
    }

    #[test]
    fn rejects_unknown_arguments_before_and_after_input() {
        let invalid = Err(ExitCode::from(EXIT_INVALID_ARGS));
        assert_eq!(parse_arguments(&args(&["exe", "set", "option", "--whatif", "--input", "{}"])), invalid);
        assert_eq!(parse_arguments(&args(&["exe", "set", "option", "--input", "{}", "--whatif"])), invalid);
        assert_eq!(parse_arguments(&args(&["exe", "set", "subcategory", "--input", "{}", "extra"])), invalid);
        assert_eq!(parse_arguments(&args(&["exe", "get", "subcategory", "-x"])), invalid);
    }
}
