// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::context::{Context, ProcessMode};
use crate::functions::{Function, FunctionArgKind, FunctionCategory, FunctionMetadata};
use crate::DscError;
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Reference {}

impl Function for Reference {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "reference".to_string(),
            description: t!("functions.reference.description").to_string(),
            category: vec![FunctionCategory::Resource],
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
        debug!("{}", t!("functions.reference.invoked"));

        if context.process_mode == ProcessMode::Copy {
            return Err(DscError::Parser(
                t!("functions.reference.cannotUseInCopyMode").to_string(),
            ));
        }
        if context.process_mode == ProcessMode::UserFunction {
            return Err(DscError::Parser(
                t!("functions.reference.unavailableInUserFunction").to_string(),
            ));
        }

        if let Some(key) = args[0].as_str() {
            if context.references.contains_key(key) {
                Ok(context.references[key].clone())
            } else {
                Err(DscError::Parser(
                    t!("functions.reference.keyNotFound", key = key).to_string(),
                ))
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
