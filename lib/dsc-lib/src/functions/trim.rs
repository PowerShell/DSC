// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use super::Function;

#[derive(Debug, Default)]
pub struct Trim {}

impl Function for Trim {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "trim".to_string(),
            description: t!("functions.trim.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let string_to_trim = args[0].as_str().unwrap();
        let result = string_to_trim.trim();
        Ok(Value::String(result.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn test_trim_leading_spaces() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[trim('   hello')]", &Context::new()).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_trim_trailing_spaces() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[trim('hello   ')]", &Context::new()).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_trim_both_sides() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[trim('  hello world  ')]", &Context::new()).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_trim_no_whitespace() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[trim('hello')]", &Context::new()).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_trim_empty_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[trim('')]", &Context::new()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_trim_only_whitespace() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[trim('   ')]", &Context::new()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_trim_tabs_and_newlines() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[trim('\t\nhello\n\t')]", &Context::new()).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_trim_internal_spaces_preserved() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[trim('  hello  world  ')]", &Context::new()).unwrap();
        assert_eq!(result, "hello  world");
    }

    #[test]
    fn test_trim_mixed_whitespace() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[trim(' \t\n  hello  \n\t ')]", &Context::new()).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_trim_unicode_text() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[trim('  café  ')]", &Context::new()).unwrap();
        assert_eq!(result, "café");
    }

    #[test]
    fn test_trim_with_nested_function() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[trim(concat('  hello', '  '))]", &Context::new()).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_trim_single_character() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[trim(' a ')]", &Context::new()).unwrap();
        assert_eq!(result, "a");
    }
}
