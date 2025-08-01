// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct GreaterOrEquals {}

impl Function for GreaterOrEquals {
    fn description(&self) -> String {
        t!("functions.greaterOrEquals.description").to_string()
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
        vec![AcceptedArgKind::Number, AcceptedArgKind::String]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.greaterOrEquals.invoked"));
        
        let first = &args[0];
        let second = &args[1];

        if let (Some(num1), Some(num2)) = (first.as_i64(), second.as_i64()) {
            return Ok(Value::Bool(num1 >= num2));
        }

        if let (Some(str1), Some(str2)) = (first.as_str(), second.as_str()) {
            return Ok(Value::Bool(str1 >= str2));
        }

        Err(DscError::Parser(t!("functions.typeMismatch").to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn number_greater_or_equals() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[greaterOrEquals(5,3)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn number_not_greater_or_equals() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[greaterOrEquals(3,5)]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn number_equal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[greaterOrEquals(5,5)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn string_greater_or_equals() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[greaterOrEquals('b','a')]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn type_mismatch_string_number() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[greaterOrEquals('5', 3)]", &Context::new());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Arguments must be of the same type"));
    }
}
