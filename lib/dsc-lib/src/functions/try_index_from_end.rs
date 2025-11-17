// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct TryIndexFromEnd {}

impl Function for TryIndexFromEnd {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "tryIndexFromEnd".to_string(),
            description: t!("functions.tryIndexFromEnd.description").to_string(),
            category: vec![FunctionCategory::Array],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Array],
                vec![FunctionArgKind::Number],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Array, FunctionArgKind::Boolean, FunctionArgKind::Null, FunctionArgKind::Number, FunctionArgKind::Object, FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.tryIndexFromEnd.invoked"));

        let Some(array) = args[0].as_array() else {
            return Err(DscError::Parser(t!("functions.tryIndexFromEnd.invalidSourceType").to_string()));
        };

        let Some(reverse_index) = args[1].as_i64() else {
            return Err(DscError::Parser(t!("functions.tryIndexFromEnd.invalidIndexType").to_string()));
        };

        if reverse_index < 1 {
            return Ok(Value::Null);
        }

        let Ok(reverse_index_usize) = usize::try_from(reverse_index) else {
            return Ok(Value::Null);
        };

        if reverse_index_usize > array.len() {
            return Ok(Value::Null);
        }

        let actual_index = array.len() - reverse_index_usize;
        
        Ok(array[actual_index].clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn try_index_from_end_valid_index() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd(createArray('a', 'b', 'c'), 1)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!("c"));
    }

    #[test]
    fn try_index_from_end_middle_element() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd(createArray('a', 'b', 'c'), 2)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!("b"));
    }

    #[test]
    fn try_index_from_end_first_element() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd(createArray('a', 'b', 'c'), 3)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!("a"));
    }

    #[test]
    fn try_index_from_end_out_of_range() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd(createArray('a', 'b', 'c'), 4)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!(null));
    }

    #[test]
    fn try_index_from_end_zero_index() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd(createArray('a', 'b', 'c'), 0)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!(null));
    }

    #[test]
    fn try_index_from_end_negative_index() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd(createArray('a', 'b', 'c'), -1)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!(null));
    }

    #[test]
    fn try_index_from_end_single_element() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd(createArray('only'), 1)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!("only"));
    }

    #[test]
    fn try_index_from_end_numbers() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd(createArray(10, 20, 30, 40), 2)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!(30));
    }

    #[test]
    fn try_index_from_end_objects() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd(createArray(createObject('key', 'value1'), createObject('key', 'value2')), 1)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!({"key": "value2"}));
    }

    #[test]
    fn try_index_from_end_nested_arrays() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd(createArray(createArray(1, 2), createArray(3, 4)), 1)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!([3, 4]));
    }

    #[test]
    fn try_index_from_end_empty_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd(createArray(), 1)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!(null));
    }

    #[test]
    fn try_index_from_end_invalid_source_type() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd('not an array', 1)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn try_index_from_end_invalid_index_type() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd(createArray('a', 'b'), 'string')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn try_index_from_end_large_index() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryIndexFromEnd(createArray('a', 'b'), 1000)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!(null));
    }
}
