// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use serde_json::Value;
use tracing::trace;

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

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        if let Some(key) = args[0].as_str() {
            trace!("parameters key: {key}");
            if context.parameters.contains_key(key) {
                Ok(context.parameters[key].clone())
            }
            else {
                Err(DscError::Parser(format!("Parameter '{key}' not found in context")))
            }
        } else {
            Err(DscError::Parser("Invalid argument type".to_string()))
        }
    }
}
