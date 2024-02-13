// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::parser::functions::{FunctionArg, FunctionResult};
use super::{Function, AcceptedArgKind};
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

    fn invoke(&self, args: &[FunctionArg], _context: &Context) -> Result<FunctionResult, DscError> {
        let FunctionArg::String(arg) = args.first().unwrap() else {
            return Err(DscError::Parser("Invalid argument type".to_string()));
        };

        let val = env::var(arg).unwrap_or_default();
        Ok(FunctionResult::String(val))
    }
}
