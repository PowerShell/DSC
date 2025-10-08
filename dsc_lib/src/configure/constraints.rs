// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::config_doc::Parameter;
use crate::DscError;
use rust_i18n::t;
use serde_json::Value;

/// Checks that the given value matches the given parameter length constraints.
///
/// # Arguments
///
/// * `name` - The name of the parameter.
/// * `value` - The value of the parameter.
/// * `constraint` - The constraints on the parameter.
///
/// # Returns
///
/// * `Ok(())` if the value matches the constraints.
/// * `Err(DscError::Validation)` if the value does not match the constraints.
///
/// # Errors
///
/// * `DscError::Validation` if the value does not match the constraints.
pub fn check_length(name: &str, value: &Value, constraint: &Parameter) -> Result<(), DscError> {
    if let Some(min_length) = constraint.min_length {
        if value.is_string() {
            let Some(value) = value.as_str() else {
                return Err(DscError::Validation(
                    t!("configure.constraints.minLengthIsNull", name = name).to_string(),
                ));
            };

            if value.len() < usize::try_from(min_length)? {
                return Err(DscError::Validation(
                    t!(
                        "configure.constraints.notMinLength",
                        name = name,
                        min_length = min_length,
                        length = value.len()
                    )
                    .to_string(),
                ));
            }
        } else if value.is_array() {
            let Some(value) = value.as_array() else {
                return Err(DscError::Validation(
                    t!("configure.constraints.minLengthIsNull", name = name).to_string(),
                ));
            };

            if value.len() < usize::try_from(min_length)? {
                return Err(DscError::Validation(
                    t!(
                        "configure.constraints.notMinLength",
                        name = name,
                        min_length = min_length,
                        length = value.len()
                    )
                    .to_string(),
                ));
            }
        } else {
            return Err(DscError::Validation(
                t!("configure.constraints.minLengthNotStringOrArray", name = name).to_string(),
            ));
        }
    }

    if let Some(max_length) = constraint.max_length {
        if value.is_string() {
            let Some(value) = value.as_str() else {
                return Err(DscError::Validation(
                    t!("configure.constraints.maxLengthIsNull", name = name).to_string(),
                ));
            };

            if value.len() > usize::try_from(max_length)? {
                return Err(DscError::Validation(
                    t!(
                        "configure.constraints.maxLengthExceeded",
                        name = name,
                        max_length = max_length,
                        length = value.len()
                    )
                    .to_string(),
                ));
            }
        } else if value.is_array() {
            let Some(value) = value.as_array() else {
                return Err(DscError::Validation(
                    t!("configure.constraints.maxLengthIsNull", name = name).to_string(),
                ));
            };

            if value.len() > usize::try_from(max_length)? {
                return Err(DscError::Validation(
                    t!(
                        "configure.constraints.maxLengthExceeded",
                        name = name,
                        max_length = max_length,
                        length = value.len()
                    )
                    .to_string(),
                ));
            }
        } else {
            return Err(DscError::Validation(
                t!("configure.constraints.maxLengthNotStringOrArray", name = name).to_string(),
            ));
        }
    }

    Ok(())
}

/// Checks that the given value matches the given number value constraints.
///
/// # Arguments
///
/// * `name` - The name of the parameter.
/// * `value` - The value of the parameter.
/// * `constraint` - The constraints on the parameter.
///
/// # Returns
///
/// * `Ok(())` if the value matches the constraints.
/// * `Err(DscError::Validation)` if the value does not match the constraints.
///
/// # Errors
///
/// * `DscError::Validation` if the value does not match the constraints.
pub fn check_number_limits(name: &str, value: &Value, constraint: &Parameter) -> Result<(), DscError> {
    if let Some(min_value) = constraint.min_value {
        if value.is_i64() && value.as_i64().is_some() {
            let Some(value) = value.as_i64() else {
                return Err(DscError::Validation(
                    t!("configure.constraints.minValueIsNull", name = name).to_string(),
                ));
            };

            if value < min_value {
                return Err(DscError::Validation(
                    t!(
                        "configure.constraints.notMinValue",
                        name = name,
                        min_value = min_value,
                        value = value
                    )
                    .to_string(),
                ));
            }
        } else {
            return Err(DscError::Validation(
                t!("configure.constraints.minValueNotInteger", name = name).to_string(),
            ));
        }
    }

    if let Some(max_value) = constraint.max_value {
        if value.is_i64() && value.as_i64().is_some() {
            let Some(value) = value.as_i64() else {
                return Err(DscError::Validation(
                    t!("configure.constraints.maxValueIsNull", name = name).to_string(),
                ));
            };

            if value > max_value {
                return Err(DscError::Validation(
                    t!(
                        "configure.constraints.notMaxValue",
                        name = name,
                        max_value = max_value,
                        value = value
                    )
                    .to_string(),
                ));
            }
        } else {
            return Err(DscError::Validation(
                t!("configure.constraints.maxValueNotInteger", name = name).to_string(),
            ));
        }
    }

    Ok(())
}

/// Checks that the given value matches the given allowed values constraints.
///
/// # Arguments
///
/// * `name` - The name of the parameter.
/// * `value` - The value of the parameter.
/// * `constraint` - The constraints on the parameter.
///
/// # Returns
///
/// * `Ok(())` if the value matches the constraints.
/// * `Err(DscError::Validation)` if the value does not match the constraints.
///
/// # Errors
///
/// * `DscError::Validation` if the value does not match the constraints.
pub fn check_allowed_values(name: &str, value: &Value, constraint: &Parameter) -> Result<(), DscError> {
    if let Some(allowed_values) = &constraint.allowed_values {
        if value.is_string() && value.as_str().is_some() {
            let Some(value) = value.as_str() else {
                return Err(DscError::Validation(
                    t!("configure.constraints.allowedValuesIsNull", name = name).to_string(),
                ));
            };

            if !allowed_values.contains(&Value::String(value.to_string())) {
                return Err(DscError::Validation(
                    t!("configure.constraints.notAllowedValue", name = name).to_string(),
                ));
            }
        } else if value.is_i64() && value.as_i64().is_some() {
            let Some(value) = value.as_i64() else {
                return Err(DscError::Validation(
                    t!("configure.constraints.allowedValuesIsNull", name = name).to_string(),
                ));
            };

            if !allowed_values.contains(&Value::Number(value.into())) {
                return Err(DscError::Validation(
                    t!("configure.constraints.notAllowedValue", name = name).to_string(),
                ));
            }
        } else {
            return Err(DscError::Validation(
                t!("configure.constraints.allowedValuesNotStringOrInteger", name = name).to_string(),
            ));
        }
    }

    Ok(())
}

// TODO: check nullable
