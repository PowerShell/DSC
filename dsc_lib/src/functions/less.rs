// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Less {}

impl Function for Less {
    fn description(&self) -> String {
        t!("functions.less.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Comparison
    }

    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        2
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::Number, AcceptedArgKind::Number]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.less.invoked"));
        
        let num1 = match &args[0] {
            Value::Number(n) => n.as_f64().ok_or_else(|| DscError::Parser(t!("functions.invalidArguments").to_string()))?,
            _ => return Err(DscError::Parser(t!("functions.invalidArguments").to_string())),
        };

        let num2 = match &args[1] {
            Value::Number(n) => n.as_f64().ok_or_else(|| DscError::Parser(t!("functions.invalidArguments").to_string()))?,
            _ => return Err(DscError::Parser(t!("functions.invalidArguments").to_string())),
        };

        Ok(Value::Bool(num1 < num2))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn number_less() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[less(3,5)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn number_not_less() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[less(5,3)]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn number_equal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[less(5,5)]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }
}
