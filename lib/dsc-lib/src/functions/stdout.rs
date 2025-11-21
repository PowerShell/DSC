// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use super::Function;

#[derive(Debug, Default)]
pub struct Stdout {}

impl Function for Stdout {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "stdout".to_string(),
            description: t!("functions.stdout.description").to_string(),
            category: vec![FunctionCategory::System],
            min_args: 0,
            max_args: 0,
            accepted_arg_ordered_types: vec![],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, _args: &[Value], context: &Context) -> Result<Value, DscError> {
        if let Some(stdout) = &context.stdout {
            let result = stdout.to_string();
            return Ok(Value::String(result));
        }
        Err(DscError::Parser(t!("functions.stdout.noStdoutAvailable").to_string(),
        ))
    }
}
