// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Add {}

impl Function for Add {
    fn description(&self) -> String {
        t!("functions.add.description").to_string()
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
        debug!("{}", t!("functions.add.invoked"));
        if let (Some(arg1), Some(arg2)) = (args[0].as_i64(), args[1].as_i64()) {
            Ok(Value::Number((arg1 + arg2).into()))
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
        let result = parser.parse_and_execute("[add(2, 3)]", &Context::new()).unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn nested() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[add(2, add(3, 4))]", &Context::new()).unwrap();
        assert_eq!(result, 9);
    }

    #[test]
    fn invalid_one_parameter() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[add(5)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn overflow_result() {
        let mut parser = Statement::new().unwrap();
        // max value for i64 is 2^63 -1 (or 9,223,372,036,854,775,807)
        let result = parser.parse_and_execute("[add(9223372036854775807, 2)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn overflow_input() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[add(9223372036854775808, 2)]", &Context::new());
        assert!(result.is_err());
    }
}
