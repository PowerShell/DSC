// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct LambdaVariables {}

impl Function for LambdaVariables {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "lambdaVariables".to_string(),
            description: t!("functions.lambdaVariables.description").to_string(),
            category: vec![FunctionCategory::Lambda],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![
                FunctionArgKind::String,
                FunctionArgKind::Number,
                FunctionArgKind::Boolean,
                FunctionArgKind::Array,
                FunctionArgKind::Object,
                FunctionArgKind::Null,
            ],
        }
    }

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.lambdaVariables.invoked"));

        let var_name = args[0].as_str().unwrap();

        // Look up the variable in the lambda context
        if let Some(value) = context.lambda_variables.get(var_name) {
            Ok(value.clone())
        } else {
            Err(DscError::Parser(t!("functions.lambdaVariables.notFound", name = var_name).to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn lookup_existing_variable() {
        let mut context = Context::new();
        context.lambda_variables.insert("x".to_string(), json!(42));
        
        let func = LambdaVariables {};
        let result = func.invoke(&[Value::String("x".to_string())], &context).unwrap();
        assert_eq!(result, json!(42));
    }

    #[test]
    fn lookup_nonexistent_variable() {
        let context = Context::new();
        let func = LambdaVariables {};
        let result = func.invoke(&[Value::String("x".to_string())], &context);
        assert!(result.is_err());
    }
}
