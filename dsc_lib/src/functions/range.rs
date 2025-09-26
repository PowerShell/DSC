// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Range {}

impl Function for Range {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "range".to_string(),
            description: t!("functions.range.description").to_string(),
            category: FunctionCategory::Array,
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Number],
                vec![FunctionArgKind::Number],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Array],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.range.invoked"));

        let start_index = args[0].as_i64()
            .ok_or_else(|| DscError::FunctionArg("range".to_string(), t!("functions.range.startIndexNotInt").to_string()))?;
        
        let count = args[1].as_i64()
            .ok_or_else(|| DscError::FunctionArg("range".to_string(), t!("functions.range.countNotInt").to_string()))?;

        // validation checks
        if count < 0 {
            return Err(DscError::FunctionArg("range".to_string(), t!("functions.range.countNegative").to_string()));
        }

        if count > 10000 {
            return Err(DscError::FunctionArg("range".to_string(), t!("functions.range.countTooLarge").to_string()));
        }

        // should not exceed
        if let Some(sum) = start_index.checked_add(count) {
            if sum > 2147483647 {
                return Err(DscError::FunctionArg("range".to_string(), t!("functions.range.sumTooLarge").to_string()));
            }
        } else {
            return Err(DscError::FunctionArg("range".to_string(), t!("functions.range.sumOverflow").to_string()));
        }

        let mut result = Vec::<Value>::new();
        for i in 0..count {
            let value = start_index + i;
            result.push(Value::Number(value.into()));
        }

        Ok(Value::Array(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::Value;

    #[test]
    fn basic_range() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[range(1, 3)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Array(vec![Value::from(1), Value::from(2), Value::from(3)]));
    }

    #[test]
    fn range_starting_zero() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[range(0, 5)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Array(vec![
            Value::from(0), Value::from(1), Value::from(2), Value::from(3), Value::from(4)
        ]));
    }

    #[test]
    fn range_negative_start() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[range(-2, 4)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Array(vec![
            Value::from(-2), Value::from(-1), Value::from(0), Value::from(1)
        ]));
    }

    #[test]
    fn range_count_zero() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[range(10, 0)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Array(vec![]));
    }

    #[test]
    fn range_count_negative() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[range(1, -1)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn range_count_too_large() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[range(1, 10001)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn range_sum_too_large() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[range(2147483647, 1)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn range_large_valid() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[range(2147473647, 10000)]", &Context::new()).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 10000);
            assert_eq!(arr[0], Value::from(2147473647));
            assert_eq!(arr[9999], Value::from(2147483646));
        } else {
            panic!("Expected array result");
        }
    }
}
