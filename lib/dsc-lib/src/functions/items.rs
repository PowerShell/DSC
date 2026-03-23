// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::{Map, Value};

#[derive(Debug, Default)]
pub struct Items {}

impl Function for Items {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "items".to_string(),
            description: t!("functions.items.description").to_string(),
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
            return Err(DscError::Parser(t!("functions.items.notObject").to_string()));
        };

        // Convert the object to an array of key-value pairs
        // Each element is an object with "key" and "value" properties
        let items: Vec<Value> = obj
            .iter()
            .map(|(key, value)| {
                let mut item = Map::new();
                item.insert("key".to_string(), Value::String(key.clone()));
                item.insert("value".to_string(), value.clone());
                Value::Object(item)
            })
            .collect();

        Ok(Value::Array(items))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::{json, Value};

    #[test]
    fn items_basic_object() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[items(createObject('a', 1, 'b', 2, 'c', 3))]", &Context::new()).unwrap();
        
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        
        for item in arr {
            assert!(item.is_object());
            let obj = item.as_object().unwrap();
            assert!(obj.contains_key("key"));
            assert!(obj.contains_key("value"));
        }
        
        let has_a = arr.iter().any(|item| {
            let obj = item.as_object().unwrap();
            obj.get("key").and_then(Value::as_str) == Some("a")
                && obj.get("value").and_then(Value::as_i64) == Some(1)
        });
        assert!(has_a);
    }

    #[test]
    fn items_empty_object() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[items(createObject())]", &Context::new()).unwrap();
        assert_eq!(result, json!([]));
    }

    #[test]
    fn items_nested_values() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[items(createObject('person', createObject('name', 'John', 'age', 30)))]", &Context::new()).unwrap();
        
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        
        let item = &arr[0];
        let obj = item.as_object().unwrap();
        assert_eq!(obj.get("key").and_then(Value::as_str), Some("person"));
        
        let person = obj.get("value").unwrap();
        assert!(person.is_object());
        let person_obj = person.as_object().unwrap();
        assert_eq!(person_obj.get("name").and_then(Value::as_str), Some("John"));
        assert_eq!(person_obj.get("age").and_then(Value::as_i64), Some(30));
    }

    #[test]
    fn items_array_values() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[items(createObject('list', createArray('a', 'b', 'c')))]", &Context::new()).unwrap();
        
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        
        let item = &arr[0];
        let obj = item.as_object().unwrap();
        assert_eq!(obj.get("key").and_then(Value::as_str), Some("list"));
        
        let list = obj.get("value").unwrap();
        assert_eq!(list, &json!(["a", "b", "c"]));
    }

    #[test]
    fn items_string_values() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[items(createObject('greeting', 'Hello', 'farewell', 'Goodbye'))]", &Context::new()).unwrap();
        
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        
        let has_greeting = arr.iter().any(|item| {
            let obj = item.as_object().unwrap();
            obj.get("key").and_then(Value::as_str) == Some("greeting")
                && obj.get("value").and_then(Value::as_str) == Some("Hello")
        });
        assert!(has_greeting);
        
        let has_farewell = arr.iter().any(|item| {
            let obj = item.as_object().unwrap();
            obj.get("key").and_then(Value::as_str) == Some("farewell")
                && obj.get("value").and_then(Value::as_str) == Some("Goodbye")
        });
        assert!(has_farewell);
    }

    #[test]
    fn items_mixed_types() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[items(createObject('str', 'text', 'num', 42, 'bool', true()))]", &Context::new()).unwrap();
        
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        
        let has_str = arr.iter().any(|item| {
            let obj = item.as_object().unwrap();
            obj.get("key").and_then(Value::as_str) == Some("str")
                && obj.get("value").and_then(Value::as_str) == Some("text")
        });
        assert!(has_str);
        
        let has_num = arr.iter().any(|item| {
            let obj = item.as_object().unwrap();
            obj.get("key").and_then(Value::as_str) == Some("num")
                && obj.get("value").and_then(Value::as_i64) == Some(42)
        });
        assert!(has_num);
        
        let has_bool = arr.iter().any(|item| {
            let obj = item.as_object().unwrap();
            obj.get("key").and_then(Value::as_str) == Some("bool")
                && obj.get("value").and_then(Value::as_bool) == Some(true)
        });
        assert!(has_bool);
    }

    #[test]
    fn items_not_object_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[items('not an object')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn items_array_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[items(createArray('a', 'b', 'c'))]", &Context::new());
        assert!(result.is_err());
    }
}
