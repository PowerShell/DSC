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
    use super::*;
    // TODO: Add tests for direct function calls with nulls and mixed types if the parser accept it
    // #[test]
    // fn all_null_returns_null() {
    //     let mut parser = Statement::new().unwrap();
    //     let result = parser.parse_and_execute("[coalesce(null, null, null)]", &Context::new()).unwrap();
    //     assert_eq!(result, serde_json::Value::Null);
    // }

    #[test]
    fn direct_function_call_with_nulls() {
        let coalesce = Coalesce {};
        let context = Context::new();
        
        let args = vec![Value::Null, Value::Null, Value::String("hello".to_string())];
        let result = coalesce.invoke(&args, &context).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
        
        let args = vec![Value::Null, Value::Null, Value::Null];
        let result = coalesce.invoke(&args, &context).unwrap();
        assert_eq!(result, Value::Null);
        
        let args = vec![Value::String("first".to_string()), Value::String("second".to_string())];
        let result = coalesce.invoke(&args, &context).unwrap();
        assert_eq!(result, Value::String("first".to_string()));
    }

    #[test]
    fn direct_function_call_mixed_types() {
        let coalesce = Coalesce {};
        let context = Context::new();
        
        let args = vec![Value::Null, serde_json::json!(42), Value::String("fallback".to_string())];
        let result = coalesce.invoke(&args, &context).unwrap();
        assert_eq!(result, serde_json::json!(42));
        
        let args = vec![Value::Null, Value::Bool(true)];
        let result = coalesce.invoke(&args, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn parser_with_values() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[coalesce('hello', 'world')]", &Context::new()).unwrap();
        assert_eq!(result, "hello");
        
        let result = parser.parse_and_execute("[coalesce(42, 'fallback')]", &Context::new()).unwrap();
        assert_eq!(result, 42);
        
        let result = parser.parse_and_execute("[coalesce(true)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }
}
