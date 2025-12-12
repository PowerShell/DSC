// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata, FunctionDispatcher};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Map {}

impl Function for Map {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "map".to_string(),
            description: t!("functions.map.description").to_string(),
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
        debug!("{}", t!("functions.map.invoked"));
        
        if args.len() != 2 {
            return Err(DscError::Parser(t!("functions.invalidArgCount", name = "map", count = 2).to_string()));
        }

        let Some(array) = args[0].as_array() else {
            return Err(DscError::Parser(t!("functions.map.firstArgMustBeArray").to_string()));
        };

        let Some(lambda_id) = args[1].as_str() else {
            return Err(DscError::Parser(t!("functions.map.secondArgMustBeLambda").to_string()));
        };

        // Retrieve the lambda from context
        let lambdas = context.lambdas.borrow();
        let Some(lambda) = lambdas.get(lambda_id) else {
            return Err(DscError::Parser(t!("functions.map.lambdaNotFound", id = lambda_id).to_string()));
        };

        // Validate parameter count (1 or 2 parameters)
        if lambda.parameters.is_empty() || lambda.parameters.len() > 2 {
            return Err(DscError::Parser(t!("functions.map.lambdaMustHave1Or2Params").to_string()));
        }

        // Create function dispatcher for evaluating lambda body
        let dispatcher = FunctionDispatcher::new();
        let mut result_array = Vec::new();

        // Iterate through array and evaluate lambda for each element
        for (index, element) in array.iter().enumerate() {
            // Create a new context with lambda variables bound
            let mut lambda_context = context.clone();
            
            // Bind first parameter to array element
            lambda_context.lambda_variables.insert(
                lambda.parameters[0].clone(),
                element.clone()
            );

            // Bind second parameter to index if provided
            if lambda.parameters.len() == 2 {
                lambda_context.lambda_variables.insert(
                    lambda.parameters[1].clone(),
                    Value::Number(serde_json::Number::from(index))
                );
            }

            // Evaluate lambda body with bound variables
            let result = lambda.body.invoke(&dispatcher, &lambda_context)?;
            result_array.push(result);
        }

        Ok(Value::Array(result_array))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_two_args() {
        let func = Map {};
        let result = func.invoke(&[], &Context::new());
        assert!(result.is_err());
    }
}
