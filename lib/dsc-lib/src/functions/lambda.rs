// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use crate::parser::functions::{FunctionArg, Lambda};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct LambdaFn {}

impl Function for LambdaFn {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "lambda".to_string(),
            description: t!("functions.lambda.description").to_string(),
            category: vec![FunctionCategory::Lambda],
            min_args: 0, // Args come through context.lambda_raw_args
            max_args: 0,
            accepted_arg_ordered_types: vec![],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Lambda],
        }
    }

    fn invoke(&self, _args: &[Value], context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.lambda.invoked"));

        let raw_args = context.lambda_raw_args.borrow();
        let args = raw_args.as_ref()
            .filter(|a| a.len() >= 2)
            .ok_or_else(|| DscError::Parser(t!("functions.lambda.requiresParamAndBody").to_string()))?;

        let (body_arg, param_args) = args.split_last().unwrap(); // safe: len >= 2

        let parameters: Vec<String> = param_args.iter()
            .map(|arg| match arg {
                FunctionArg::Value(Value::String(s)) => Ok(s.clone()),
                _ => Err(DscError::Parser(t!("functions.lambda.paramsMustBeStrings").to_string())),
            })
            .collect::<Result<_, _>>()?;

        // Extract body expression
        let body = match body_arg {
            FunctionArg::Expression(expr) => expr.clone(),
            _ => return Err(DscError::Parser(t!("functions.lambda.bodyMustBeExpression").to_string())),
        };

        // Create Lambda and store in Context with unique ID
        let lambda = Lambda { parameters, body };
        let lambda_id = format!("__lambda_{}", Uuid::new_v4());
        context.lambdas.borrow_mut().insert(lambda_id.clone(), lambda);

        Ok(Value::String(lambda_id))
    }
}
