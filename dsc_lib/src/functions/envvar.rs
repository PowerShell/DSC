// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::AcceptedArgKind;
use super::Function;
use serde_json::Value;
use std::env;

#[derive(Debug, Default)]
pub struct Envvar {}

impl Function for Envvar {
    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String]
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> usize {
        1
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let val = env::var(args[0].as_str().unwrap_or_default()).unwrap_or_default();
        Ok(Value::String(val))
    }
}
