// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct First {}

impl Function for First {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "first".to_string(),
            description: t!("functions.first.description").to_string(),
            category: vec![FunctionCategory::Array, FunctionCategory::String],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::Array, FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String, FunctionArgKind::Number, FunctionArgKind::Array, FunctionArgKind::Object],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.first.invoked"));
        
        if let Some(array) = args[0].as_array() {
            if array.is_empty() {
                return Err(DscError::Parser(t!("functions.first.emptyArray").to_string()));
            }
            return Ok(array[0].clone());
        }

        if let Some(string) = args[0].as_str() {
            if string.is_empty() {
                return Err(DscError::Parser(t!("functions.first.emptyString").to_string()));
            }
            return Ok(Value::String(string.chars().next().unwrap().to_string()));
        }

        Err(DscError::Parser(t!("functions.first.invalidArgType").to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn array_of_strings() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[first(createArray('hello', 'world'))]", &Context::new()).unwrap();
        assert_eq!(result.as_str(), Some("hello"));
    }

    #[test]
    fn array_of_numbers() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[first(createArray(1, 2, 3))]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "1");
    }

    #[test]
    fn array_of_single_element() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[first(array('hello'))]", &Context::new()).unwrap();
        assert_eq!(result.as_str(), Some("hello"));
    }

    #[test]
    fn string_input() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[first('hello')]", &Context::new()).unwrap();
        assert_eq!(result.as_str(), Some("h"));
    }

    #[test]
    fn single_character_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[first('a')]", &Context::new()).unwrap();
        assert_eq!(result.as_str(), Some("a"));
    }

    #[test]
    fn empty_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[first(createArray())]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn empty_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[first('')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_type_object() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[first(createObject('key', 'value'))]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_type_number() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[first(42)]", &Context::new());
        assert!(result.is_err());
    }
}
