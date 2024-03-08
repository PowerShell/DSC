// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Min {}

impl Function for Min {
    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> usize {
        usize::MAX
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::Number, AcceptedArgKind::Array]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("min function");
        if args.len() == 1 {
            if let Some(array) = args[0].as_array() {
                if array.len() > 1 {
                    find_min(array)
                }
                else {
                    Err(DscError::Parser("Array must contain more than 1 integer".to_string()))
                }
            }
            else {
                Err(DscError::Parser("List must contain more than 1 integer".to_string()))
            }
        }
        else {
            find_min(args)
        }
    }
}

fn find_min(args: &[Value]) -> Result<Value, DscError> {
    let mut min_value = i64::MAX;
    for value in args {
        if let Some(int_value) = value.as_i64() {
            if int_value < min_value {
                min_value = int_value;
            }
        }
        else {
            return Err(DscError::Parser("Input must only contain integers".to_string()));
        }
    }
    Ok(Value::Number(min_value.into()))
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn list() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[min(3,2,5,4)]", &Context::new()).unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn list_with_spaces() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[min(3, 2, 5, 4)]", &Context::new()).unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[min(createArray(0, 3, 2, 5, 4)]", &Context::new()).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn array_single_value() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[min(createArray(0)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn arrays() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[min(createArray('0','3'), createArray('2','5'))]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn string_and_numbers() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[min('a', 1)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn nested() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[min(8, min(2, 5), 3)]", &Context::new()).unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn invalid_one_parameter() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[min(1)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn int_and_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[min(1, createArray(0,2))]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn array_and_int() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[min(createArray(0,2), 1)]", &Context::new());
        assert!(result.is_err());
    }
}
