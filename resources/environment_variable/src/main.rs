// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod types;

#[cfg(windows)]
mod environment;

use rust_i18n::t;
use std::process::exit;
use types::{
    EnvironmentPathVariable, EnvironmentVariable, EnvironmentVariableFilterList,
    EnvironmentVariableList, Operation,
};

rust_i18n::i18n!("locales", fallback = "en-us");

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_ARGS: i32 = 1;
const EXIT_INVALID_INPUT: i32 = 2;
const EXIT_RESOURCE_ERROR: i32 = 3;
const EXIT_ELEVATION_REQUIRED: i32 = 4;

#[derive(Clone, Copy)]
enum InputKind {
    Scalar,
    Path,
    List,
}

fn write_error(message: &str) {
    eprintln!("{}", serde_json::json!({ "error": message }));
}

fn print_json(value: &impl serde::Serialize) {
    match serde_json::to_string(value) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            write_error(&t!("main.serializeError", error = error.to_string()));
            exit(EXIT_RESOURCE_ERROR);
        }
    }
}

fn parse_list(
    input_json: Option<String>,
    operation: Operation,
    kind: InputKind,
) -> Result<EnvironmentVariableList, (String, i32)> {
    let json =
        input_json.ok_or_else(|| (t!("main.missingInput").to_string(), EXIT_INVALID_ARGS))?;

    let input = match kind {
        InputKind::List => serde_json::from_str::<EnvironmentVariableList>(&json),
        InputKind::Scalar => {
            serde_json::from_str::<EnvironmentVariable>(&json).map(EnvironmentVariableList::from)
        }
        InputKind::Path => serde_json::from_str::<EnvironmentPathVariable>(&json)
            .map(EnvironmentVariableList::from),
    }
    .map_err(|error| {
        (
            t!("main.invalidJson", error = error.to_string()).to_string(),
            EXIT_INVALID_INPUT,
        )
    })?;

    input
        .validate(operation)
        .map_err(|error| (error, EXIT_INVALID_INPUT))?;
    Ok(input)
}

fn parse_export_filters(
    input_json: Option<String>,
) -> Result<EnvironmentVariableFilterList, (String, i32)> {
    let Some(json) = input_json else {
        return Ok(EnvironmentVariableFilterList::default());
    };
    let input = serde_json::from_str::<EnvironmentVariableFilterList>(&json).map_err(|error| {
        (
            t!("main.invalidJson", error = error.to_string()).to_string(),
            EXIT_INVALID_INPUT,
        )
    })?;
    input
        .validate()
        .map_err(|error| (error, EXIT_INVALID_INPUT))?;
    Ok(input)
}

fn serialize_result(
    mut value: EnvironmentVariableList,
    kind: InputKind,
) -> Result<serde_json::Value, String> {
    if matches!(kind, InputKind::List) {
        return serde_json::to_value(value)
            .map_err(|error| t!("main.serializeError", error = error.to_string()).to_string());
    }

    let variable = value
        .environment_variables
        .pop()
        .ok_or_else(|| t!("main.missingState").to_string())?;
    let mut output = serde_json::to_value(variable)
        .map_err(|error| t!("main.serializeError", error = error.to_string()).to_string())?;
    if let Some(in_desired_state) = value.in_desired_state {
        output["_inDesiredState"] = serde_json::Value::Bool(in_desired_state);
    }
    Ok(output)
}

fn print_result(value: EnvironmentVariableList, kind: InputKind) {
    match serialize_result(value, kind) {
        Ok(output) => print_json(&output),
        Err(error) => {
            write_error(&error);
            exit(EXIT_RESOURCE_ERROR);
        }
    }
}

#[cfg(not(windows))]
fn main() {
    write_error(&t!("main.windowsOnly"));
    exit(EXIT_RESOURCE_ERROR);
}

#[cfg(windows)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        write_error(&t!("main.missingOperation"));
        exit(EXIT_INVALID_ARGS);
    }

    let operation = args[1].as_str();
    let input_json = parse_input_arg(&args);
    let kind = if args.iter().any(|arg| arg == "--list") {
        InputKind::List
    } else if args.iter().any(|arg| arg == "--path") {
        InputKind::Path
    } else {
        InputKind::Scalar
    };

    let input = match operation {
        "get" => parse_list(input_json, Operation::Get, kind)
            .map(|input| environment::get_variables(&input)),
        "set" => parse_list(input_json, Operation::Set, kind)
            .map(|input| environment::set_variables(&input)),
        "test" => parse_list(input_json, Operation::Test, kind)
            .map(|input| environment::test_variables(&input)),
        "export" if matches!(kind, InputKind::List) => {
            parse_export_filters(input_json).map(|input| environment::export_variables(&input))
        }
        _ => {
            write_error(&t!("main.unknownOperation", operation = operation));
            exit(EXIT_INVALID_ARGS);
        }
    };
    let result = match input {
        Ok(result) => result,
        Err((error, exit_code)) => {
            write_error(&error);
            exit(exit_code);
        }
    };

    match result {
        Ok(value) => {
            print_result(value, kind);
            exit(EXIT_SUCCESS);
        }
        Err(error) => {
            write_error(&error.to_string());
            exit(if error.is_elevation_required() {
                EXIT_ELEVATION_REQUIRED
            } else {
                EXIT_RESOURCE_ERROR
            });
        }
    }
}

fn parse_input_arg(args: &[String]) -> Option<String> {
    let mut index = 2;
    while index < args.len() {
        if args[index] == "--input" || args[index] == "-i" {
            if index + 1 < args.len() {
                return Some(args[index + 1].clone());
            }
            write_error(&t!("main.missingInputValue"));
            exit(EXIT_INVALID_ARGS);
        }
        index += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{InputKind, parse_export_filters, parse_input_arg, parse_list, serialize_result};
    use crate::types::{EnvironmentVariableItem, Operation};

    #[test]
    fn parses_each_input_kind() {
        let scalar = parse_list(
            Some(r#"{"name":"Scalar","value":"text"}"#.to_string()),
            Operation::Set,
            InputKind::Scalar,
        )
        .unwrap();
        assert!(matches!(
            scalar.environment_variables[0],
            EnvironmentVariableItem::Scalar(_)
        ));

        let path = parse_list(
            Some(r#"{"name":"Path","value":["one"],"delimiter":"::"}"#.to_string()),
            Operation::Set,
            InputKind::Path,
        )
        .unwrap();
        assert!(matches!(
            path.environment_variables[0],
            EnvironmentVariableItem::Path(_)
        ));

        let list = parse_list(
            Some(
                r#"{"environmentVariables":[{"name":"Scalar","value":"text"},{"name":"Path","value":["one"]}]}"#
                    .to_string(),
            ),
            Operation::Test,
            InputKind::List,
        )
        .unwrap();
        assert_eq!(list.environment_variables.len(), 2);
    }

    #[test]
    fn reports_missing_invalid_and_failed_validation() {
        assert_eq!(
            parse_list(None, Operation::Get, InputKind::Scalar)
                .unwrap_err()
                .1,
            super::EXIT_INVALID_ARGS
        );
        assert_eq!(
            parse_list(
                Some("{invalid".to_string()),
                Operation::Get,
                InputKind::Scalar
            )
            .unwrap_err()
            .1,
            super::EXIT_INVALID_INPUT
        );
        assert_eq!(
            parse_list(
                Some(r#"{"name":"MissingValue"}"#.to_string()),
                Operation::Set,
                InputKind::Scalar
            )
            .unwrap_err()
            .1,
            super::EXIT_INVALID_INPUT
        );
    }

    #[test]
    fn parses_and_validates_export_filters() {
        assert!(
            parse_export_filters(None)
                .unwrap()
                .environment_variables
                .is_empty()
        );
        assert_eq!(
            parse_export_filters(Some("{invalid".to_string()))
                .unwrap_err()
                .1,
            super::EXIT_INVALID_INPUT
        );
        assert!(
            parse_export_filters(Some(
                r#"{"environmentVariables":[{"name":"*Path","value":[]}]}"#.to_string()
            ))
            .is_ok()
        );
        assert_eq!(
            parse_export_filters(Some(
                r#"{"environmentVariables":[{"name":"Path","delimiter":""}]}"#.to_string()
            ))
            .unwrap_err()
            .1,
            super::EXIT_INVALID_INPUT
        );
    }

    #[test]
    fn serializes_single_and_list_results() {
        let mut scalar = parse_list(
            Some(r#"{"name":"Scalar","value":"text"}"#.to_string()),
            Operation::Get,
            InputKind::Scalar,
        )
        .unwrap();
        scalar.in_desired_state = Some(true);
        let output = serialize_result(scalar, InputKind::Scalar).unwrap();
        assert_eq!(output["_inDesiredState"], true);
        assert_eq!(output["value"], "text");

        let list = parse_list(
            Some(r#"{"environmentVariables":[{"name":"Scalar"}]}"#.to_string()),
            Operation::Get,
            InputKind::List,
        )
        .unwrap();
        assert!(
            serialize_result(list, InputKind::List)
                .unwrap()
                .get("environmentVariables")
                .is_some()
        );

        assert!(
            serialize_result(
                crate::types::EnvironmentVariableList {
                    environment_variables: Vec::new(),
                    in_desired_state: None,
                },
                InputKind::Path,
            )
            .is_err()
        );
    }

    #[test]
    fn finds_long_and_short_input_arguments() {
        assert_eq!(
            parse_input_arg(&["exe".into(), "get".into(), "--input".into(), "{}".into()]),
            Some("{}".to_string())
        );
        assert_eq!(
            parse_input_arg(&["exe".into(), "get".into(), "-i".into(), "[]".into()]),
            Some("[]".to_string())
        );
        assert_eq!(parse_input_arg(&["exe".into(), "get".into()]), None);
    }
}
