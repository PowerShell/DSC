// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use super::Function;
use rust_i18n::t;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct If {}

impl Function for If {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "if".to_string(),
            description: t!("functions.if.description").to_string(),
            category: vec![FunctionCategory::Logical],
            min_args: 3,
            max_args: 3,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Boolean],
                vec![FunctionArgKind::String, FunctionArgKind::Number, FunctionArgKind::Array, FunctionArgKind::Object],
                vec![FunctionArgKind::String, FunctionArgKind::Number, FunctionArgKind::Array, FunctionArgKind::Object],
                ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String, FunctionArgKind::Number, FunctionArgKind::Array, FunctionArgKind::Object],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let Some(condition) = args[0].as_bool() else {
            return Err(DscError::Function("if".to_string(), t!("functions.if.conditionNotBoolean").to_string()));
        };

        if condition {
            Ok(args[1].clone())
        } else {
            Ok(args[2].clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn invalid_condition() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[if('PATH', 1 , 2)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn condition_true() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[if(true, 'left', 'right')]", &Context::new()).unwrap();
        assert_eq!(result, "left");
    }

    #[test]
    fn condition_false() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[if(false, 'left', 'right')]", &Context::new()).unwrap();
        assert_eq!(result, "right");
    }
}
