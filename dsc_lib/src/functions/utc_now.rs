// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use chrono::{SecondsFormat, Utc};
use rust_i18n::t;
use serde_json::Value;
use super::Function;
use tracing::debug;

#[derive(Debug, Default)]
pub struct UtcNow {}

impl Function for UtcNow {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "utcNow".to_string(),
            description: t!("functions.utcNow.description").to_string(),
            category: FunctionCategory::Date,
            min_args: 0,
            max_args: 1,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.utcNow.invoked"));

        if !context.processing_parameter_defaults {
            return Err(DscError::Parser(t!("functions.utcNow.onlyUsedAsParameterDefault").to_string()));
        }

        if args.is_empty() {
            // If no format is provided, return the current UTC time in ISO 8601 format
            let utc_now = Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true);
            return Ok(Value::String(utc_now));
        }

        if let Some(format) = args[0].as_str() {
            let converted_format = convert_dotnet_format_to_chrono(format);
            let utc_now = Utc::now().format(&converted_format);
            Ok(Value::String(utc_now.to_string()))
        } else {
            Err(DscError::Parser(t!("functions.invalidArguments").to_string()))
        }
    }
}

fn convert_dotnet_format_to_chrono(format: &str) -> String {
    const DOTNET_TO_CHRONO: &[(&str, &str)] = &[
        ("yyyy", "%Y"), // Full year, zero padded to 4 digits
        ("yy", "%y"), // Year, zero padded to 2 digits
        ("y", "%-Y"), // Year without leading zeroes
        ("dddd", "%A"), // Full weekday name
        ("ddd", "%a"), // Abbreviated weekday name
        ("dd", "%d"), // Day of the month, zero padded to 2 digits
        ("d", "%-d"), // Day of the month without leading zeroes
        ("HH", "%H"), // Hour in 24-hour format, zero padded to 2 digits
        ("H", "%-H"), // Hour in 24-hour format without leading zeroes
        ("mm", "%M"), // Minute, zero padded to 2 digits
        ("m", "%-M"), // Minute without leading zeroes
        ("ss", "%S"), // Second, zero padded to 2 digits
        ("s", "%-S"), // Second without leading zeroes
        ("fff", "%f"), // Milliseconds, zero padded to 3 digits
        ("MMMM", "%B"), // Full month name
        ("MMM", "%b"), // Abbreviated month name
        ("MM", "%m"), // Month, zero padded to 2 digits
        ("zzz", "%:z"), // Time zone offset without colon
        ("tt", "%p"), // AM/PM designator (same as t)
    ];

    let mut converted_format = String::new();
    // need to step through the format string character by character and see if it matches any of the dotnet formats, if so, we replace it with the chrono equivalent
    let mut start = 0;
    while start < format.len() {
        let mut matched = false;
        for (dotnet, chrono) in DOTNET_TO_CHRONO {
            if format[start..].starts_with(dotnet) {
                converted_format.push_str(chrono);
                start += dotnet.len();
                matched = true;
                break;
            }
        }
        if !matched {
            converted_format.push(format[start..].chars().next().unwrap());
            start += 1;
        }
    }

    converted_format
}
