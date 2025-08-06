// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct CreateArray {}

impl Function for CreateArray {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "createArray".to_string(),
            description: t!("functions.createArray.description").to_string(),
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
        debug!("{}", t!("functions.createArray.invoked"));
        let mut array_result = Vec::<Value>::new();
        let mut input_type : Option<FunctionArgKind> = None;
        for value in args {
            if value.is_array() {
                if input_type.is_none() {
                    input_type = Some(FunctionArgKind::Array);
                } else if input_type != Some(FunctionArgKind::Array) {
                    return Err(DscError::Parser(t!("functions.createArray.argsMustAllBeArrays").to_string()));
                }
            } else if value.is_number() {
                if input_type.is_none() {
                    input_type = Some(FunctionArgKind::Number);
                } else if input_type != Some(FunctionArgKind::Number) {
                    return Err(DscError::Parser(t!("functions.createArray.argsMustAllBeIntegers").to_string()));
                }
            } else if value.is_object() {
                if input_type.is_none() {
                    input_type = Some(FunctionArgKind::Object);
                } else if input_type != Some(FunctionArgKind::Object) {
                    return Err(DscError::Parser(t!("functions.createArray.argsMustAllBeObjects").to_string()));
                }
            } else if value.is_string() {
                if input_type.is_none() {
                    input_type = Some(FunctionArgKind::String);
                } else if input_type != Some(FunctionArgKind::String) {
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
        let result = parser.parse_and_execute("[createArray('a', 'b')]", &Context::new(), true).unwrap();
        assert_eq!(result.to_string(), r#"["a","b"]"#);
    }

    #[test]
    fn integers() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[createArray(1,2,3)]", &Context::new(), true).unwrap();
        assert_eq!(result.to_string(), "[1,2,3]");
    }

    #[test]
    fn arrays() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[createArray(createArray('a','b'), createArray('c','d'))]", &Context::new(), true).unwrap();
        assert_eq!(result.to_string(), r#"[["a","b"],["c","d"]]"#);
    }

    #[test]
    fn objects() {
        // TODO
    }

    #[test]
    fn mixed_types() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[createArray(1,'a')]", &Context::new(), true);
        assert!(result.is_err());
    }

    #[test]
    fn empty() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[createArray()]", &Context::new(), true).unwrap();
        assert_eq!(result.to_string(), "[]");
    }
}
