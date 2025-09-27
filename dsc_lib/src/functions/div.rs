// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Div {}

impl Function for Div {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "div".to_string(),
            description: t!("functions.div.description").to_string(),
            category: vec![FunctionCategory::Numeric],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Number],
                vec![FunctionArgKind::Number],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Number],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.div.invoked"));
        if let (Some(arg1), Some(arg2)) = (args[0].as_i64(), args[1].as_i64()) {
            if let Some(value) = arg1.checked_div(arg2) {
                Ok(Value::Number(value.into()))
            } else {
                Err(DscError::Parser(t!("functions.div.divideByZero").to_string()))
            }
        } else {
            Err(DscError::Parser(t!("functions.invalidArguments").to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn numbers() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[div(8, 3)]", &Context::new()).unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn nested() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[div(18, div(9, 3))]", &Context::new()).unwrap();
        assert_eq!(result, 6);
    }

    #[test]
    fn invalid_one_parameter() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[div(5)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_div_by_zero() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[div(5, 0)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn overflow_input() {
        let mut parser = Statement::new().unwrap();
        // max value for i64 is 2^63 -1 (or 9,223,372,036,854,775,807)
        let result = parser.parse_and_execute("[div(9223372036854775808, 2)]", &Context::new());
        assert!(result.is_err());
    }
}

