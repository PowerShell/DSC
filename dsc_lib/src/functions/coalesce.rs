// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Coalesce {}

impl Function for Coalesce {
    fn description(&self) -> String {
        t!("functions.coalesce.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Logical
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> usize {
        usize::MAX
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![
            AcceptedArgKind::Array,
            AcceptedArgKind::Boolean,
            AcceptedArgKind::Number,
            AcceptedArgKind::Object,
            AcceptedArgKind::String,
        ]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.coalesce.invoked"));

        for arg in args {
            if !arg.is_null() {
                return Ok(arg.clone());
            }
        }

        Ok(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn first_non_null_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[coalesce(null, 'hello', 'world')]", &Context::new()).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn first_non_null_number() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[coalesce(null, null, 42)]", &Context::new()).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn first_non_null_boolean() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[coalesce(null, true)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn all_null_returns_null() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[coalesce(null, null, null)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn single_non_null_value() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[coalesce('first')]", &Context::new()).unwrap();
        assert_eq!(result, "first");
    }

    #[test]
    fn mixed_types() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[coalesce(null, 123, 'fallback')]", &Context::new()).unwrap();
        assert_eq!(result, 123);
    }
}
