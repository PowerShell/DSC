// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use rust_i18n::t;
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
        debug!("{}", t!("functions.reference.invoked"));
        if let Some(key) = args[0].as_str() {
            if context.references.contains_key(key) {
                Ok(context.references[key].clone())
            } else {
                Err(DscError::Parser(t!("functions.reference.keyNotFound", key = key).to_string()))
            }
        } else {
            Err(DscError::Parser(t!("functions.invalidArguments").to_string()))
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
        context.references.insert("foo:bar".to_string(), "baz".into());
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
