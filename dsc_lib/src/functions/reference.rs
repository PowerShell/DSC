// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Reference {}

impl Function for Reference {
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
        debug!("reference function");
        if let Some(key) = args[0].as_str() {
            if context.outputs.contains_key(key) {
                Ok(context.outputs[key].clone())
            } else {
                Err(DscError::Parser(format!("Invalid resourceId or resource has not executed yet: {key}")))
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
    fn valid_resourceid() {
        let mut parser = Statement::new().unwrap();
        let mut context = Context::new();
        context.outputs.insert("foo:bar".to_string(), "baz".into());
        let result = parser.parse_and_execute("[reference('foo:bar')]", &context).unwrap();
        assert_eq!(result, "baz");
    }

    #[test]
    fn invalid_resourceid() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[reference('foo:bar')]", &Context::new());
        assert!(result.is_err());
    }
}
