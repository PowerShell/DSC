// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Last {}

impl Function for Last {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "last".to_string(),
            description: t!("functions.last.description").to_string(),
            category: vec![FunctionCategory::Array, FunctionCategory::String],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::Array, FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String, FunctionArgKind::Number, FunctionArgKind::Array, FunctionArgKind::Object],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.last.invoked"));
        
        if let Some(array) = args[0].as_array() {
            if array.is_empty() {
                return Err(DscError::Parser(t!("functions.last.emptyArray").to_string()));
            }
            return Ok(array[array.len() - 1].clone());
        }

        if let Some(string) = args[0].as_str() {
            if string.is_empty() {
                return Err(DscError::Parser(t!("functions.last.emptyString").to_string()));
            }
            return Ok(Value::String(string.chars().last().unwrap().to_string()));
        }

        Err(DscError::Parser(t!("functions.last.invalidArgType").to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn array_of_strings() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[last(createArray('hello', 'world'))]", &Context::new()).unwrap();
        assert_eq!(result.as_str(), Some("world"));
    }

    #[test]
    fn array_of_numbers() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[last(createArray(1, 2, 3))]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "3");
    }

    #[test]
    fn array_of_single_element() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[last(array('hello'))]", &Context::new()).unwrap();
        assert_eq!(result.as_str(), Some("hello"));
    }

    #[test]
    fn string_input() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[last('hello')]", &Context::new()).unwrap();
        assert_eq!(result.as_str(), Some("o"));
    }

    #[test]
    fn single_character_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[last('a')]", &Context::new()).unwrap();
        assert_eq!(result.as_str(), Some("a"));
    }

    #[test]
    fn empty_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[last(createArray())]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn empty_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[last('')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_type_object() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[last(createObject('key', 'value'))]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_type_number() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[last(42)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn array_of_multiple_strings() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[last(createArray('text', 'middle', 'last'))]", &Context::new()).unwrap();
        assert_eq!(result.as_str(), Some("last"));
    }

    #[test]
    fn unicode_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[last('Hello🌍')]", &Context::new()).unwrap();
        assert_eq!(result.as_str(), Some("🌍"));
    }
}
