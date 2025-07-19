// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Bool {}

impl Function for Bool {
    fn description(&self) -> String {
        t!("functions.bool.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Logical
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> usize {
        1
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String, AcceptedArgKind::Number]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.bool.invoked"));
        if let Some(arg) = args[0].as_str() {
            match arg.to_lowercase().as_str() {
                "true" => Ok(Value::Bool(true)),
                "false" => Ok(Value::Bool(false)),
                _ => Err(DscError::Parser(t!("functions.invalidArguments").to_string())),
            }
        } else if let Some(num) = args[0].as_i64() {
            Ok(Value::Bool(num != 0))
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
    fn true_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[bool('true')]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn false_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[bool('false')]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn number_1() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[bool(1)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn number_0() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[bool(0)]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }
}
