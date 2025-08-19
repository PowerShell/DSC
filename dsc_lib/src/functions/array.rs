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
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String, FunctionArgKind::Number, FunctionArgKind::Object, FunctionArgKind::Array],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Array],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.array.invoked"));
        
        let value = &args[0];
        
        if value.is_number() || value.is_string() || value.is_array() || value.is_object() {
            Ok(Value::Array(vec![value.clone()]))
        } else {
            Err(DscError::Parser(t!("functions.array.invalidArgType").to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn single_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[array('hello')]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), r#"["hello"]"#);
    }

    #[test]
    fn single_number() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[array(42)]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "[42]");
    }

    #[test]
    fn single_object() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[array(createObject('key', 'value'))]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), r#"[{"key":"value"}]"#);
    }

    #[test]
    fn single_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[array(createArray('a','b'))]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), r#"[["a","b"]]"#);
    }

    #[test]
    fn empty_array_not_allowed() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[array()]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn multiple_args_not_allowed() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[array('hello', 42)]", &Context::new());
        assert!(result.is_err());
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
