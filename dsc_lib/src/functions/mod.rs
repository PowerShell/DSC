// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use crate::DscError;
use crate::configure::context::Context;
use serde_json::Value;

pub mod add;
pub mod base64;
pub mod concat;
pub mod create_array;
pub mod div;
pub mod envvar;
pub mod int;
pub mod max;
pub mod min;
pub mod mod_function;
pub mod mul;
pub mod parameters;
pub mod path;
pub mod reference;
pub mod resource_id;
pub mod sub;
pub mod system_root;
pub mod variables;

/// The kind of argument that a function accepts.
#[derive(Debug, PartialEq)]
pub enum AcceptedArgKind {
    Array,
    Boolean,
    Number,
    Object,
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
    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError>;
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
        functions.insert("add".to_string(), Box::new(add::Add{}));
        functions.insert("base64".to_string(), Box::new(base64::Base64{}));
        functions.insert("concat".to_string(), Box::new(concat::Concat{}));
        functions.insert("createArray".to_string(), Box::new(create_array::CreateArray{}));
        functions.insert("div".to_string(), Box::new(div::Div{}));
        functions.insert("envvar".to_string(), Box::new(envvar::Envvar{}));
        functions.insert("int".to_string(), Box::new(int::Int{}));
        functions.insert("max".to_string(), Box::new(max::Max{}));
        functions.insert("min".to_string(), Box::new(min::Min{}));
        functions.insert("mod".to_string(), Box::new(mod_function::Mod{}));
        functions.insert("mul".to_string(), Box::new(mul::Mul{}));
        functions.insert("parameters".to_string(), Box::new(parameters::Parameters{}));
        functions.insert("path".to_string(), Box::new(path::Path{}));
        functions.insert("reference".to_string(), Box::new(reference::Reference{}));
        functions.insert("resourceId".to_string(), Box::new(resource_id::ResourceId{}));
        functions.insert("sub".to_string(), Box::new(sub::Sub{}));
        functions.insert("systemRoot".to_string(), Box::new(system_root::SystemRoot{}));
        functions.insert("variables".to_string(), Box::new(variables::Variables{}));
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
    pub fn invoke(&self, name: &str, args: &Vec<Value>, context: &Context) -> Result<Value, DscError> {
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
        for value in args {
            if value.is_array() && !accepted_arg_types.contains(&AcceptedArgKind::Array) {
                return Err(DscError::Parser(format!("Function '{name}' does not accept array arguments, accepted types are: {accepted_args_string}")));
            } else if value.is_boolean() && !accepted_arg_types.contains(&AcceptedArgKind::Boolean) {
                return Err(DscError::Parser(format!("Function '{name}' does not accept boolean arguments, accepted types are: {accepted_args_string}")));
            } else if value.is_number() && !accepted_arg_types.contains(&AcceptedArgKind::Number) {
                return Err(DscError::Parser(format!("Function '{name}' does not accept number arguments, accepted types are: {accepted_args_string}")));
            } else if value.is_object() && !accepted_arg_types.contains(&AcceptedArgKind::Object) {
                return Err(DscError::Parser(format!("Function '{name}' does not accept object arguments, accepted types are: {accepted_args_string}")));
            } else if value.is_string() && !accepted_arg_types.contains(&AcceptedArgKind::String) {
                return Err(DscError::Parser(format!("Function '{name}' does not accept string argument, accepted types are: {accepted_args_string}")));
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
