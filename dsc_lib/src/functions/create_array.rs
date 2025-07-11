// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct CreateArray {}

impl Function for CreateArray {
    fn description(&self) -> String {
        t!("functions.createArray.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Array
    }

    fn min_args(&self) -> usize {
        0
    }

    fn max_args(&self) -> usize {
        usize::MAX
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String, AcceptedArgKind::Number, AcceptedArgKind::Object, AcceptedArgKind::Array]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.createArray.invoked"));
        let mut array_result = Vec::<Value>::new();
        let mut input_type : Option<AcceptedArgKind> = None;
        for value in args {
            if value.is_array() {
                if input_type.is_none() {
                    input_type = Some(AcceptedArgKind::Array);
                } else if input_type != Some(AcceptedArgKind::Array) {
                    return Err(DscError::Parser(t!("functions.createArray.argsMustAllBeArrays").to_string()));
                }
            } else if value.is_number() {
                if input_type.is_none() {
                    input_type = Some(AcceptedArgKind::Number);
                } else if input_type != Some(AcceptedArgKind::Number) {
                    return Err(DscError::Parser(t!("functions.createArray.argsMustAllBeIntegers").to_string()));
                }
            } else if value.is_object() {
                if input_type.is_none() {
                    input_type = Some(AcceptedArgKind::Object);
                } else if input_type != Some(AcceptedArgKind::Object) {
                    return Err(DscError::Parser(t!("functions.createArray.argsMustAllBeObjects").to_string()));
                }
            } else if value.is_string() {
                if input_type.is_none() {
                    input_type = Some(AcceptedArgKind::String);
                } else if input_type != Some(AcceptedArgKind::String) {
                    return Err(DscError::Parser(t!("functions.createArray.argsMustAllBeStrings").to_string()));
                }
            } else {
                return Err(DscError::Parser(t!("functions.invalidArgType").to_string()));
            }
            array_result.push(value.clone());
        }

        Ok(Value::Array(array_result))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn strings() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[createArray('a', 'b')]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), r#"["a","b"]"#);
    }

    #[test]
    fn integers() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[createArray(1,2,3)]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "[1,2,3]");
    }

    #[test]
    fn arrays() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[createArray(createArray('a','b'), createArray('c','d'))]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), r#"[["a","b"],["c","d"]]"#);
    }

    #[test]
    fn objects() {
        // TODO
    }

    #[test]
    fn mixed_types() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[createArray(1,'a')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn empty() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[createArray()]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "[]");
    }
}
