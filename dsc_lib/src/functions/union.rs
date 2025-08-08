// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::{Map, Value};
use tracing::debug;

#[derive(Debug, Default)]
pub struct Union {}

impl Function for Union {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "union".to_string(),
            description: t!("functions.union.description").to_string(),
            category: FunctionCategory::Array,
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
        debug!("{}", t!("functions.union.invoked"));
        if args[0].is_array() {
            let mut result = Vec::new();
            // iterate through array and skip elements that are already in result
            for arg in args {
                if let Some(array) = arg.as_array() {
                    for item in array {
                        if !result.contains(item) {
                            result.push(item.clone());
                        }
                    }
                } else {
                    return Err(DscError::Parser(t!("functions.union.invalidArgType").to_string()));
                }
            }
            return Ok(Value::Array(result));
        }

        if args[0].is_object() {
            let mut result = Map::new();
            // iterate through objects, duplicate keys are overwritten
            for arg in args {
                if let Some(object) = arg.as_object() {
                    for (key, value) in object {
                        result.insert(key.clone(), value.clone());
                    }
                } else {
                    return Err(DscError::Parser(t!("functions.union.invalidArgType").to_string()));
                }
            }
            return Ok(Value::Object(result));
        }

        Err(DscError::Parser(t!("functions.union.invalidArgType").to_string()))
    }
}
