// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use super::Function;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct Equals {}

impl Function for Equals {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "equals".to_string(),
            description: t!("functions.equals.description").to_string(),
            category: vec![FunctionCategory::Comparison],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Null, FunctionArgKind::Number, FunctionArgKind::String, FunctionArgKind::Array, FunctionArgKind::Object],
                vec![FunctionArgKind::Null, FunctionArgKind::Number, FunctionArgKind::String, FunctionArgKind::Array, FunctionArgKind::Object],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        Ok(Value::Bool(args[0] == args[1]))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::Value;

    #[test]
    fn int_equal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[equals(1,1)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn int_notequal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[equals(1,2)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn string_equal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[equals('test','test')]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn string_notequal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[equals('test','TEST')]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn different_types() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[equals(1,'string')]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn null_equal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[equals(null(),null())]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // TODO: Add tests for arrays once `createArray()` is implemented
    // TODO: Add tests for objects once `createObject()` is implemented
}
