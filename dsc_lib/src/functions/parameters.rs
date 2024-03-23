// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::config_doc::DataType;
use crate::configure::parameters::SecureKind;
use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use serde_json::Value;
use tracing::{debug, trace};

#[derive(Debug, Default)]
pub struct Parameters {}

impl Function for Parameters {
    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> usize {
        1
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String]
    }

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        debug!("Invoke parameters function");
        if let Some(key) = args[0].as_str() {
            trace!("parameters key: {key}");
            if context.parameters.contains_key(key) {
                let (value, data_type) = &context.parameters[key];

                // if secureString or secureObject types, we keep it as JSON object
                match data_type {
                    DataType::SecureString => {
                        let Some(value) = value.as_str() else {
                            return Err(DscError::Parser(format!("Parameter '{key}' is not a string")));
                        };
                        let secure_string = SecureKind::SecureString(value.to_string());
                        Ok(serde_json::to_value(secure_string)?)
                    },
                    DataType::SecureObject => {
                        let secure_object = SecureKind::SecureObject(value.clone());
                        Ok(serde_json::to_value(secure_object)?)
                    },
                    _ => {
                        Ok(value.clone())
                    }
                }
            }
            else {
                Err(DscError::Parser(format!("Parameter '{key}' not found in context")))
            }
        } else {
            Err(DscError::Parser("Invalid argument type".to_string()))
        }
    }
}
