// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use crate::functions::lambda_helpers::{get_lambda, apply_lambda_to_array};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Filter {}

impl Function for Filter {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "filter".to_string(),
            description: t!("functions.filter.description").to_string(),
            category: vec![FunctionCategory::Array, FunctionCategory::Lambda],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Array],
                vec![FunctionArgKind::Lambda],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Array],
        }
    }

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.filter.invoked"));

        let array = args[0].as_array().unwrap();
        let lambda_id = args[1].as_str().unwrap();

        let lambdas = get_lambda(context, lambda_id, "filter")?;
        let lambda = lambdas.get(lambda_id).unwrap();

        let result_array = apply_lambda_to_array(array, lambda, context, |result, element| {
            let Some(include) = result.as_bool() else {
                return Err(DscError::Parser(t!("functions.filter.lambdaMustReturnBool").to_string()));
            };
            if include {
                Ok(Some(element.clone()))
            } else {
                Ok(None)
            }
        })?;

        Ok(Value::Array(result_array))
    }
}
