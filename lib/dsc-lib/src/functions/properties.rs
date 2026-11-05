// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct Properties {}

/// Implements the `properties` function.
/// Accepts a variable number of arguments, each of which is a string.
/// Returns a string that is the concatenation of the arguments, separated by the platform's path separator.
impl Function for Properties {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "properties".to_string(),
            description: t!("functions.properties.description").to_string(),
            syntax: t!("functions.properties.syntax").to_string(),
            constraints: None,
            category: vec![FunctionCategory::Deployment],
            min_args: 0,
            max_args: 0,
            accepted_arg_ordered_types: vec![],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Object],
        }
    }

    fn invoke(&self, _args: &[Value], context: &Context) -> Result<Value, DscError> {
        Ok(Value::Object(context.properties.clone().unwrap_or_default()))
    }
}
