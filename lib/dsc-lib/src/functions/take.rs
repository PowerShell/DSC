// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Take {}

impl Function for Take {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "take".to_string(),
            description: t!("functions.take.description").to_string(),
            category: vec![FunctionCategory::Array, FunctionCategory::String],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Array, FunctionArgKind::String],
                vec![FunctionArgKind::Number],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Array, FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.take.invoked"));

        if let Some(count_i64) = args[1].as_i64() {
            let count: usize = if count_i64 <= 0 {
                0
            } else {
                count_i64.try_into().unwrap_or(usize::MAX)
            };

            if let Some(array) = args[0].as_array() {
                let take_count = count.min(array.len());
                let taken = array.iter().take(take_count).cloned().collect::<Vec<Value>>();
                return Ok(Value::Array(taken));
            }

            if let Some(s) = args[0].as_str() {
                let result: String = s.chars().take(count).collect();
                return Ok(Value::String(result));
            }

            return Err(DscError::Parser(t!("functions.take.invalidOriginalValue").to_string()));
        }

        Err(DscError::Parser(t!("functions.take.invalidNumberToTake").to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::Value;

    #[test]
    fn take_array_basic() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[take(createArray('a','b','c','d'), 2)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Array(vec![Value::String("a".into()), Value::String("b".into())]));
    }

    #[test]
    fn take_string_basic() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[take('hello', 2)]", &Context::new()).unwrap();
        assert_eq!(result, Value::String("he".into()));
    }

    #[test]
    fn take_more_than_length() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[take(createArray('a','b'), 5)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Array(vec![Value::String("a".into()), Value::String("b".into())]));
    }

    #[test]
    fn take_string_more_than_length() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[take('hi', 10)]", &Context::new()).unwrap();
        assert_eq!(result, Value::String("hi".into()));
    }

    #[test]
    fn take_array_zero() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[take(createArray('a','b'), 0)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Array(vec![]));
    }

    #[test]
    fn take_string_zero() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[take('hello', 0)]", &Context::new()).unwrap();
        assert_eq!(result, Value::String("".into()));
    }

    #[test]
    fn take_array_negative_is_empty() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[take(createArray('a','b','c'), -1)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Array(vec![]));
    }

    #[test]
    fn take_string_negative_is_empty() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[take('hello', -2)]", &Context::new()).unwrap();
        assert_eq!(result, Value::String("".into()));
    }

    #[test]
    fn take_empty_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[take(createArray(), 2)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Array(vec![]));
    }

    #[test]
    fn take_empty_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[take('', 1)]", &Context::new()).unwrap();
        assert_eq!(result, Value::String("".into()));
    }

    #[test]
    fn take_array_all_elements() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[take(createArray('x','y','z'), 3)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Array(vec![
            Value::String("x".into()),
            Value::String("y".into()),
            Value::String("z".into()),
        ]));
    }

    #[test]
    fn take_string_all_characters() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[take('test', 4)]", &Context::new()).unwrap();
        assert_eq!(result, Value::String("test".into()));
    }
}
