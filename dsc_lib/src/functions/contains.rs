// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Contains {}

impl Function for Contains {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "contains".to_string(),
            description: t!("functions.contains.description").to_string(),
            category: FunctionCategory::Array,
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Array, FunctionArgKind::Object, FunctionArgKind::String],
                vec![FunctionArgKind::String, FunctionArgKind::Number],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.contains.invoked"));
        let mut found = false;

        let (string_to_find, number_to_find) = if let Some(string) = args[1].as_str() {
            (Some(string.to_string()), None)
        } else if let Some(number) = args[1].as_i64() {
            (None, Some(number))
        } else {
            return Err(DscError::Parser(t!("functions.contains.invalidItemToFind").to_string()));
        };

        // for array, we check if the string or number exists
        if let Some(array) = args[0].as_array() {
            for item in array {
                if let Some(item_str) = item.as_str() {
                    if let Some(string) = &string_to_find {
                        if item_str == string {
                            found = true;
                            break;
                        }
                    }
                } else if let Some(item_num) = item.as_i64() {
                    if let Some(number) = number_to_find {
                        if item_num == number {
                            found = true;
                            break;
                        }
                    }
                }
            }
            return Ok(Value::Bool(found));
        }

        // for object, we check if the key exists
        if let Some(object) = args[0].as_object() {
            // see if key exists
            for key in object.keys() {
                if let Some(string) = &string_to_find {
                    if key == string {
                        found = true;
                        break;
                    }
                } else if let Some(number) = number_to_find {
                    if key == &number.to_string() {
                        found = true;
                        break;
                    }
                }
            }
            return Ok(Value::Bool(found));
        }

        // for string, we check if the string contains the substring or number
        if let Some(str) = args[0].as_str() {
            if let Some(string) = &string_to_find {
                found = str.contains(string);
            } else if let Some(number) = number_to_find {
                found = str.contains(&number.to_string());
            }
            return Ok(Value::Bool(found));
        }

        Err(DscError::Parser(t!("functions.contains.invalidArgType").to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn string_contains_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[contains('hello', 'lo')]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn string_does_not_contain_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[contains('hello', 'world')]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn string_contains_number() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[contains('hello123', 123)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }
}

