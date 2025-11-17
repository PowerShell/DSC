// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::{Context, ProcessMode};
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Variables {}

impl Function for Variables {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "variables".to_string(),
            description: t!("functions.variables.description").to_string(),
            category: vec![FunctionCategory::Deployment],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![
                FunctionArgKind::Array,
                FunctionArgKind::Boolean,
                FunctionArgKind::Number,
                FunctionArgKind::Object,
                FunctionArgKind::String,
            ],
        }
    }

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.variables.invoked"));
        if context.process_mode == ProcessMode::UserFunction {
            return Err(DscError::Parser(t!("functions.variables.unavailableInUserFunction").to_string()));
        }

        if let Some(key) = args[0].as_str() {
            if context.variables.contains_key(key) {
                Ok(context.variables[key].clone())
            } else {
                Err(DscError::Parser(t!("functions.variables.keyNotFound", key = key).to_string()))
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
