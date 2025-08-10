// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Array {}

impl Function for Array {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "array".to_string(),
            description: t!("functions.array.description").to_string(),
            category: FunctionCategory::Array,
            min_args: 0,
            max_args: usize::MAX,
            accepted_arg_ordered_types: vec![],
            remaining_arg_accepted_types: Some(vec![
                FunctionArgKind::String,
                FunctionArgKind::Number,
                FunctionArgKind::Object,
                FunctionArgKind::Array,
            ]),
            return_types: vec![FunctionArgKind::Array],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.array.invoked"));
        let mut array_result = Vec::<Value>::new();
        
        for value in args {
            // Only accept int, string, array, or object as specified
            if value.is_number() || value.is_string() || value.is_array() || value.is_object() {
                array_result.push(value.clone());
            } else {
                return Err(DscError::Parser(t!("functions.array.invalidArgType").to_string()));
            }
        }

        Ok(Value::Array(array_result))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn mixed_types() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[array('hello', 42)]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), r#"["hello",42]"#);
    }

    #[test]
    fn strings_only() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[array('a', 'b', 'c')]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), r#"["a","b","c"]"#);
    }

    #[test]
    fn numbers_only() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[array(1, 2, 3)]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "[1,2,3]");
    }

    #[test]
    fn arrays_and_objects() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[array(createArray('a','b'), createObject('key', 'value'))]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), r#"[["a","b"],{"key":"value"}]"#);
    }

    #[test]
    fn empty_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[array()]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "[]");
    }

    #[test]
    fn invalid_type_boolean() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[array(true)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_type_null() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[array(null())]", &Context::new());
        assert!(result.is_err());
    }
}
