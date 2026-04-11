// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::{{config_doc::{DataType, UserFunctionDefinition}, context::ProcessMode}, validate_parameter_type};
use crate::dscerror::DscError;
use crate::functions::Context;
use crate::parser::Statement;
use rust_i18n::t;
use serde_json::Value;

/// Invoke a user-defined function by name with the provided arguments and context.
///
/// # Arguments
/// * `name` - The name of the user-defined function to invoke.
/// * `args` - The arguments to pass to the user-defined function.
/// * `context` - The current execution context.
///
/// # Returns
/// * `Result<Value, DscError>` - The result of the function invocation or an error.
///
/// # Errors
/// * `DscError::Parser` - If the function is not found, parameters are invalid, or output type is incorrect.
///
pub fn invoke_user_function(name: &str, args: &[Value], context: &Context) -> Result<Value, DscError> {
    if let Some(function_definition) = context.user_functions.get(name) {
        validate_parameters(name, function_definition, args)?;
        let mut user_context = Context {
            process_mode: ProcessMode::UserFunction,
            parameters: HashMap::new(),
            user_functions: HashMap::new(),
            // Copy other necessary fields by reference or shallow copy
            copy: context.copy.clone(),
            copy_current_loop_name: context.copy_current_loop_name.clone(),
            dsc_version: context.dsc_version.clone(),
            execution_type: context.execution_type,
            extensions: context.extensions.clone(),
            lambda_raw_args: std::cell::RefCell::new(None),
            lambda_variables: HashMap::new(),
            lambdas: std::cell::RefCell::new(HashMap::new()),
            operation: context.operation,
            outputs: context.outputs.clone(),
            processing_parameter_defaults: context.processing_parameter_defaults,
            process_expressions: context.process_expressions,
            references: context.references.clone(),
            restart_required: context.restart_required.clone(),
            security_context: context.security_context.clone(),
            start_datetime: context.start_datetime,
            stdout: context.stdout.clone(),
            system_root: context.system_root.clone(),
            variables: context.variables.clone(),
        };
        for (i, arg) in args.iter().enumerate() {
            let Some(params) = &function_definition.parameters else {
                return Err(DscError::Parser(t!("functions.userFunction.expectedNoParameters", name = name).to_string()));
            };
            user_context.parameters.insert(params[i].name.to_string(), (arg.clone(), params[i].r#type.clone()));
        }
        let mut parser = Statement::new()?;
        let result = parser.parse_and_execute(&function_definition.output.value, &user_context)?;
        validate_output_type(name, function_definition, &result)?;
        Ok(result)
    } else {
        Err(DscError::Parser(t!("functions.userFunction.unknownUserFunction", name = name).to_string()))
    }
}

fn validate_parameters(name: &str, function_definition: &UserFunctionDefinition, args: &[Value]) -> Result<(), DscError> {
    if let Some(expected_params) = &function_definition.parameters {
        if args.len() != expected_params.len() {
            return Err(DscError::Parser(t!("functions.userFunction.wrongParamCount", name = name, expected = expected_params.len(), got = args.len()).to_string()));
        }
        for (arg, expected_param) in args.iter().zip(expected_params) {
            validate_parameter_type(name, arg, &expected_param.r#type)?;
        }
    }
    Ok(())
}

fn validate_output_type(name: &str, function_definition: &UserFunctionDefinition, output: &Value) -> Result<(), DscError> {
    match function_definition.output.r#type {
        DataType::String | DataType::SecureString => {
            if !output.is_string() {
                return Err(DscError::Validation(t!("functions.userFunction.incorrectOutputType", name = name, expected_type = "string").to_string()));
            }
        },
        DataType::Int => {
            if !output.is_i64() {
                return Err(DscError::Validation(t!("functions.userFunction.incorrectOutputType", name = name, expected_type = "int").to_string()));
            }
        },
        DataType::Bool => {
            if !output.is_boolean() {
                return Err(DscError::Validation(t!("functions.userFunction.incorrectOutputType", name = name, expected_type = "bool").to_string()));
            }
        },
        DataType::Array => {
            if !output.is_array() {
                return Err(DscError::Validation(t!("functions.userFunction.incorrectOutputType", name = name, expected_type = "array").to_string()));
            }
        },
        DataType::Object | DataType::SecureObject => {
            if !output.is_object() {
                return Err(DscError::Validation(t!("functions.userFunction.incorrectOutputType", name = name, expected_type = "object").to_string()));
            }
        },
    }

    Ok(())
}
