// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, FunctionCategory};
use super::Function;
use rust_i18n::t;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct If {}

impl Function for If {
    fn description(&self) -> String {
        t!("functions.if.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Comparison
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::Boolean, AcceptedArgKind::String, AcceptedArgKind::Number, AcceptedArgKind::Array, AcceptedArgKind::Object]
    }

    fn min_args(&self) -> usize {
        3
    }

    fn max_args(&self) -> usize {
        3
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
