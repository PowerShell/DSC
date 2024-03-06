// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Div {}

impl Function for Div {
    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        2
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::Number]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("div function");
        if let Some(value) = args[0].as_i64().unwrap().checked_div(args[1].as_i64().unwrap()) {
            Ok(Value::Number(value.into()))
        } else {
            Err(DscError::Parser("Cannot divide by zero".to_string()))
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

