// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct IndexOf {}

impl Function for IndexOf {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "indexOf".to_string(),
            description: t!("functions.indexOf.description").to_string(),
            category: FunctionCategory::Array,
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Array],
                vec![FunctionArgKind::String, FunctionArgKind::Number, FunctionArgKind::Array, FunctionArgKind::Object],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Number],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.indexOf.invoked"));
        
        let array = args[0].as_array().ok_or_else(|| {
            DscError::Parser(t!("functions.indexOf.invalidArrayArg").to_string())
        })?;

        let item_to_find = &args[1];

        for (index, item) in array.iter().enumerate() {
            let matches = match (item_to_find, item) {
                // String comparison (case-sensitive)
                (Value::String(find_str), Value::String(item_str)) => find_str == item_str,
                (Value::Number(find_num), Value::Number(item_num)) => find_num == item_num,
                (Value::Array(find_arr), Value::Array(item_arr)) => find_arr == item_arr,
                (Value::Object(find_obj), Value::Object(item_obj)) => find_obj == item_obj,
                _ => false,
            };

            if matches {
                return Ok(Value::Number((index as i64).into()));
            }
        }

        // Not found is -1
        Ok(Value::Number((-1i64).into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn find_string_in_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[indexOf(createArray('apple', 'banana', 'cherry'), 'banana')]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "1");
    }

    #[test]
    fn find_number_in_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[indexOf(createArray(10, 20, 30), 20)]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "1");
    }

    #[test]
    fn find_first_occurrence() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[indexOf(createArray('a', 'b', 'a', 'c'), 'a')]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "0");
    }

    #[test]
    fn item_not_found() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[indexOf(createArray('apple', 'banana'), 'orange')]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "-1");
    }

    #[test]
    fn case_sensitive_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[indexOf(createArray('Apple', 'Banana'), 'apple')]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "-1");
    }

    #[test]
    fn find_array_in_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[indexOf(array(createArray('a', 'b'), createArray('c', 'd')), createArray('c', 'd'))]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "1");
    }

    #[test]
    fn find_object_in_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[indexOf(array(createObject('name', 'John'), createObject('name', 'Jane')), createObject('name', 'Jane'))]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "1");
    }

    #[test]
    fn empty_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[indexOf(createArray(), 'test')]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), "-1");
    }

    #[test]
    fn invalid_array_arg() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[indexOf('not_an_array', 'test')]", &Context::new());
        assert!(result.is_err());
    }
}
