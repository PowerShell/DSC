// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg_attr(not(windows), allow(dead_code))]
mod rights;
#[cfg_attr(not(windows), allow(dead_code))]
mod types;

#[cfg(windows)]
mod lsa;
#[cfg(windows)]
mod sid;
#[cfg(windows)]
mod user_rights;

use rust_i18n::t;
use std::process::ExitCode;

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: u8 = 0;
const EXIT_INVALID_ARGS: u8 = 1;
const EXIT_INVALID_INPUT: u8 = 2;
const EXIT_SECURITY_POLICY_ERROR: u8 = 3;
const EXIT_ELEVATION_REQUIRED: u8 = 4;

/// Write a JSON error object to stderr: `{"error":"<message>"}`
fn write_error(message: &str) {
    eprintln!("{}", serde_json::json!({"error": message}));
}

#[cfg(not(windows))]
fn main() -> ExitCode {
    write_error(&t!("main.windowsOnly"));
    ExitCode::from(EXIT_SECURITY_POLICY_ERROR)
}

#[cfg(windows)]
fn main() -> ExitCode {
    use types::UserRightsAssignment;

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        write_error(&t!("main.missingOperation"));
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    let operation = args[1].as_str();
    if !matches!(operation, "get" | "set" | "test" | "export") {
        write_error(&t!("main.unknownOperation", operation = operation));
        return ExitCode::from(EXIT_INVALID_ARGS);
    }

    let Some(target) = args.get(2) else {
        write_error(&t!("main.missingTarget"));
        return ExitCode::from(EXIT_INVALID_ARGS);
    };

    let input_json = match parse_input_arg(&args) {
        Ok(input) => input,
        Err(code) => return code,
    };
    let what_if = parse_what_if_flag(&args);

    match (operation, target.as_str()) {
        ("get", "user-right") => with_input(input_json, |input: UserRightsAssignment| user_rights::get(&input)),
        ("set", "user-right") => with_input(input_json, |input: UserRightsAssignment| user_rights::set(&input, what_if)),
        ("test", "user-right") => with_input(input_json, |input: UserRightsAssignment| user_rights::test(&input)),
        ("export", "user-right") => finish_many(user_rights::export()),
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

/// Run an operation that needs input and print its single result.
#[cfg(windows)]
fn with_input<T, U>(input_json: Option<String>, operation: impl FnOnce(T) -> Result<U, types::SecurityPolicyError>) -> ExitCode
where
    T: serde::de::DeserializeOwned,
    U: serde::Serialize,
{
    match require_input::<T>(input_json) {
        Ok(input) => finish(operation(input)),
        Err(code) => code,
    }
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
            Err(ExitCode::from(EXIT_SECURITY_POLICY_ERROR))
        }
    }
}

/// Map a resource error to its documented exit code.
#[cfg(windows)]
fn exit_code_for(error: &types::SecurityPolicyError) -> ExitCode {
    use types::SecurityPolicyError;

    ExitCode::from(match error {
        SecurityPolicyError::ElevationRequired => EXIT_ELEVATION_REQUIRED,
        SecurityPolicyError::InvalidInput(_) => EXIT_INVALID_INPUT,
        SecurityPolicyError::Policy(_) => EXIT_SECURITY_POLICY_ERROR,
    })
}

/// Print a single result as JSON, or write the error and return its exit code.
#[cfg(windows)]
fn finish<T: serde::Serialize>(result: Result<T, types::SecurityPolicyError>) -> ExitCode {
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
fn finish_many<T: serde::Serialize>(result: Result<Vec<T>, types::SecurityPolicyError>) -> ExitCode {
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

/// Parse the `--input <json>` argument. Arguments 1 and 2 are the operation and the target.
#[cfg(windows)]
fn parse_input_arg(args: &[String]) -> Result<Option<String>, ExitCode> {
    let mut index = 3;
    while index < args.len() {
        if args[index] == "--input" || args[index] == "-i" {
            if index + 1 < args.len() {
                return Ok(Some(args[index + 1].clone()));
            }
            write_error(&t!("main.missingInputValue"));
            return Err(ExitCode::from(EXIT_INVALID_ARGS));
        }
        index += 1;
    }
    Ok(None)
}

/// Parse the `--what-if` / `-w` flag. Arguments 1 and 2 are the operation and the target.
#[cfg(windows)]
fn parse_what_if_flag(args: &[String]) -> bool {
    args.iter().skip(3).any(|arg| arg == "--what-if" || arg == "-w")
}

#[cfg(all(test, windows))]
mod tests {
    use super::{parse_input_arg, parse_what_if_flag};

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn finds_long_and_short_input_arguments_after_the_target() {
        assert_eq!(
            parse_input_arg(&args(&["exe", "get", "user-right", "--input", "{}"])),
            Ok(Some("{}".to_string()))
        );
        assert_eq!(
            parse_input_arg(&args(&["exe", "set", "user-right", "-w", "-i", "[]"])),
            Ok(Some("[]".to_string()))
        );
        assert_eq!(parse_input_arg(&args(&["exe", "export", "user-right"])), Ok(None));
        assert!(parse_input_arg(&args(&["exe", "get", "user-right", "--input"])).is_err());
    }

    #[test]
    fn finds_what_if_flag_after_the_target() {
        assert!(parse_what_if_flag(&args(&["exe", "set", "user-right", "--what-if", "--input", "{}"])));
        assert!(parse_what_if_flag(&args(&["exe", "set", "user-right", "--input", "{}", "-w"])));
        assert!(!parse_what_if_flag(&args(&["exe", "set", "user-right", "--input", "{}"])));
        assert!(!parse_what_if_flag(&args(&["exe", "-w", "user-right"])));
    }
}
