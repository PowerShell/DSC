// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;
use tracing::debug;

use crate::DscError;
use crate::parser::functions::{FunctionArg, FunctionResult};

pub mod base64;
pub mod concat;

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
    fn invoke(&self, args: &[FunctionArg]) -> Result<FunctionResult, DscError>;
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
    pub fn invoke(&self, name: &str, args: &Vec<FunctionArg>) -> Result<FunctionResult, DscError> {
        let function = self.functions.get(name);
        match function {
            Some(function) => {
                // check if arg number are valid
                if args.len() < function.min_args() {
                    return Err(DscError::Parser(format!("Function '{name}' requires at least {0} arguments", function.min_args())));
                }
                if args.len() > function.max_args() {
                    return Err(DscError::Parser(format!("Function '{name}' supports at most {0} arguments", function.max_args())));
                }
                // check if arg types are valid
                for arg in args {
                    match arg {
                        FunctionArg::String(_) => {
                            if !function.accepted_arg_types().contains(&AcceptedArgKind::String) {
                                return Err(DscError::Parser(format!("Function '{name}' does not accept string arguments")));
                            }
                        },
                        FunctionArg::Integer(_) => {
                            if !function.accepted_arg_types().contains(&AcceptedArgKind::Integer) {
                                return Err(DscError::Parser(format!("Function '{name}' does not accept integer arguments")));
                            }
                        },
                        FunctionArg::Boolean(_) => {
                            if !function.accepted_arg_types().contains(&AcceptedArgKind::Boolean) {
                                return Err(DscError::Parser(format!("Function '{name}' does not accept boolean arguments")));
                            }
                        },
                        FunctionArg::Expression(_) => {
                            debug!("An expression was not resolved before invoking a function");
                            return Err(DscError::Parser("Error in parsing".to_string()));
                        }
                    }
                }

                function.invoke(args)
            },
            None => Err(DscError::Parser(format!("Unknown function '{name}'"))),
        }
    }
}

impl Default for FunctionDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
