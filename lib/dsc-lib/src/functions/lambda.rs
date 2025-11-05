// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;


/// The lambda() function is special - it's not meant to be invoked directly
/// through the normal function dispatcher path. Instead, it's caught in the
/// Function::invoke method and handled specially via invoke_lambda().
/// 
/// This struct exists for metadata purposes and to signal errors if someone
/// tries to invoke lambda() as a regular function (which shouldn't happen).
#[derive(Debug, Default)]
pub struct LambdaFn {}

impl Function for LambdaFn {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "lambda".to_string(),
            description: t!("functions.lambda.description").to_string(),
            category: vec![FunctionCategory::Lambda],
            min_args: 2,
            max_args: 10, // Up to 9 parameters + 1 body
            accepted_arg_ordered_types: vec![],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Object], // Lambda is represented as a special object
        }
    }

    fn invoke(&self, _args: &[Value], _context: &Context) -> Result<Value, DscError> {
        // This should never be called - lambda() is handled specially in Function::invoke
        Err(DscError::Parser(t!("functions.lambda.cannotInvokeDirectly").to_string()))
    }
}
