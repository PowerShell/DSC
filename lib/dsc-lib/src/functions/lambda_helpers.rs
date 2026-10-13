// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Helper functions for lambda-consuming functions like `map()` and `filter()`.
//!
//! This module provides common utilities for retrieving lambdas from context,
//! validating lambda parameters, and iterating over arrays with lambda application.

use crate::DscError;
use crate::configure::context::Context;
use crate::parser::functions::Lambda;
use crate::functions::FunctionDispatcher;
use rust_i18n::t;
use serde_json::Value;
use std::cell::Ref;

/// Retrieves a lambda from the context and validates it has 1 or 2 parameters.
///
/// # Arguments
///
/// * `context` - The context containing the lambda registry
/// * `lambda_id` - The lambda ID string (e.g., `__lambda_<uuid>`)
/// * `func_name` - The name of the calling function (for error messages)
///
/// # Returns
///
/// A reference to the borrowed lambdas HashMap. The caller must use the returned
/// `Ref` to access the lambda to keep the borrow active.
///
/// # Errors
///
/// Returns an error if the lambda is not found or has invalid parameter count.
pub fn get_lambda<'a>(
    context: &'a Context,
    lambda_id: &str,
    func_name: &str,
) -> Result<Ref<'a, std::collections::HashMap<String, Lambda>>, DscError> {
    let lambdas = context.lambdas.borrow();
    
    if !lambdas.contains_key(lambda_id) {
        return Err(DscError::Parser(t!("functions.lambdaNotFound", name = func_name, id = lambda_id).to_string()));
    }
    
    let lambda = lambdas.get(lambda_id).unwrap();
    if lambda.parameters.is_empty() || lambda.parameters.len() > 2 {
        return Err(DscError::Parser(t!("functions.lambdaTooManyParams", name = func_name).to_string()));
    }
    
    Ok(lambdas)
}

/// Applies a lambda to each element of an array, yielding transformed values.
///
/// This is the core iteration logic shared by `map()`, `filter()`, and similar
/// lambda-consuming functions.
///
/// # Arguments
///
/// * `array` - The input array to iterate over
/// * `lambda` - The lambda to apply to each element
/// * `context` - The base context (will be cloned for each iteration)
/// * `mut apply` - A closure that receives the lambda result and can transform/filter it
///
/// # Returns
///
/// A vector of values produced by the `apply` closure.
pub fn apply_lambda_to_array<F>(
    array: &[Value],
    lambda: &Lambda,
    context: &Context,
    mut apply: F,
) -> Result<Vec<Value>, DscError>
where
    F: FnMut(Value, &Value) -> Result<Option<Value>, DscError>,
{
    let dispatcher = FunctionDispatcher::new();
    let mut results = Vec::new();

    for (index, element) in array.iter().enumerate() {
        let mut lambda_context = context.clone();
        
        lambda_context.lambda_variables.insert(
            lambda.parameters[0].clone(),
            element.clone()
        );

        if lambda.parameters.len() == 2 {
            lambda_context.lambda_variables.insert(
                lambda.parameters[1].clone(),
                Value::Number(serde_json::Number::from(index))
            );
        }

        let result = lambda.body.invoke(&dispatcher, &lambda_context)?;
        
        if let Some(value) = apply(result, element)? {
            results.push(value);
        }
    }

    Ok(results)
}
