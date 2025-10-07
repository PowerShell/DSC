// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::context::Context;
use crate::functions::{Function, FunctionArgKind, FunctionCategory, FunctionMetadata};
use crate::DscError;
use rust_i18n::t;
use serde_json::{Map, Value};
use tracing::debug;

#[derive(Debug, Default)]
pub struct Intersection {}

impl Function for Intersection {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "intersection".to_string(),
            description: t!("functions.intersection.description").to_string(),
            category: vec![FunctionCategory::Array, FunctionCategory::Object],
            min_args: 2,
            max_args: usize::MAX,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Array, FunctionArgKind::Object],
                vec![FunctionArgKind::Array, FunctionArgKind::Object],
            ],
            remaining_arg_accepted_types: Some(vec![FunctionArgKind::Array, FunctionArgKind::Object]),
            return_types: vec![FunctionArgKind::Array, FunctionArgKind::Object],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.intersection.invoked"));

        if let Some(first_array) = args[0].as_array() {
            let mut result = Vec::new();

            for item in first_array {
                let mut found_in_all = true;

                for arg in &args[1..] {
                    if let Some(array) = arg.as_array() {
                        if !array.contains(item) {
                            found_in_all = false;
                            break;
                        }
                    } else {
                        return Err(DscError::Parser(
                            t!("functions.intersection.invalidArgType").to_string(),
                        ));
                    }
                }

                if found_in_all && !result.contains(item) {
                    result.push(item.clone());
                }
            }

            return Ok(Value::Array(result));
        }

        if let Some(first_object) = args[0].as_object() {
            let mut result = Map::new();

            for (key, value) in first_object {
                let mut found_in_all = true;

                for arg in &args[1..] {
                    if let Some(object) = arg.as_object() {
                        if let Some(other_value) = object.get(key) {
                            if other_value != value {
                                found_in_all = false;
                                break;
                            }
                        } else {
                            found_in_all = false;
                            break;
                        }
                    } else {
                        return Err(DscError::Parser(
                            t!("functions.intersection.invalidArgType").to_string(),
                        ));
                    }
                }

                if found_in_all {
                    result.insert(key.clone(), value.clone());
                }
            }

            return Ok(Value::Object(result));
        }

        Err(DscError::Parser(
            t!("functions.intersection.invalidArgType").to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn array_intersection() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute(
                "[intersection(createArray(1, 2, 3), createArray(2, 3, 4))]",
                &Context::new(),
            )
            .unwrap();
        assert_eq!(result, serde_json::json!([2, 3]));
    }

    #[test]
    fn array_intersection_three_arrays() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute(
                "[intersection(createArray(1, 2, 3, 4), createArray(2, 3, 4, 5), createArray(3, 4, 5, 6))]",
                &Context::new(),
            )
            .unwrap();
        assert_eq!(result, serde_json::json!([3, 4]));
    }

    #[test]
    fn array_intersection_no_common_elements() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[intersection(createArray(1, 2), createArray(3, 4))]", &Context::new())
            .unwrap();
        assert_eq!(result, serde_json::json!([]));
    }

    #[test]
    fn array_intersection_with_duplicates() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute(
                "[intersection(createArray(1, 2, 2, 3), createArray(2, 2, 3, 4))]",
                &Context::new(),
            )
            .unwrap();
        assert_eq!(result, serde_json::json!([2, 3]));
    }

    #[test]
    fn object_intersection() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute(
                "[intersection(createObject('a', 1, 'b', 2), createObject('b', 2, 'c', 3))]",
                &Context::new(),
            )
            .unwrap();
        assert_eq!(result, serde_json::json!({"b": 2}));
    }

    #[test]
    fn object_intersection_different_values() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute(
                "[intersection(createObject('a', 1, 'b', 2), createObject('a', 2, 'b', 2))]",
                &Context::new(),
            )
            .unwrap();
        assert_eq!(result, serde_json::json!({"b": 2}));
    }

    #[test]
    fn object_intersection_no_common_keys() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute(
                "[intersection(createObject('a', 1), createObject('b', 2))]",
                &Context::new(),
            )
            .unwrap();
        assert_eq!(result, serde_json::json!({}));
    }

    #[test]
    fn mixed_types_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(
            "[intersection(createArray(1, 2), createObject('a', 1))]",
            &Context::new(),
        );
        assert!(result.is_err());
    }
}
