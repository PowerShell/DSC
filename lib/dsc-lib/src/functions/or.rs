// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Or {}

impl Function for Or {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "or".to_string(),
            description: t!("functions.or.description").to_string(),
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
        debug!("{}", t!("functions.or.invoked"));
        for arg in args {
            if let Some(value) = arg.as_bool() {
                if value {
                    return Ok(Value::Bool(true));
                }
            } else {
                return Err(DscError::Parser(t!("functions.invalidArguments").to_string()));
            }
        }
        Ok(Value::Bool(false))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn two_values() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[or(true, false)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn multiple_values() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[or(true, false, true)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn all_false() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[or(false, false)]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }
}
