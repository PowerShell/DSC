// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::config_doc::DataType;
use crate::configure::parameters::{SecureObject, SecureString};
use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::{debug, trace};

#[derive(Debug, Default)]
pub struct Parameters {}

impl Function for Parameters {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "parameters".to_string(),
            description: t!("functions.parameters.description").to_string(),
            category: vec![FunctionCategory::Deployment],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String, FunctionArgKind::Number, FunctionArgKind::Boolean, FunctionArgKind::Object, FunctionArgKind::Array, FunctionArgKind::Null],
        }
    }

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.parameters.invoked"));
        if let Some(key) = args[0].as_str() {
            trace!("{}", t!("functions.parameters.traceKey", key = key));
            if context.parameters.contains_key(key) {
                let (value, data_type) = &context.parameters[key];

                // if secureString or secureObject types, we keep it as JSON object
                match data_type {
                    DataType::SecureString => {
                        let Some(value) = value.as_str() else {
                            return Err(DscError::Parser(t!("functions.parameters.keyNotString", key = key).to_string()));
                        };
                        let secure_string = SecureString {
                            secure_string: value.to_string(),
                        };
                        Ok(serde_json::to_value(secure_string)?)
                    },
                    DataType::SecureObject => {
                        let secure_object = SecureObject {
                            secure_object: value.clone(),
                        };
                        Ok(serde_json::to_value(secure_object)?)
                    },
                    DataType::String => {
                        let Some(value) = value.as_str() else {
                            return Err(DscError::Parser(t!("functions.parameters.keyNotString", key = key).to_string()));
                        };
                        Ok(serde_json::to_value(value)?)
                    },
                    DataType::Int => {
                        let Some(value) = value.as_i64() else {
                            return Err(DscError::Parser(t!("functions.parameters.keyNotInt", key = key).to_string()));
                        };
                        Ok(serde_json::to_value(value)?)
                    },
                    DataType::Bool => {
                        let Some(value) = value.as_bool() else {
                            return Err(DscError::Parser(t!("functions.parameters.keyNotBool", key = key).to_string()));
                        };
                        Ok(serde_json::to_value(value)?)
                    },
                    DataType::Object => {
                        let Some(value) = value.as_object() else {
                            return Err(DscError::Parser(t!("functions.parameters.keyNotObject", key = key).to_string()));
                        };
                        Ok(serde_json::to_value(value)?)
                    },
                    DataType::Array => {
                        let Some(value) = value.as_array() else {
                            return Err(DscError::Parser(t!("functions.parameters.keyNotArray", key = key).to_string()));
                        };
                        Ok(serde_json::to_value(value)?)
                    },
                }
            }
            else {
                Err(DscError::Parser(t!("functions.parameters.keyNotFound", key = key).to_string()))
            }
        } else {
            Err(DscError::Parser(t!("functions.invalidArgType").to_string()))
        }
    }
}
