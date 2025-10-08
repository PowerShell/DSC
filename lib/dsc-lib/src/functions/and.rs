// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct And {}

impl Function for And {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "and".to_string(),
            description: t!("functions.and.description").to_string(),
            category: vec![FunctionCategory::Logical],
            min_args: 2,
            max_args: usize::MAX,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Boolean],
                vec![FunctionArgKind::Boolean],
            ],
            remaining_arg_accepted_types: Some(vec![FunctionArgKind::Boolean]),
            return_types: vec![FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.and.invoked"));
        for arg in args {
            if let Some(value) = arg.as_bool() {
                if !value {
                    return Ok(Value::Bool(false));
                }
            } else {
                return Err(DscError::Parser(t!("functions.invalidArguments").to_string()));
            }
        }
        Ok(Value::Bool(true))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn two_values() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[and(true, false)]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn multiple_values() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[and(true, false, true)]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn all_false() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[and(false, false)]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn all_true() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[and(true, true)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }
}
