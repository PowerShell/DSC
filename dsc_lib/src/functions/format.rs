// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use rt_format::{Format as RtFormat, FormatArgument, ParsedFormat, Specifier};
use rust_i18n::t;
use serde_json::Value;
use std::fmt;
use std::fmt::{Error, Formatter, Write};

impl FormatArgument for Value {
    fn supports_format(&self, spec: &Specifier) -> bool {
        match self {
            Value::Boolean(_) | Value::String(_) | Value::Number(_) => true,
            _ => false,
        }
    }

    fn fmt_display(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            _ => Err(fmt::Error),
        }
    }

    fn fmt_lower_hex(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{:x}", n.as_i64().unwrap_or_default()),
            _ => Err(fmt::Error),
        }
    }

    fn fmt_upper_hex(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{:X}", n.as_i64().unwrap_or_default()),
            _ => Err(fmt::Error),
        }
    }

    fn fmt_binary(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{:b}", n.as_i64().unwrap_or_default()),
            _ => Err(fmt::Error),
        }
    }

    fn fmt_octal(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{:o}", n.as_i64().unwrap_or_default()),
            _ => Err(fmt::Error),
        }
    }
}

#[derive(Debug, Default)]
pub struct Format {}

impl Function for Format {
    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        usize::MAX
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::Boolean, AcceptedArgKind::String, AcceptedArgKind::Number]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let mut string_result = String::new();
        let Ok(format_string) = args[0].as_str() else {
            return Err(DscError::Parser("First `format()` argument must be a string".to_string()));
        };
        for value in &args[1..] {
            if let Some(parsed_format) = ParsedFormat::parse(format_string) {
                let mut formatted_string = String::new();
                for specifier in parsed_format.specifiers() {
                    if let Some(arg) = args.get(specifier.index()) {
                        formatted_string.push_str(&arg.to_string());
                    } else {
                        return Err(DscError::Parser("Invalid format specifier".to_string()));
                    }
                }
                string_result.push_str(&formatted_string);
            } else {
                return Err(DscError::Parser("Invalid format string".to_string()));
            }
        }
        Ok(Value::String(string_result))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn position() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{0} - {1}', 'hello', 2)]", &Context::new()).unwrap();
        assert_eq!(result, "hello - 2");
    }

    #[test]
    fn numbers_as_hex() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{0:x} {0:X}', 12, 13)]", &Context::new()).unwrap();
        assert_eq!(result, "c D");
    }

}
