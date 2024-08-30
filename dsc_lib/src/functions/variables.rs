// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Variables {}

impl Function for Variables {
    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> usize {
        1
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String]
    }

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        debug!("variables function");
        if let Some(key) = args[0].as_str() {
            if context.variables.contains_key(key) {
                Ok(context.variables[key].clone())
            } else {
                Err(DscError::Parser(format!("Variable '{key}' does not exist or has not been initialized yet")))
            }
        } else {
            Err(DscError::Parser("Invalid argument".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn valid_variable() {
        let mut parser = Statement::new().unwrap();
        let mut context = Context::new();
        context.variables.insert("hello".to_string(), "world".into());
        let result = parser.parse_and_execute("[variables('hello')]", &context).unwrap();
        assert_eq!(result, "world");
    }

    #[test]
    fn invalid_resourceid() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[variables('foo')]", &Context::new());
        assert!(result.is_err());
    }
}
