// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Mod {}

impl Function for Mod {
    fn description(&self) -> String {
        t!("functions.mod.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Numeric
    }

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
        debug!("mod function");
        if let (Some(arg1), Some(arg2)) = (args[0].as_i64(), args[1].as_i64()) {
            if arg2 == 0 {
                return Err(DscError::Parser(t!("functions.mod.divideByZero").to_string()));
            }
            Ok(Value::Number((arg1 % arg2).into()))
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
        let result = parser.parse_and_execute("[mod(7, 3)]", &Context::new()).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn nested() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[mod(18, mod(8, 3))]", &Context::new()).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn invalid_one_parameter() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[mod(5)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_div_by_zero() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[mod(5, 0)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn overflow_input() {
        let mut parser = Statement::new().unwrap();
        // max value for i64 is 2^63 -1 (or 9,223,372,036,854,775,807)
        let result = parser.parse_and_execute("[mod(9223372036854775808, 2)]", &Context::new());
        assert!(result.is_err());
    }
}
