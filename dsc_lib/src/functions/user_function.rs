// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::{config_doc::{DataType, UserFunctionDefinition}, context::ProcessMode};
use crate::dscerror::DscError;
use crate::functions::Context;
use crate::parser::Statement;
use serde_json::Value;

pub fn invoke_user_function(name: &str, args: &[Value], context: &Context) -> Result<Value, DscError> {
    if let Some(function_definition) = context.user_functions.get(name.to_lowercase().as_str()) {
        validate_parameters(name, function_definition, args)?;
        let mut user_context = context.clone();
        user_context.process_mode = ProcessMode::UserFunction;
        // can only use its own parameters and not the global ones
        user_context.parameters.clear();
        // cannot call other user functions
        user_context.user_functions.clear();
        for (i, arg) in args.iter().enumerate() {
            let Some(params) = &function_definition.parameters else {
                return Err(DscError::Parser(format!("Function '{}' does not accept any parameters, but {} were provided", name, args.len())));
            };
            user_context.parameters.insert(params[i].name.clone(), (arg.clone(), params[i].r#type.clone()));
        }
        let mut parser = Statement::new()?;
        let result = parser.parse_and_execute(&function_definition.output.value, &user_context)?;
        validate_output_type(name, function_definition, &result)?;
        Ok(result)
    } else {
        Err(DscError::Parser(format!("Unknown user function: {}", name)))
    }
}

fn validate_parameters(name: &str, function_definition: &UserFunctionDefinition, args: &[Value]) -> Result<(), DscError> {
    if let Some(expected_params) = &function_definition.parameters {
        if args.len() != expected_params.len() {
            return Err(DscError::Parser(format!("Function '{}' expects {} parameters, but {} were provided", name, expected_params.len(), args.len())));
        }
        for (i, (arg, expected_param)) in args.iter().zip(expected_params).enumerate() {
            match expected_param.r#type {
                DataType::String => {
                    if !arg.is_string() {
                        return Err(DscError::Parser(format!("Parameter {} of function '{}' expects a string", i + 1, name)));
                    }
                }
                DataType::Int => {
                    if !arg.is_number() {
                        return Err(DscError::Parser(format!("Parameter {} of function '{}' expects a number", i + 1, name)));
                    }
                }
                DataType::Bool => {
                    if !arg.is_boolean() {
                        return Err(DscError::Parser(format!("Parameter {} of function '{}' expects a boolean", i + 1, name)));
                    }
                }
                DataType::Array => {
                    if !arg.is_array() {
                        return Err(DscError::Parser(format!("Parameter {} of function '{}' expects an array", i + 1, name)));
                    }
                }
                DataType::Object => {
                    if !arg.is_object() {
                        return Err(DscError::Parser(format!("Parameter {} of function '{}' expects an object", i + 1, name)));
                    }
                }
                _ => {
                    return Err(DscError::Parser(format!("Function '{}' has an unsupported parameter type: {:?}", name, expected_param.r#type)));
                }
            }
        }
    } else {
        if !args.is_empty() {
            return Err(DscError::Parser(format!("Function '{}' does not accept any parameters, but {} were provided", name, args.len())));
        }
    }
    Ok(())
}

fn validate_output_type(name: &str, function_definition: &UserFunctionDefinition, result: &Value) -> Result<(), DscError> {
    match function_definition.output.r#type {
        DataType::String => {
            if !result.is_string() {
                return Err(DscError::Parser(format!("Function '{}' is expected to return a string", name)));
            }
        }
        DataType::Int => {
            if !result.is_number() {
                return Err(DscError::Parser(format!("Function '{}' is expected to return a number", name)));
            }
        }
        DataType::Bool => {
            if !result.is_boolean() {
                return Err(DscError::Parser(format!("Function '{}' is expected to return a boolean", name)));
            }
        }
        DataType::Array => {
            if !result.is_array() {
                return Err(DscError::Parser(format!("Function '{}' is expected to return an array", name)));
            }
        }
        DataType::Object => {
            if !result.is_object() {
                return Err(DscError::Parser(format!("Function '{}' is expected to return an object", name)));
            }
        }
        _ => {
            return Err(DscError::Parser(format!("Function '{}' has an unsupported output type: {:?}", name, function_definition.output.r#type)));
        }
    }
    Ok(())
}
