// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::config_doc::Parameter;
use crate::DscError;
use serde_json::Value;

pub fn check_length(name: &str, value: &Value, constraint: &Parameter) -> Result<(), DscError> {
    if let Some(min_length) = constraint.min_length {
        if value.is_string() {
            let value = value.as_str().unwrap();
            if value.len() < min_length as usize {
                return Err(DscError::Validation(format!("Parameter '{name}' has minimum length constraint of {min_length} but is {0}", value.len())));
            }
        }
        else if value.is_array() {
            let value = value.as_array().unwrap();
            if value.len() < min_length as usize {
                return Err(DscError::Validation(format!("Parameter '{name}' has minimum length constraint of {min_length} but is {0}", value.len())));
            }
        }
        else {
            return Err(DscError::Validation(format!("Parameter '{name}' has minimum length constraint but is not a string or array")));
        }
    }

    if let Some(max_length) = constraint.max_length {
        if value.is_string() {
            let value = value.as_str().unwrap();
            if value.len() > max_length as usize {
                return Err(DscError::Validation(format!("Parameter '{name}' has maximum length constraint of {max_length} but is {0}", value.len())));
            }
        }
        else if value.is_array() {
            let value = value.as_array().unwrap();
            if value.len() > max_length as usize {
                return Err(DscError::Validation(format!("Parameter '{name}' has maximum length constraint of {max_length} but is {0}", value.len())));
            }
        }
        else {
            return Err(DscError::Validation(format!("Parameter '{name}' has maximum length constraint but is not a string or array")));
        }
    }

    Ok(())
}

pub fn check_number(name: &str, value: &Value, constraint: &Parameter) -> Result<(), DscError> {
    if let Some(min_value) = constraint.min_value {
        if value.is_i64() && value.as_i64().is_some() {
            let value = value.as_i64().unwrap();
            if value < min_value {
                return Err(DscError::Validation(format!("Parameter '{name}' has minimum value constraint of {min_value} but is {0}", value)));
            }
        }
        else {
            return Err(DscError::Validation(format!("Parameter '{name}' has minimum value constraint but is not an integer")));
        }
    }

    if let Some(max_value) = constraint.max_value {
        if value.is_i64() && value.as_i64().is_some() {
            let value = value.as_i64().unwrap();
            if value > max_value {
                return Err(DscError::Validation(format!("Parameter '{name}' has maximum value constraint of {max_value} but is {0}", value)));
            }
        }
        else {
            return Err(DscError::Validation(format!("Parameter '{name}' has maximum value constraint but is not an integer")));
        }
    }

    Ok(())
}

pub fn check_allowed_values(name: &str, value: &Value, constraint: &Parameter) -> Result<(), DscError> {
    if let Some(allowed_values) = &constraint.allowed_values {
        if value.is_string() && value.as_str().is_some(){
            let value = value.as_str().unwrap();
            if !allowed_values.contains(&Value::String(value.to_string())) {
                return Err(DscError::Validation(format!("Parameter '{name}' has value not in the allowed values list")));
            }
        }
        else if value.is_i64() && value.as_i64().is_some() {
            let value = value.as_i64().unwrap();
            if !allowed_values.contains(&Value::Number(value.into())) {
                return Err(DscError::Validation(format!("Parameter '{name}' has value not in the allowed values list")));
            }
        }
        else {
            return Err(DscError::Validation(format!("Parameter '{name}' has allowed values constraint but is not a string or integer")));
        }
    }

    Ok(())
}

// TODO: check nullable
