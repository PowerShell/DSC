// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use crate::DscError;
use crate::configure::context::Context;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;
use std::fmt::Display;

pub mod add;
pub mod and;
pub mod base64;
pub mod bool;
pub mod concat;
pub mod create_array;
pub mod div;
pub mod envvar;
pub mod equals;
pub mod r#if;
pub mod r#false;
pub mod format;
pub mod int;
pub mod max;
pub mod min;
pub mod mod_function;
pub mod mul;
pub mod not;
pub mod or;
pub mod parameters;
pub mod path;
pub mod reference;
pub mod resource_id;
pub mod secret;
pub mod sub;
pub mod system_root;
pub mod r#true;
pub mod variables;

/// The kind of argument that a function accepts.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, JsonSchema)]
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
    fn description(&self) -> String;
    fn category(&self) -> FunctionCategory;
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
        functions.insert("and".to_string(), Box::new(and::And{}));
        functions.insert("base64".to_string(), Box::new(base64::Base64{}));
        functions.insert("bool".to_string(), Box::new(bool::Bool{}));
        functions.insert("concat".to_string(), Box::new(concat::Concat{}));
        functions.insert("createArray".to_string(), Box::new(create_array::CreateArray{}));
        functions.insert("div".to_string(), Box::new(div::Div{}));
        functions.insert("envvar".to_string(), Box::new(envvar::Envvar{}));
        functions.insert("equals".to_string(), Box::new(equals::Equals{}));
        functions.insert("false".to_string(), Box::new(r#false::False{}));
        functions.insert("if".to_string(), Box::new(r#if::If{}));
        functions.insert("format".to_string(), Box::new(format::Format{}));
        functions.insert("int".to_string(), Box::new(int::Int{}));
        functions.insert("max".to_string(), Box::new(max::Max{}));
        functions.insert("min".to_string(), Box::new(min::Min{}));
        functions.insert("mod".to_string(), Box::new(mod_function::Mod{}));
        functions.insert("mul".to_string(), Box::new(mul::Mul{}));
        functions.insert("not".to_string(), Box::new(not::Not{}));
        functions.insert("or".to_string(), Box::new(or::Or{}));
        functions.insert("parameters".to_string(), Box::new(parameters::Parameters{}));
        functions.insert("path".to_string(), Box::new(path::Path{}));
        functions.insert("reference".to_string(), Box::new(reference::Reference{}));
        functions.insert("resourceId".to_string(), Box::new(resource_id::ResourceId{}));
        functions.insert("secret".to_string(), Box::new(secret::Secret{}));
        functions.insert("sub".to_string(), Box::new(sub::Sub{}));
        functions.insert("systemRoot".to_string(), Box::new(system_root::SystemRoot{}));
        functions.insert("true".to_string(), Box::new(r#true::True{}));
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
            return Err(DscError::Parser(t!("functions.unknownFunction", name = name).to_string()));
        };

        // check if arg number are valid
        let min_args = function.min_args();
        let max_args = function.max_args();
        if args.len() < min_args || args.len() > max_args {
            if max_args == 0 {
                return Err(DscError::Parser(t!("functions.noArgsAccepted", name = name).to_string()));
            }
            else if min_args == max_args {
                return Err(DscError::Parser(t!("functions.invalidArgCount", name = name, count = min_args).to_string()));
            }
            else if max_args == usize::MAX {
                return Err(DscError::Parser(t!("functions.minArgsRequired", name = name, count = min_args).to_string()));
            }

            return Err(DscError::Parser(t!("functions.argCountRequired", name = name, min = min_args, max = max_args).to_string()));
        }
        // check if arg types are valid
        let accepted_arg_types = function.accepted_arg_types();
        let accepted_args_string = accepted_arg_types.iter().map(|x| format!("{x:?}")).collect::<Vec<String>>().join(", ");
        for value in args {
            if value.is_array() && !accepted_arg_types.contains(&AcceptedArgKind::Array) {
                return Err(DscError::Parser(t!("functions.noArrayArgs", name = name, accepted_args_string = accepted_args_string).to_string()));
            } else if value.is_boolean() && !accepted_arg_types.contains(&AcceptedArgKind::Boolean) {
                return Err(DscError::Parser(t!("functions.noBooleanArgs", name = name, accepted_args_string = accepted_args_string).to_string()));
            } else if value.is_number() && !accepted_arg_types.contains(&AcceptedArgKind::Number) {
                return Err(DscError::Parser(t!("functions.noNumberArgs", name = name, accepted_args_string = accepted_args_string).to_string()));
            } else if value.is_object() && !accepted_arg_types.contains(&AcceptedArgKind::Object) {
                return Err(DscError::Parser(t!("functions.noObjectArgs", name = name, accepted_args_string = accepted_args_string).to_string()));
            } else if value.is_string() && !accepted_arg_types.contains(&AcceptedArgKind::String) {
                return Err(DscError::Parser(t!("functions.noStringArgs", name = name, accepted_args_string = accepted_args_string).to_string()));
            }
        }

        function.invoke(args, context)
    }

    #[must_use]
    pub fn list(&self) -> Vec<FunctionDefinition> {
        self.functions.iter().map(|(name, function)| {
            FunctionDefinition {
                category: function.category(),
                name: name.clone(),
                description: function.description(),
                min_args: function.min_args(),
                max_args: function.max_args(),
                accepted_arg_types: function.accepted_arg_types().clone(),
            }
        }).collect()
    }
}

impl Default for FunctionDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunctionDefinition {
    pub category: FunctionCategory,
    pub name: String,
    pub description: String,
    #[serde(rename = "minArgs")]
    pub min_args: usize,
    #[serde(rename = "maxArgs")]
    pub max_args: usize,
    #[serde(rename = "acceptedArgTypes")]
    pub accepted_arg_types: Vec<AcceptedArgKind>,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub enum FunctionCategory {
    Array,
    Comparison,
    Date,
    Deployment,
    Lambda,
    Logical,
    Numeric,
    Object,
    Resource,
    String,
    System,
}

impl Display for FunctionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionCategory::Array => write!(f, "Array"),
            FunctionCategory::Comparison => write!(f, "Comparison"),
            FunctionCategory::Date => write!(f, "Date"),
            FunctionCategory::Deployment => write!(f, "Deployment"),
            FunctionCategory::Lambda => write!(f, "Lambda"),
            FunctionCategory::Logical => write!(f, "Logical"),
            FunctionCategory::Numeric => write!(f, "Numeric"),
            FunctionCategory::Object => write!(f, "Object"),
            FunctionCategory::Resource => write!(f, "Resource"),
            FunctionCategory::String => write!(f, "String"),
            FunctionCategory::System => write!(f, "System"),
        }
    }
}
