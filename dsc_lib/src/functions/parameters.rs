// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::config_doc::DataType;
use crate::configure::parameters::SecureKind;
use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::{debug, trace};

#[derive(Debug, Default)]
pub struct Parameters {}

impl Function for Parameters {
    fn description(&self) -> String {
        t!("functions.parameters.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Deployment
    }

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
        debug!("{}", t!("functions.parameters.invoked"));
        if let Some(key) = args[0].as_str() {
            trace!("{}", t!("functions.parameters.traceKey", key = key));
            if context.parameters.contains_key(key) {
                let (value, data_type) = &context.parameters[key];

                // Handle null values explicitly
                if value.is_null() {
                    return Ok(Value::Null);
                }

                // if secureString or secureObject types, we keep it as JSON object
                match data_type {
                    DataType::SecureString => {
                        let Some(value) = value.as_str() else {
                            return Err(DscError::Parser(t!("functions.parameters.keyNotString", key = key).to_string()));
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
                Err(DscError::Parser(t!("functions.parameters.keyNotFound", key = key).to_string()))
            }
        } else {
            Err(DscError::Parser(t!("functions.invalidArgType").to_string()))
        }
    }
}
