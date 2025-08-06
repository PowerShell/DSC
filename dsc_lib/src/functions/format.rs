// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rt_format::{Format as RtFormat, FormatArgument, ParsedFormat, argument::NoNamedArguments};
use rust_i18n::t;
use serde_json::Value;
use tracing::warn;

#[derive(Debug, PartialEq)]
enum Variant {
    Boolean(bool),
    Number(i64),
    String(String),
}

impl FormatArgument for Variant {
    fn supports_format(&self, specifier: &rt_format::Specifier) -> bool {
        match self {
            Variant::Boolean(_) | Variant::String(_)=> matches!(specifier.format, RtFormat::Display),
            Variant::Number(_) => matches!(specifier.format, RtFormat::Display | RtFormat::Binary | RtFormat::Octal | RtFormat::LowerHex | RtFormat::UpperHex | RtFormat::Debug | RtFormat::LowerExp | RtFormat::UpperExp),
        }
    }

    fn fmt_display(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Variant::Boolean(value) => write!(f, "{value}"),
            Variant::Number(value) => write!(f, "{value}"),
            Variant::String(value) => write!(f, "{value}"),
        }
    }

    fn fmt_octal(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Variant::Number(value) => write!(f, "{value:o}"),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_lower_hex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Variant::Number(value) => write!(f, "{value:x}"),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_upper_hex(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Variant::Number(value) => write!(f, "{value:X}"),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_binary(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Variant::Number(value) => write!(f, "{value:b}"),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_debug(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Err(std::fmt::Error)
    }

    fn fmt_lower_exp(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Variant::Number(value) => write!(f, "{value:e}"),
            _ => Err(std::fmt::Error),
        }
    }

    fn fmt_upper_exp(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Variant::Number(value) => write!(f, "{value:E}"),
            _ => Err(std::fmt::Error),
        }
    }
}

#[derive(Debug, Default)]
pub struct Format {}

impl Function for Format {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "format".to_string(),
            description: t!("functions.format.description").to_string(),
            category: FunctionCategory::String,
            min_args: 2,
            max_args: usize::MAX,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: Some(vec![
                FunctionArgKind::Boolean,
                FunctionArgKind::Number,
                FunctionArgKind::String,
            ]),
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        warn!("{}", t!("functions.format.experimental"));
        let Some(format_string) = args[0].as_str() else {
            return Err(DscError::Parser(t!("functions.format.formatInvalid").to_string()));
        };
        let mut position_args =     Vec::new();
        for value in &args[1..] {
            let arg = match value {
                Value::Bool(b) => Variant::Boolean(*b),
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Variant::Number(i)
                    } else {
                        return Err(DscError::Parser(t!("functions.format.numberTooLarge").to_string()));
                    }
                }
                Value::String(s) => Variant::String(s.clone()),
                _ => return Err(DscError::Parser(t!("functions.format.invalidArgType").to_string())),
            };
            position_args.push(arg);
        }
        let string_result = match ParsedFormat::parse(format_string, &position_args, &NoNamedArguments) {
            Ok(parsed_format) => format!("{parsed_format}"),
            Err(_e) => return Err(DscError::Parser(t!("functions.format.invalidFormatString").to_string())),
        };
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
        let result = parser.parse_and_execute("[format('world {0} - {1}', 'hello', 2)]", &Context::new(), true).unwrap();
        assert_eq!(result, "world hello - 2");
    }

    #[test]
    fn reverse_position() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('two{1} - {0}world', 'hello', 2)]", &Context::new(), true).unwrap();
        assert_eq!(result, "two2 - helloworld");
    }

    #[test]
    fn repeated_position() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{0} - {0}{1}', 'hello', 2)]", &Context::new(), true).unwrap();
        assert_eq!(result, "hello - hello2");
    }

    #[test]
    fn numbers_as_hex() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{0:x} = {1:X}', 12, 13)]", &Context::new(), true).unwrap();
        assert_eq!(result, "c = D");
    }

    #[test]
    fn numbers_as_octal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{0:o} == {1:o}', 12, 13)]", &Context::new(), true).unwrap();
        assert_eq!(result, "14 == 15");
    }

    #[test]
    fn numbers_as_binary() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{0:b} = {1:b}', 12, 13)]", &Context::new(), true).unwrap();
        assert_eq!(result, "1100 = 1101");
    }

    #[test]
    fn numbers_as_exp() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{0:e} = {1:E}', 12, 13)]", &Context::new(), true).unwrap();
        assert_eq!(result, "1.2e1 = 1.3E1");
    }

    #[test]
    fn numbers_as_display_just_one() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('hello {0} there', 12, 13)]", &Context::new(), true).unwrap();
        assert_eq!(result, "hello 12 there");
    }

    #[test]
    fn string_as_octal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{0:o} = {1:O}', 'hello', 13)]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn bool_as_octal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{0:o} = {1:O}', true, 13)]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn string_as_hex() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{0:x} = {1:X}', 'hello', 13)]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn bool_as_hex() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{0:x} = {1:X}', true, 13)]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn string_as_binary() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{1:b} = {0:B}', 'hello', 13)]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn bool_as_binary() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{1:b} = {0:B}', true, 13)]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn string_as_exp() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{1:e} = {0:E}', 'hello', 13)]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn bool_as_exp() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('{1:e} = {0:E}', true, 13)]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn args_out_of_bounds() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('hello {1} {2} there', 12, 13)]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn missing_closing_brace() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('hello {0 there', 12, 13)]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn missing_opening_brace() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('hello 0} there', 12, 13)]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_format_option() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('hello {0:invalid} there', 12, 13)]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_index_syntax() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('hello {0;x} there', 12, 13)]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn missing_format_type() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[format('hello {0:} there', 12, 13)]", &Context::new(), true).unwrap();
        assert_eq!(result, "hello 12 there");
    }
}
