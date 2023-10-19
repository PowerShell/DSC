// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;
use crate::DscError;
use crate::parser::functions::{FunctionArg, FunctionResult};
use tracing::debug;

pub mod base64;
pub mod concat;

#[derive(Debug, PartialEq)]
pub enum AcceptedArgKind {
    String,
    Integer,
    Boolean,
}

pub trait Function {
    fn min_args(&self) -> usize;
    fn max_args(&self) -> usize;
    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind>;
    fn invoke(&self, args: &Vec<FunctionArg>) -> Result<FunctionResult, DscError>;
}

pub struct FunctionDispatcher {
    functions: HashMap<String, Box<dyn Function>>,
}

impl FunctionDispatcher {
    pub fn new() -> Self {
        let mut functions: HashMap<String, Box<dyn Function>> = HashMap::new();
        functions.insert("base64".to_string(), Box::new(base64::Base64{}));
        functions.insert("concat".to_string(), Box::new(concat::Concat{}));
        Self {
            functions,
        }
    }

    pub fn invoke(&self, name: &str, args: &Vec<FunctionArg>) -> Result<FunctionResult, DscError> {
        let function = self.functions.get(name);
        match function {
            Some(function) => {
                // check if arg number are valid
                if args.len() < function.min_args() {
                    return Err(DscError::Parser(format!("Function {0} requires at least {1} arguments", name, function.min_args())));
                }
                if args.len() > function.max_args() {
                    return Err(DscError::Parser(format!("Function {0} requires at most {1} arguments", name, function.max_args())));
                }
                // check if arg types are valid
                for arg in args {
                    match arg {
                        FunctionArg::String(_) => {
                            if !function.accepted_arg_types().contains(&AcceptedArgKind::String) {
                                return Err(DscError::Parser(format!("Function {0} does not accept string arguments", name)));
                            }
                        },
                        FunctionArg::Integer(_) => {
                            if !function.accepted_arg_types().contains(&AcceptedArgKind::Integer) {
                                return Err(DscError::Parser(format!("Function {0} does not accept integer arguments", name)));
                            }
                        },
                        FunctionArg::Boolean(_) => {
                            if !function.accepted_arg_types().contains(&AcceptedArgKind::Boolean) {
                                return Err(DscError::Parser(format!("Function {0} does not accept boolean arguments", name)));
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
            None => Err(DscError::Parser(format!("Unknown function {0}", name))),
        }
    }
}
