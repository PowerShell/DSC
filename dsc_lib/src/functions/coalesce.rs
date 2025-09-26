// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Coalesce {}

impl Function for Coalesce {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "coalesce".to_string(),
            description: t!("functions.coalesce.description").to_string(),
            category: vec![FunctionCategory::Logical],
            min_args: 1,
            max_args: usize::MAX,
            accepted_arg_ordered_types: vec![vec![
                FunctionArgKind::Array,
                FunctionArgKind::Boolean,
                FunctionArgKind::Null,
                FunctionArgKind::Number,
                FunctionArgKind::Object,
                FunctionArgKind::String,
            ]],
            remaining_arg_accepted_types: Some(vec![
                FunctionArgKind::Array,
                FunctionArgKind::Boolean,
                FunctionArgKind::Null,
                FunctionArgKind::Number,
                FunctionArgKind::Object,
                FunctionArgKind::String,
            ]),
            return_types: vec![
                FunctionArgKind::Array,
                FunctionArgKind::Boolean,
                FunctionArgKind::Null,
                FunctionArgKind::Number,
                FunctionArgKind::Object,
                FunctionArgKind::String,
            ],
        }
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
    fn direct_function_call_with_arrays() {
        let coalesce = Coalesce {};
        let context = Context::new();

        let first_array = serde_json::json!(["a", "b", "c"]);
        let second_array = serde_json::json!(["x", "y", "z"]);

        let args = vec![Value::Null, first_array.clone(), second_array];
        let result = coalesce.invoke(&args, &context).unwrap();
        assert_eq!(result, first_array);

        let args = vec![Value::Null, Value::Null, serde_json::json!([1, 2, 3])];
        let result = coalesce.invoke(&args, &context).unwrap();
        assert_eq!(result, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn direct_function_call_with_objects() {
        let coalesce = Coalesce {};
        let context = Context::new();

        let first_obj = serde_json::json!({"name": "test", "value": 42});
        let second_obj = serde_json::json!({"name": "fallback", "value": 0});

        let args = vec![Value::Null, first_obj.clone(), second_obj];
        let result = coalesce.invoke(&args, &context).unwrap();
        assert_eq!(result, first_obj);

        let args = vec![Value::Null, Value::Null, serde_json::json!({"key": "value"})];
        let result = coalesce.invoke(&args, &context).unwrap();
        assert_eq!(result, serde_json::json!({"key": "value"}));
    }

    #[test]
    fn direct_function_call_with_empty_collections() {
        let coalesce = Coalesce {};
        let context = Context::new();

        let empty_array = serde_json::json!([]);
        let args = vec![Value::Null, empty_array.clone(), Value::String("fallback".to_string())];
        let result = coalesce.invoke(&args, &context).unwrap();
        assert_eq!(result, empty_array);

        let empty_obj = serde_json::json!({});
        let args = vec![Value::Null, empty_obj.clone(), Value::String("fallback".to_string())];
        let result = coalesce.invoke(&args, &context).unwrap();
        assert_eq!(result, empty_obj);
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
