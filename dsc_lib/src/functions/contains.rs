// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Contains {}

impl Function for Contains {
    fn description(&self) -> String {
        t!("functions.contains.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Array
    }

    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        usize::MAX
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::Array, AcceptedArgKind::Object, AcceptedArgKind::String, AcceptedArgKind::Number]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.contains.invoked"));
        let mut found = false;

        let (string_to_find, number_to_find) = if let Some(string) = args[1].as_str() {
            (string.to_string(), 0)
        } else if let Some(number) = args[1].as_i64() {
            (number.to_string(), number)
        } else {
            return Err(DscError::Parser(t!("functions.contains.invalidItemToFind").to_string()));
        };

        // for array, we check if the string or number exists
        if let Some(array) = args[0].as_array() {
            for item in array {
                if let Some(item_str) = item.as_str() {
                    if item_str == string_to_find {
                        found = true;
                        break;
                    }
                } else if let Some(item_num) = item.as_i64() {
                    if item_num == number_to_find {
                        found = true;
                        break;
                    }
                }
            }
            return Ok(Value::Bool(found));
        }

        // for object, we check if the key exists
        if let Some(object) = args[0].as_object() {
            // see if key exists
            for key in object.keys() {
                if key == &string_to_find {
                    found = true;
                    break;
                }
            }
            return Ok(Value::Bool(found));
        }

        // for string, we check if the string contains the substring or number
        if let Some(str) = args[0].as_str() {
            if str.contains(&string_to_find) {
                found = true;
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

