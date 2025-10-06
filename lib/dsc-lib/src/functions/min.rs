// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Min {}

impl Function for Min {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "min".to_string(),
            description: t!("functions.min.description").to_string(),
            category: vec![FunctionCategory::Numeric],
            min_args: 1,
            max_args: usize::MAX,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::Number, FunctionArgKind::Array]],
            remaining_arg_accepted_types: Some(vec![FunctionArgKind::Number, FunctionArgKind::Array]),
            return_types: vec![FunctionArgKind::Number],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.min.invoked"));
        if args.len() == 1 {
            if let Some(array) = args[0].as_array() {
                find_min(array)
            }
            else {
                Err(DscError::Parser(t!("functions.min.emptyArray").to_string()))
            }
        }
        else {
            find_min(args)
        }
    }
}

fn find_min(args: &[Value]) -> Result<Value, DscError> {
    let array = args.iter().map(|v| v.as_i64().ok_or(DscError::Parser(t!("functions.min.integersOnly").to_string()))).collect::<Result<Vec<i64>, DscError>>()?;
    let value = array.iter().min().ok_or(DscError::Parser(t!("functions.min.noMin").to_string()))?;
    Ok(Value::Number((*value).into()))
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
        let result = parser.parse_and_execute("[min(createArray(0, 3, 2, 5, 4))]", &Context::new()).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn array_single_value() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[min(createArray(0))]", &Context::new()).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn arrays() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[min(createArray(0, 3), createArray(2, 5))]", &Context::new());
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
        let result = parser.parse_and_execute("[min(8, min(2, -9), 3)]", &Context::new()).unwrap();
        assert_eq!(result, -9);
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
