// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::Function;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use crate::DscError;
use rust_i18n::t;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct ToLower {}

impl Function for ToLower {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "toLower".to_string(),
            description: t!("functions.toLower.description").to_string(),
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
        let result = string_to_change.to_lowercase();
        Ok(Value::String(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn test_to_lower_uppercase() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[toLower('HELLO WORLD')]", &Context::new())
            .unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_to_lower_mixed_case() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[toLower('Hello World')]", &Context::new())
            .unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_to_lower_already_lowercase() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[toLower('hello world')]", &Context::new())
            .unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_to_lower_empty_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[toLower('')]", &Context::new()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_to_lower_with_numbers_and_symbols() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[toLower('HELLO123!@#')]", &Context::new())
            .unwrap();
        assert_eq!(result, "hello123!@#");
    }

    #[test]
    fn test_to_lower_unicode() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[toLower('CAFÉ')]", &Context::new()).unwrap();
        assert_eq!(result, "café");
    }

    #[test]
    fn test_to_lower_with_spaces() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[toLower('  HELLO  WORLD  ')]", &Context::new())
            .unwrap();
        assert_eq!(result, "  hello  world  ");
    }

    #[test]
    fn test_to_lower_single_character() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[toLower('A')]", &Context::new()).unwrap();
        assert_eq!(result, "a");
    }

    #[test]
    fn test_to_lower_nested_function() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[toLower(concat('HELLO', ' WORLD'))]", &Context::new())
            .unwrap();
        assert_eq!(result, "hello world");
    }
}
