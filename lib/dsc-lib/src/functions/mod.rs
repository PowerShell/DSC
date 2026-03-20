// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use crate::DscError;
use crate::configure::context::{Context, ProcessMode};
use crate::functions::user_function::invoke_user_function;
use crate::schemas::dsc_repo::DscRepoSchema;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;
use std::fmt::Display;

pub mod add;
pub mod and;
pub mod array;
pub mod base64;
pub mod base64_to_string;
pub mod bool;
pub mod cidr_host;
pub mod cidr_subnet;
pub mod coalesce;
pub mod concat;
pub mod contains;
pub mod context;
pub mod copy_index;
pub mod create_array;
pub mod data_uri;
pub mod data_uri_to_string;
pub mod create_object;
pub mod div;
pub mod empty;
pub mod ends_with;
pub mod envvar;
pub mod equals;
pub mod filter;
pub mod greater;
pub mod greater_or_equals;
pub mod r#if;
pub mod r#false;
pub mod first;
pub mod last;
pub mod length;
pub mod less;
pub mod less_or_equals;
pub mod format;
pub mod int;
pub mod index_of;
pub mod intersection;
pub mod items;
pub mod join;
pub mod json;
pub mod lambda;
pub mod lambda_helpers;
pub mod lambda_variables;
pub mod last_index_of;
pub mod map;
pub mod max;
pub mod min;
pub mod mod_function;
pub mod mul;
pub mod not;
pub mod null;
pub mod object_keys;
pub mod or;
pub mod parameters;
pub mod parse_cidr;
pub mod path;
pub mod range;
pub mod reference;
pub mod resource_id;
pub mod secret;
pub mod shallow_merge;
pub mod skip;
pub mod starts_with;
pub mod stdout;
pub mod string;
pub mod take;
pub mod sub;
pub mod substring;
pub mod system_root;
pub mod to_lower;
pub mod to_upper;
pub mod trim;
pub mod r#true;
pub mod try_get;
pub mod try_index_from_end;
pub mod union;
pub mod unique_string;
pub mod uri;
pub mod uri_component;
pub mod uri_component_to_string;
pub mod user_function;
pub mod utc_now;
pub mod variables;
pub mod try_which;

/// The kind of argument that a function accepts.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "argKind", folder_path = "definitions/functions/builtin")]
pub enum FunctionArgKind {
    Array,
    Boolean,
    Lambda,
    Null,
    Number,
    Object,
    String,
}

impl Display for FunctionArgKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionArgKind::Array => write!(f, "Array"),
            FunctionArgKind::Boolean => write!(f, "Boolean"),
            FunctionArgKind::Lambda => write!(f, "Lambda"),
            FunctionArgKind::Null => write!(f, "Null"),
            FunctionArgKind::Number => write!(f, "Number"),
            FunctionArgKind::Object => write!(f, "Object"),
            FunctionArgKind::String => write!(f, "String"),
        }
    }
}

pub struct FunctionMetadata {
    pub name: String,
    pub description: String,
    pub category: Vec<FunctionCategory>,
    pub min_args: usize,
    pub max_args: usize,
    pub accepted_arg_ordered_types: Vec<Vec<FunctionArgKind>>,
    pub remaining_arg_accepted_types: Option<Vec<FunctionArgKind>>,
    pub return_types: Vec<FunctionArgKind>,
}

/// A function that can be invoked.
pub trait Function {
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
    fn get_metadata(&self) -> FunctionMetadata;
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
        let function_list : Vec<Box<dyn Function>> = vec![
            Box::new(add::Add{}),
            Box::new(and::And{}),
            Box::new(array::Array{}),
            Box::new(base64::Base64{}),
            Box::new(base64_to_string::Base64ToString{}),
            Box::new(bool::Bool{}),
            Box::new(cidr_host::CidrHost{}),
            Box::new(cidr_subnet::CidrSubnet{}),
            Box::new(coalesce::Coalesce{}),
            Box::new(concat::Concat{}),
            Box::new(contains::Contains{}),
            Box::new(context::Context{}),
            Box::new(copy_index::CopyIndex{}),
            Box::new(create_array::CreateArray{}),
            Box::new(create_object::CreateObject{}),
            Box::new(data_uri::DataUri{}),
            Box::new(data_uri_to_string::DataUriToString{}),
            Box::new(div::Div{}),
            Box::new(empty::Empty{}),
            Box::new(ends_with::EndsWith{}),
            Box::new(envvar::Envvar{}),
            Box::new(equals::Equals{}),
            Box::new(greater::Greater{}),
            Box::new(greater_or_equals::GreaterOrEquals{}),
            Box::new(r#if::If{}),
            Box::new(r#false::False{}),
            Box::new(first::First{}),
            Box::new(last::Last{}),
            Box::new(length::Length{}),
            Box::new(less::Less{}),
            Box::new(less_or_equals::LessOrEquals{}),
            Box::new(format::Format{}),
            Box::new(int::Int{}),
            Box::new(index_of::IndexOf{}),
            Box::new(intersection::Intersection{}),
            Box::new(items::Items{}),
            Box::new(join::Join{}),
            Box::new(json::Json{}),
            Box::new(filter::Filter{}),
            Box::new(lambda::LambdaFn{}),
            Box::new(lambda_variables::LambdaVariables{}),
            Box::new(last_index_of::LastIndexOf{}),
            Box::new(map::Map{}),
            Box::new(max::Max{}),
            Box::new(min::Min{}),
            Box::new(mod_function::Mod{}),
            Box::new(mul::Mul{}),
            Box::new(not::Not{}),
            Box::new(null::Null{}),
            Box::new(object_keys::ObjectKeys{}),
            Box::new(or::Or{}),
            Box::new(parameters::Parameters{}),
            Box::new(parse_cidr::ParseCidr{}),
            Box::new(path::Path{}),
            Box::new(range::Range{}),
            Box::new(reference::Reference{}),
            Box::new(resource_id::ResourceId{}),
            Box::new(secret::Secret{}),
            Box::new(shallow_merge::ShallowMerge{}),
            Box::new(skip::Skip{}),
            Box::new(starts_with::StartsWith{}),
            Box::new(stdout::Stdout{}),
            Box::new(string::StringFn{}),
            Box::new(sub::Sub{}),
            Box::new(take::Take{}),
            Box::new(substring::Substring{}),
            Box::new(system_root::SystemRoot{}),
            Box::new(to_lower::ToLower{}),
            Box::new(to_upper::ToUpper{}),
            Box::new(trim::Trim{}),
            Box::new(r#true::True{}),
            Box::new(try_get::TryGet{}),
            Box::new(try_index_from_end::TryIndexFromEnd{}),
            Box::new(union::Union{}),
            Box::new(unique_string::UniqueString{}),
            Box::new(uri::Uri{}),
            Box::new(uri_component::UriComponent{}),
            Box::new(uri_component_to_string::UriComponentToString{}),
            Box::new(utc_now::UtcNow{}),
            Box::new(variables::Variables{}),
            Box::new(try_which::TryWhich{}),
        ];
        for function in function_list {
            functions.insert(function.get_metadata().name.clone(), function);
        }

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
    pub fn invoke(&self, name: &str, args: &[Value], context: &Context) -> Result<Value, DscError> {
        let Some(function) = self.functions.get(name) else {
            // if function name contains a period, it might be a user function
            if name.contains('.') {
                return invoke_user_function(name, args, context);
            }
            return Err(DscError::Parser(t!("functions.unknownFunction", name = name).to_string()));
        };

        let metadata = function.get_metadata();

        // check if arg number are valid
        let min_args = metadata.min_args;
        let max_args = metadata.max_args;
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

        for (index, value) in args.iter().enumerate() {
            if index >= metadata.accepted_arg_ordered_types.len() {
                break;
            }

            Self::check_arg_against_expected_types(name, value, &metadata.accepted_arg_ordered_types[index])?;
        }

        // if we have remaining args, they must match one of the remaining_arg_types
        if let Some(ref remaining_arg_types) = metadata.remaining_arg_accepted_types {
            for value in args.iter().skip(metadata.accepted_arg_ordered_types.len()) {
                Self::check_arg_against_expected_types(name, value, remaining_arg_types)?;
            }
        }

        let accepts_lambda = metadata.accepted_arg_ordered_types.iter().any(|types| types.contains(&FunctionArgKind::Lambda))
            || metadata.remaining_arg_accepted_types.as_ref().is_some_and(|types| types.contains(&FunctionArgKind::Lambda));

        if accepts_lambda {
            let mut lambda_context = context.clone();
            lambda_context.process_mode = ProcessMode::Lambda;
            function.invoke(args, &lambda_context)
        } else {
            function.invoke(args, context)
        }
    }

    fn check_arg_against_expected_types(name: &str, arg: &Value, expected_types: &[FunctionArgKind]) -> Result<(), DscError> {
        let is_lambda = arg.as_str().is_some_and(|s| s.starts_with("__lambda_"));

        if arg.is_array() && !expected_types.contains(&FunctionArgKind::Array) {
            return Err(DscError::Parser(t!("functions.noArrayArgs", name = name, accepted_args_string = expected_types.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", ")).to_string()));
        } else if arg.is_boolean() && !expected_types.contains(&FunctionArgKind::Boolean) {
            return Err(DscError::Parser(t!("functions.noBooleanArgs", name = name, accepted_args_string = expected_types.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", ")).to_string()));
        } else if is_lambda && !expected_types.contains(&FunctionArgKind::Lambda) {
            return Err(DscError::Parser(t!("functions.noLambdaArgs", name = name, accepted_args_string = expected_types.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", ")).to_string()));
        } else if arg.is_null() && !expected_types.contains(&FunctionArgKind::Null) {
            return Err(DscError::Parser(t!("functions.noNullArgs", name = name, accepted_args_string = expected_types.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", ")).to_string()));
        } else if arg.is_number() && !expected_types.contains(&FunctionArgKind::Number) {
            return Err(DscError::Parser(t!("functions.noNumberArgs", name = name, accepted_args_string = expected_types.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", ")).to_string()));
        } else if arg.is_object() && !expected_types.contains(&FunctionArgKind::Object) {
            return Err(DscError::Parser(t!("functions.noObjectArgs", name = name, accepted_args_string = expected_types.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", ")).to_string()));
        } else if arg.is_string() && !is_lambda && !expected_types.contains(&FunctionArgKind::String) {
            return Err(DscError::Parser(t!("functions.noStringArgs", name = name, accepted_args_string = expected_types.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", ")).to_string()));
        }
        Ok(())
    }

    #[must_use]
    pub fn list(&self) -> Vec<FunctionDefinition> {
        self.functions.iter().map(|(name, function)| {
            let metadata = function.get_metadata();
            FunctionDefinition {
                category: metadata.category.clone(),
                name: name.clone(),
                description: metadata.description,
                min_args: metadata.min_args,
                max_args: metadata.max_args,
                accepted_arg_ordered_types: metadata.accepted_arg_ordered_types.clone(),
                remaining_arg_accepted_types: metadata.remaining_arg_accepted_types.clone(),
                return_types: metadata.return_types,
            }
        }).collect()
    }
}

impl Default for FunctionDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(base_name = "list", folder_path = "outputs/function")]
pub struct FunctionDefinition {
    pub category: Vec<FunctionCategory>,
    pub name: String,
    pub description: String,
    #[serde(rename = "minArgs")]
    pub min_args: usize,
    #[serde(rename = "maxArgs")]
    pub max_args: usize,
    #[serde(rename = "acceptedArgOrderedTypes")]
    pub accepted_arg_ordered_types: Vec<Vec<FunctionArgKind>>,
    #[serde(rename = "remainingArgAcceptedTypes")]
    pub remaining_arg_accepted_types: Option<Vec<FunctionArgKind>>,
    #[serde(rename = "returnTypes")]
    pub return_types: Vec<FunctionArgKind>,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(base_name = "category", folder_path = "definitions/functions/builtin")]
pub enum FunctionCategory {
    Array,
    Cidr,
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
            FunctionCategory::Cidr => write!(f, "CIDR"),
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
