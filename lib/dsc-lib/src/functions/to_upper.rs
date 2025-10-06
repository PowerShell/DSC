// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use super::Function;

#[derive(Debug, Default)]
pub struct ToUpper {}

impl Function for ToUpper {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "toUpper".to_string(),
            description: t!("functions.toUpper.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let string_to_change = args[0].as_str().unwrap();
        let result = string_to_change.to_uppercase();
        Ok(Value::String(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn test_to_upper_lowercase() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[toUpper('hello world')]", &Context::new()).unwrap();
        assert_eq!(result, "HELLO WORLD");
    }

    #[test]
    fn test_to_upper_mixed_case() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[toUpper('Hello World')]", &Context::new()).unwrap();
        assert_eq!(result, "HELLO WORLD");
    }

    #[test]
    fn test_to_upper_already_uppercase() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[toUpper('HELLO WORLD')]", &Context::new()).unwrap();
        assert_eq!(result, "HELLO WORLD");
    }

    #[test]
    fn test_to_upper_empty_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[toUpper('')]", &Context::new()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_to_upper_with_numbers_and_symbols() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[toUpper('Hello123!@#')]", &Context::new()).unwrap();
        assert_eq!(result, "HELLO123!@#");
    }

    #[test]
    fn test_to_upper_unicode() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[toUpper('café')]", &Context::new()).unwrap();
        assert_eq!(result, "CAFÉ");
    }

    #[test]
    fn test_to_upper_with_spaces() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[toUpper('  hello  world  ')]", &Context::new()).unwrap();
        assert_eq!(result, "  HELLO  WORLD  ");
    }

    #[test]
    fn test_to_upper_single_character() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[toUpper('a')]", &Context::new()).unwrap();
        assert_eq!(result, "A");
    }

    #[test]
    fn test_to_upper_nested_function() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[toUpper(concat('hello', ' world'))]", &Context::new()).unwrap();
        assert_eq!(result, "HELLO WORLD");
    }
}