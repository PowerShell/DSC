// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::dscerror::DscError;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, fmt::Display};
use tracing::trace;

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
pub struct Input {
    pub parameters: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ComplexInput {
    pub parameters: HashMap<String, InputObject>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct InputObject {
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ParametersJson {
    #[serde(rename = "parametersJson")]
    pub parameters_json: String,
}

pub const SECURE_VALUE_REDACTED: &str = "<secureValue>";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct SecureString {
    #[serde(rename = "secureString")]
    pub secure_string: String,
}

impl Display for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{SECURE_VALUE_REDACTED}")
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct SecureObject {
    #[serde(rename = "secureObject")]
    pub secure_object: Value,
}

impl Display for SecureObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{SECURE_VALUE_REDACTED}")
    }
}

/// Check if a given JSON value is a secure value (either `SecureString` or `SecureObject`).
///
/// # Arguments
///
/// * `value` - The JSON value to check.
///
/// # Returns
///
/// `true` if the value is a secure value, `false` otherwise.
#[must_use]
pub fn is_secure_value(value: &Value) -> bool {
    if let Some(obj) = value.as_object() {
        if obj.len() == 1 && (obj.contains_key("secureString") || obj.contains_key("secureObject")) {
            return true;
        }
    }
    false
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, untagged)]
pub enum SecureKind {
    #[serde(rename = "secureString")]
    SecureString(SecureString),
    #[serde(rename = "secureObject")]
    SecureObject(SecureObject),
}

pub fn import_parameters(parameters: &Value) -> Result<HashMap<String, Value>, DscError> {
    let input = match serde_json::from_value::<ParametersJson>(parameters.clone()) {
        Ok(input) => {
            trace!("{}", t!("configure.parameters.importingParametersFromJson"));
            let param_map = match serde_json::from_str::<ComplexInput>(&input.parameters_json) {
                Ok(param_map) => param_map,
                Err(e) => {
                    return Err(DscError::Parser(t!("configure.parameters.invalidParamsJsonFormat", error = e).to_string()));
                }
            };
            let mut result: HashMap<String, Value> = HashMap::new();
            for (name, input_object) in param_map.parameters {
                result.insert(name, input_object.value);
            }
            result
        },
        Err(_) => {
            let complex_input = match serde_json::from_value::<ComplexInput>(parameters.clone()) {
                Ok(complex_input) => {
                    trace!("{}", t!("configure.parameters.importingParametersFromComplexInput"));
                    let mut result: HashMap<String, Value> = HashMap::new();
                    for (name, input_object) in complex_input.parameters {
                        result.insert(name, input_object.value);
                    }
                    result
                },
                Err(_) => {
                    let simple_input = match serde_json::from_value::<Input>(parameters.clone()) {
                        Ok(simple_input) => {
                            trace!("{}", t!("configure.parameters.importingParametersFromInput"));
                            simple_input.parameters
                        }
                        Err(e) => {
                            return Err(DscError::Parser(t!("configure.parameters.invalidParamsFormat", error = e).to_string()));
                        }
                    };
                    simple_input
                }
            };
            complex_input
        },
    };
    Ok(input)
}
