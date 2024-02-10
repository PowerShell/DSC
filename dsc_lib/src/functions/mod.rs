// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;
use tracing::debug;

use crate::DscError;
use crate::configure::context::Context;
use crate::parser::functions::{FunctionArg, FunctionResult};

pub mod base64;
pub mod concat;
pub mod envvar;
pub mod parameters;
pub mod resource_id;

/// The kind of argument that a function accepts.
#[derive(Debug, PartialEq)]
pub enum AcceptedArgKind {
    Boolean,
    Integer,
    String,
}

/// A function that can be invoked.
pub trait Function {
    /// The minimum number of arguments that the function accepts.
    fn min_args(&self) -> usize;
    /// The maximum number of arguments that the function accepts.
    fn max_args(&self) -> usize;
    /// The types of arguments that the function accepts.
    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind>;
    /// Invoke the function.
    ///
    /// # Arguments
    ///
    /// * `args` - The arguments to the function.
    ///
    /// # Errors
    ///
    /// This function will return an error if the function fails to execute.
    fn invoke(&self, args: &[FunctionArg], context: &Context) -> Result<FunctionResult, DscError>;
}

/// A dispatcher for functions.
pub struct FunctionDispatcher {
    functions: HashMap<String, Box<dyn Function>>,
}

impl FunctionDispatcher {
    /// Create a new `FunctionDispatcher` instance.
    #[must_use]
    pub fn new() -> Self {
        let mut functions: HashMap<String, Box<dyn Function>> = HashMap::new();
        functions.insert("base64".to_string(), Box::new(base64::Base64{}));
        functions.insert("concat".to_string(), Box::new(concat::Concat{}));
        functions.insert("envvar".to_string(), Box::new(envvar::Envvar{}));
        functions.insert("parameters".to_string(), Box::new(parameters::Parameters{}));
        functions.insert("resourceId".to_string(), Box::new(resource_id::ResourceId{}));
        Self {
            functions,
        }
    }

    /// Invoke a function.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function to invoke.
    /// * `args` - The arguments to the function.
    ///
    /// # Errors
    ///
    /// This function will return an error if the function fails to execute.
    pub fn invoke(&self, name: &str, args: &Vec<FunctionArg>, context: &Context) -> Result<FunctionResult, DscError> {
        let Some(function) = self.functions.get(name) else {
            return Err(DscError::Parser(format!("Unknown function '{name}'")));
        };

        // check if arg number are valid
        let min_args = function.min_args();
        let max_args = function.max_args();
        if args.len() < min_args || args.len() > max_args {
            if max_args == 0 {
                return Err(DscError::Parser(format!("Function '{name}' does not accept arguments")));
            }
            else if min_args == max_args {
                return Err(DscError::Parser(format!("Function '{name}' requires exactly {min_args} arguments")));
            }
            else if max_args == usize::MAX {
                return Err(DscError::Parser(format!("Function '{name}' requires at least {min_args} arguments")));
            }

            return Err(DscError::Parser(format!("Function '{name}' requires between {min_args} and {max_args} arguments")));
        }
        // check if arg types are valid
        let accepted_arg_types = function.accepted_arg_types();
        let accepted_args_string = accepted_arg_types.iter().map(|x| format!("{x:?}")).collect::<Vec<String>>().join(", ");
        for arg in args {
            match arg {
                FunctionArg::String(_) => {
                    if !accepted_arg_types.contains(&AcceptedArgKind::String) {
                        return Err(DscError::Parser(format!("Function '{name}' does not accept string argument, accepted types are: {accepted_args_string}")));
                    }
                },
                FunctionArg::Integer(_) => {
                    if !accepted_arg_types.contains(&AcceptedArgKind::Integer) {
                        return Err(DscError::Parser(format!("Function '{name}' does not accept integer arguments, accepted types are: {accepted_args_string}")));
                    }
                },
                FunctionArg::Boolean(_) => {
                    if !accepted_arg_types.contains(&AcceptedArgKind::Boolean) {
                        return Err(DscError::Parser(format!("Function '{name}' does not accept boolean arguments, accepted types are: {accepted_args_string}")));
                    }
                },
                FunctionArg::Expression(_) => {
                    debug!("An expression was not resolved before invoking a function");
                    return Err(DscError::Parser("Error in parsing".to_string()));
                }
            }
        }

        function.invoke(args, context)
    }
}

impl Default for FunctionDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
