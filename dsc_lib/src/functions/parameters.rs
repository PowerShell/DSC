// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{Function, FunctionArg, FunctionResult, AcceptedArgKind};
use tracing::debug;

#[derive(Debug, Default)]
pub struct Parameters {}

impl Function for Parameters {
    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> usize {
        1
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String]
    }

    fn invoke(&self, args: &[FunctionArg], context: &Context) -> Result<FunctionResult, DscError> {
        let FunctionArg::String(key) = &args[0] else {
            return Err(DscError::Parser("Invalid argument type".to_string()));
        };
        debug!("parameters key: {key}");
        if context.parameters.contains_key(key) {
            Ok(FunctionResult::Object(context.parameters[key].clone()))
        }
        else {
            Err(DscError::Parser(format!("Parameter '{key}' not found in context")))
        }
    }
}
