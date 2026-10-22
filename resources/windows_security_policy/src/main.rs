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

/// Write a JSON informational message to stderr: `{"info":"<message>"}`. DSC logs it at the
/// info level.
#[cfg(windows)]
fn write_info(message: &str) {
    eprintln!("{}", serde_json::json!({"info": message}));
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

    let options = match parse_options(&args) {
        Ok(options) => options,
        Err(error) => {
            write_error(&error.to_string());
            return ExitCode::from(EXIT_INVALID_ARGS);
        }
    };

    match (operation, target.as_str()) {
        ("get", "user-right") => with_input(options.input, |input: UserRightsAssignment| user_rights::get(&input)),
        ("set", "user-right") => {
            with_input(options.input, |input: UserRightsAssignment| user_rights::set(&input, options.what_if))
        }
        ("test", "user-right") => with_input(options.input, |input: UserRightsAssignment| user_rights::test(&input)),
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

/// The options that may follow the operation and the target.
#[cfg(windows)]
#[derive(Debug, Default, PartialEq, Eq)]
struct Options {
    /// The JSON given with `--input` / `-i`.
    input: Option<String>,
    /// Whether `--what-if` / `-w` was given.
    what_if: bool,
}

/// Why the argument list was rejected.
#[cfg(windows)]
#[derive(Debug, PartialEq, Eq)]
enum ArgError {
    /// `--input` was the last argument.
    MissingInputValue,
    /// An argument that is neither a known option nor the value of `--input`.
    UnknownOption(String),
}

#[cfg(windows)]
impl std::fmt::Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingInputValue => f.write_str(&t!("main.missingInputValue")),
            Self::UnknownOption(option) => f.write_str(&t!("main.unknownOption", option = option)),
        }
    }
}

/// Parse every argument after the operation and the target (arguments 1 and 2).
///
/// Only `--input <json>` / `-i <json>` and `--what-if` / `-w` are accepted, in any order. Every
/// other argument is an error, so that a misspelled option can never be ignored.
///
/// # Errors
///
/// Returns `MissingInputValue` when `--input` has no value and `UnknownOption` for any argument
/// that isn't a known option or the value of `--input`.
#[cfg(windows)]
fn parse_options(args: &[String]) -> Result<Options, ArgError> {
    let mut options = Options::default();
    let mut remaining = args.iter().skip(3);
    while let Some(arg) = remaining.next() {
        match arg.as_str() {
            "--input" | "-i" => options.input = Some(remaining.next().ok_or(ArgError::MissingInputValue)?.clone()),
            "--what-if" | "-w" => options.what_if = true,
            unknown => return Err(ArgError::UnknownOption(unknown.to_string())),
        }
    }
    Ok(options)
}

#[cfg(all(test, windows))]
mod tests {
    use super::{ArgError, Options, parse_options};

    fn parse(list: &[&str]) -> Result<Options, ArgError> {
        let args: Vec<String> = list.iter().map(ToString::to_string).collect();
        parse_options(&args)
    }

    fn options(input: Option<&str>, what_if: bool) -> Options {
        Options { input: input.map(ToString::to_string), what_if }
    }

    #[test]
    fn accepts_input_and_what_if_in_any_order() {
        assert_eq!(parse(&["exe", "get", "user-right", "--input", "{}"]), Ok(options(Some("{}"), false)));
        assert_eq!(parse(&["exe", "set", "user-right", "-w", "-i", "[]"]), Ok(options(Some("[]"), true)));
        assert_eq!(parse(&["exe", "set", "user-right", "--input", "{}", "--what-if"]), Ok(options(Some("{}"), true)));
        assert_eq!(parse(&["exe", "export", "user-right"]), Ok(options(None, false)));
    }

    #[test]
    fn input_requires_a_value() {
        assert_eq!(parse(&["exe", "get", "user-right", "--input"]), Err(ArgError::MissingInputValue));
        assert_eq!(parse(&["exe", "set", "user-right", "-w", "-i"]), Err(ArgError::MissingInputValue));
    }

    #[test]
    fn unknown_options_are_rejected_before_and_after_input() {
        let unknown = |option: &str| Err(ArgError::UnknownOption(option.to_string()));
        assert_eq!(parse(&["exe", "set", "user-right", "--whatif", "--input", "{}"]), unknown("--whatif"));
        assert_eq!(parse(&["exe", "set", "user-right", "--input", "{}", "--whatif"]), unknown("--whatif"));
        assert_eq!(parse(&["exe", "set", "user-right", "--input", "{}", "extra"]), unknown("extra"));
        assert_eq!(parse(&["exe", "export", "user-right", "-x"]), unknown("-x"));
    }

    #[test]
    fn the_value_of_input_is_never_read_as_an_option() {
        assert_eq!(parse(&["exe", "set", "user-right", "--input", "--what-if"]), Ok(options(Some("--what-if"), false)));
    }
}
