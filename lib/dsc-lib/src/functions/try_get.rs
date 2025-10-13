// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct TryGet {}

impl Function for TryGet {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "tryGet".to_string(),
            description: t!("functions.tryGet.description").to_string(),
            category: vec![FunctionCategory::Array, FunctionCategory::Object],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Array, FunctionArgKind::Object],
                vec![FunctionArgKind::Number, FunctionArgKind::String],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Array, FunctionArgKind::Boolean, FunctionArgKind::Null, FunctionArgKind::Number, FunctionArgKind::Object, FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.tryGet.invoked"));

        if let Some(object) = args[0].as_object() {
            if let Some(key) = args[1].as_str() {
                if let Some(value) = object.get(key) {
                    return Ok(value.clone());
                }
                return Ok(Value::Null);
            }
            return Err(DscError::Parser(t!("functions.tryGet.invalidKeyType").to_string()));
        }

        if let Some(array) = args[0].as_array() {
            if let Some(index) = args[1].as_i64() {
                let Ok(index) = usize::try_from(index) else {
                    // handle negative index
                    return Ok(Value::Null);
                };
                let index = if index >= array.len() {
                    return Ok(Value::Null);
                } else {
                    index
                };
                return Ok(array[index].clone());
            }
            return Err(DscError::Parser(t!("functions.tryGet.invalidIndexType").to_string()));
        }

        Err(DscError::Parser(t!("functions.invalidArgType").to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn try_get_object() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryGet(createObject('key1', 'value1'), 'key1')]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!("value1"));
    }

    #[test]
    fn try_get_object_not_found() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryGet(createObject('key1', 'value1'), 'key2')]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!(null));
    }

    #[test]
    fn try_get_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryGet(createArray('value1', 'value2'), 1)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!("value2"));
    }

    #[test]
    fn try_get_array_negative_index() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryGet(createArray('value1', 'value2'), -1)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!(null));
    }

    #[test]
    fn try_get_array_index_not_found() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryGet(createArray('value1', 'value2'), 2)]", &Context::new()).unwrap();
        assert_eq!(result, serde_json::json!(null));
    }

    #[test]
    fn try_get_object_invalid_key_type() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryGet(createObject('key1', 'value1'), 1)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn try_get_array_invalid_index_type() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryGet(createArray('value1', 'value2'), '1')]", &Context::new());
        assert!(result.is_err());
    }
}
