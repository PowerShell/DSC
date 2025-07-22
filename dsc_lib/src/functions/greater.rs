// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, FunctionCategory};
use rust_i18n::t;
use super::Function;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct Greater {}

impl Function for Greater {
    fn description(&self) -> String {
        t!("functions.greater.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Comparison
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::Number, AcceptedArgKind::String]
    }

    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        2
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let first = &args[0];
        let second = &args[1];

        if let (Some(num1), Some(num2)) = (first.as_f64(), second.as_f64()) {
            return Ok(Value::Bool(num1 > num2));
        }

        if let (Some(str1), Some(str2)) = (first.as_str(), second.as_str()) {
            return Ok(Value::Bool(str1 > str2));
        }

        Ok(Value::Bool(false))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::Value;

    #[test]
    fn number_greater() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[greater(2,1)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn number_not_greater() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[greater(1,2)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn number_equal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[greater(1,1)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }
}
