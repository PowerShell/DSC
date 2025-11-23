// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::context::Context;
use crate::functions::{Function, FunctionArgKind, FunctionCategory, FunctionMetadata};
use crate::DscError;
use rust_i18n::t;
use serde_json::{Map, Value};

#[derive(Debug, Default)]
pub struct ShallowMerge {}

impl Function for ShallowMerge {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "shallowMerge".to_string(),
            description: t!("functions.shallowMerge.description").to_string(),
            category: vec![FunctionCategory::Object],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::Array]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Object],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let array = args[0].as_array().unwrap();

        let mut result = Map::new();

        for item in array {
            let obj = item.as_object().ok_or_else(|| {
                DscError::Parser(format!(
                    "shallowMerge requires all array elements to be objects, but found: {}",
                    item
                ))
            })?;
            
            for (key, value) in obj {
                // Shallow merge: replace the entire value, even if it's a nested object
                result.insert(key.clone(), value.clone());
            }
        }

        Ok(Value::Object(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::json;

    #[test]
    fn shallow_merge_basic() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(
            "[shallowMerge(createArray(createObject('one', 'a'), createObject('two', 'b'), createObject('two', 'c')))]",
            &Context::new()
        ).unwrap();

        assert_eq!(result, json!({"one": "a", "two": "c"}));
    }

    #[test]
    fn shallow_merge_with_nested_objects() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(
            "[shallowMerge(createArray(createObject('one', 'a', 'nested', createObject('a', 1, 'nested', createObject('c', 3))), createObject('two', 'b', 'nested', createObject('b', 2))))]",
            &Context::new()
        ).unwrap();

        // The nested object should be completely replaced, not merged
        assert_eq!(result, json!({"one": "a", "nested": {"b": 2}, "two": "b"}));
    }

    #[test]
    fn shallow_merge_empty_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[shallowMerge(createArray())]", &Context::new())
            .unwrap();

        assert_eq!(result, json!({}));
    }

    #[test]
    fn shallow_merge_single_object() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute(
                "[shallowMerge(createArray(createObject('name', 'John', 'age', 30)))]",
                &Context::new(),
            )
            .unwrap();

        assert_eq!(result, json!({"name": "John", "age": 30}));
    }

    #[test]
    fn shallow_merge_overwrite_primitives() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(
            "[shallowMerge(createArray(createObject('a', 1, 'b', 2), createObject('b', 3, 'c', 4)))]",
            &Context::new()
        ).unwrap();

        assert_eq!(result, json!({"a": 1, "b": 3, "c": 4}));
    }

    #[test]
    fn shallow_merge_multiple_objects() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(
            "[shallowMerge(createArray(createObject('a', 1), createObject('b', 2), createObject('c', 3), createObject('d', 4)))]",
            &Context::new()
        ).unwrap();

        assert_eq!(result, json!({"a": 1, "b": 2, "c": 3, "d": 4}));
    }

    #[test]
    fn shallow_merge_replaces_nested_completely() {
        let mut parser = Statement::new().unwrap();
        // First object has nested.x and nested.y, second has nested.z
        // Result should only have nested.z (complete replacement)
        let result = parser.parse_and_execute(
            "[shallowMerge(createArray(createObject('nested', createObject('x', 1, 'y', 2)), createObject('nested', createObject('z', 3))))]",
            &Context::new()
        ).unwrap();

        assert_eq!(result, json!({"nested": {"z": 3}}));
    }

    #[test]
    fn shallow_merge_mixed_types() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(
            "[shallowMerge(createArray(createObject('str', 'text', 'num', 42), createObject('bool', true(), 'arr', createArray(1, 2, 3))))]",
            &Context::new()
        ).unwrap();

        assert_eq!(
            result,
            json!({"str": "text", "num": 42, "bool": true, "arr": [1, 2, 3]})
        );
    }

    #[test]
    fn shallow_merge_not_array_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[shallowMerge('not an array')]", &Context::new());

        assert!(result.is_err());
    }

    #[test]
    fn shallow_merge_object_error() {
        let mut parser = Statement::new().unwrap();
        let result =
            parser.parse_and_execute("[shallowMerge(createObject('a', 1))]", &Context::new());

        assert!(result.is_err());
    }

    #[test]
    fn shallow_merge_array_with_non_object_elements_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(
            "[shallowMerge(createArray('string', 'another'))]",
            &Context::new()
        );
        assert!(result.is_err());

        let result = parser.parse_and_execute(
            "[shallowMerge(createArray(1, 2, 3))]",
            &Context::new()
        );
        assert!(result.is_err());

        let result = parser.parse_and_execute(
            "[shallowMerge(createArray(createObject('a', 1), 'string', createObject('b', 2)))]",
            &Context::new()
        );
        assert!(result.is_err());
    }
}
