// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct ObjectKeys {}

impl Function for ObjectKeys {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "objectKeys".to_string(),
            description: t!("functions.objectKeys.description").to_string(),
            category: vec![FunctionCategory::Object],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Object],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Array],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let Some(obj) = args[0].as_object() else {
            return Err(DscError::Parser(t!("functions.objectKeys.notObject").to_string()));
        };

        // Extract all keys from the object and return as an array of strings
        let keys: Vec<Value> = obj
            .keys()
            .map(|key| Value::String(key.clone()))
            .collect();

        Ok(Value::Array(keys))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::{json, Value};

    #[test]
    fn object_keys_basic() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[objectKeys(createObject('a', 1, 'b', 2, 'c', 3))]", &Context::new()).unwrap();
        
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        
        for key in arr {
            assert!(key.is_string());
        }
        
        let keys: Vec<&str> = arr.iter().filter_map(Value::as_str).collect();
        assert!(keys.contains(&"a"));
        assert!(keys.contains(&"b"));
        assert!(keys.contains(&"c"));
    }

    #[test]
    fn object_keys_empty_object() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[objectKeys(createObject())]", &Context::new()).unwrap();
        assert_eq!(result, json!([]));
    }

    #[test]
    fn object_keys_single_key() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[objectKeys(createObject('name', 'John'))]", &Context::new()).unwrap();
        
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].as_str(), Some("name"));
    }

    #[test]
    fn object_keys_nested_values() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[objectKeys(createObject('person', createObject('name', 'John', 'age', 30)))]", &Context::new()).unwrap();
        
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].as_str(), Some("person"));
    }

    #[test]
    fn object_keys_mixed_value_types() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[objectKeys(createObject('str', 'text', 'num', 42, 'bool', true(), 'arr', createArray(1,2,3)))]", &Context::new()).unwrap();
        
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 4);
        
        let keys: Vec<&str> = arr.iter().filter_map(Value::as_str).collect();
        assert!(keys.contains(&"str"));
        assert!(keys.contains(&"num"));
        assert!(keys.contains(&"bool"));
        assert!(keys.contains(&"arr"));
    }

    #[test]
    fn object_keys_can_be_used_with_length() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[length(objectKeys(createObject('a', 1, 'b', 2, 'c', 3)))]", &Context::new()).unwrap();
        assert_eq!(result, json!(3));
    }

    #[test]
    fn object_keys_not_object_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[objectKeys('not an object')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn object_keys_array_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[objectKeys(createArray('a', 'b', 'c'))]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn object_keys_number_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[objectKeys(42)]", &Context::new());
        assert!(result.is_err());
    }
}
