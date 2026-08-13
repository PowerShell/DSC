// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types::{EnvironmentVariable, EnvironmentVariableList, PathAction, Scope};
use dsc_lib_registry::{RegistryHelper, config::RegistryValueData};
use dsc_lib_security_context::{SecurityContext, get_security_context};
use rust_i18n::t;
use std::collections::HashSet;

const CURRENT_USER_KEY: &str = r"HKCU\Environment";
const ALL_USERS_KEY: &str = r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment";

#[derive(Debug)]
pub enum EnvironmentError {
    ElevationRequired,
    Resource(String),
}

impl EnvironmentError {
    pub fn is_elevation_required(&self) -> bool {
        matches!(self, Self::ElevationRequired)
    }
}

impl std::fmt::Display for EnvironmentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ElevationRequired => formatter.write_str(&t!("set.elevationRequired")),
            Self::Resource(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for EnvironmentError {}

pub fn get_variables(
    input: &EnvironmentVariableList,
) -> Result<EnvironmentVariableList, EnvironmentError> {
    let environment_variables = input
        .environment_variables
        .iter()
        .map(get_variable)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(EnvironmentVariableList {
        environment_variables,
    })
}

pub fn set_variables(
    input: &EnvironmentVariableList,
) -> Result<EnvironmentVariableList, EnvironmentError> {
    if input
        .environment_variables
        .iter()
        .any(|variable| variable.scope == Scope::AllUsers)
        && get_security_context() != SecurityContext::Admin
    {
        return Err(EnvironmentError::ElevationRequired);
    }

    let mut environment_variables = Vec::with_capacity(input.environment_variables.len());
    for variable in &input.environment_variables {
        let helper = registry_helper(variable, None)?;
        if !variable.exist.unwrap_or(true) {
            helper
                .remove()
                .map_err(|error| operation_error("set.removeError", variable, &error))?;
            environment_variables.push(EnvironmentVariable {
                scope: variable.scope,
                name: variable.name.clone(),
                value: None,
                path_value: None,
                path_action: None,
                exist: Some(false),
            });
            continue;
        }

        let current_data = helper
            .get()
            .map_err(|error| operation_error("set.readError", variable, &error))?
            .value_data;
        let desired_value = desired_value(variable, current_data.as_ref());
        let value_data = registry_data(&desired_value, current_data.as_ref());
        registry_helper(variable, Some(value_data))?
            .set()
            .map_err(|error| operation_error("set.writeError", variable, &error))?;
        environment_variables.push(get_variable(variable)?);
    }

    Ok(EnvironmentVariableList {
        environment_variables,
    })
}

fn get_variable(variable: &EnvironmentVariable) -> Result<EnvironmentVariable, EnvironmentError> {
    let state = registry_helper(variable, None)?
        .get()
        .map_err(|error| operation_error("get.readError", variable, &error))?;

    if state.exist == Some(false) {
        return Ok(EnvironmentVariable {
            scope: variable.scope,
            name: variable.name.clone(),
            value: None,
            path_value: None,
            path_action: None,
            exist: Some(false),
        });
    }

    let value = match state.value_data {
        Some(RegistryValueData::String(value) | RegistryValueData::ExpandString(value)) => value,
        Some(_) => {
            return Err(EnvironmentError::Resource(
                t!(
                    "get.unsupportedType",
                    name = variable.name.as_str(),
                    scope = variable.scope.to_string()
                )
                .to_string(),
            ));
        }
        None => String::new(),
    };

    let (value, path_value) = if variable.path_value.is_some() {
        (None, Some(split_path(&value)))
    } else {
        (Some(value), None)
    };

    Ok(EnvironmentVariable {
        scope: variable.scope,
        name: variable.name.clone(),
        value,
        path_value,
        path_action: None,
        exist: Some(true),
    })
}

fn registry_helper(
    variable: &EnvironmentVariable,
    value_data: Option<RegistryValueData>,
) -> Result<RegistryHelper, EnvironmentError> {
    RegistryHelper::new(
        key_path(variable.scope),
        Some(variable.name.clone()),
        value_data,
    )
    .map_err(|error| operation_error("main.registryError", variable, &error))
}

fn key_path(scope: Scope) -> &'static str {
    match scope {
        Scope::AllUsers => ALL_USERS_KEY,
        Scope::CurrentUser => CURRENT_USER_KEY,
    }
}

fn desired_value(
    variable: &EnvironmentVariable,
    current_data: Option<&RegistryValueData>,
) -> String {
    if let Some(value) = &variable.value {
        return value.clone();
    }

    let desired = variable.path_value.as_deref().unwrap_or_default();
    let existing = match current_data {
        Some(RegistryValueData::String(value) | RegistryValueData::ExpandString(value)) => {
            split_path(value)
        }
        _ => Vec::new(),
    };

    merge_path(&existing, desired, variable.path_action.unwrap_or_default()).join(";")
}

fn registry_data(value: &str, current_data: Option<&RegistryValueData>) -> RegistryValueData {
    if matches!(current_data, Some(RegistryValueData::ExpandString(_))) || value.contains('%') {
        RegistryValueData::ExpandString(value.to_string())
    } else {
        RegistryValueData::String(value.to_string())
    }
}

fn split_path(value: &str) -> Vec<String> {
    value
        .split(';')
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect()
}

fn merge_path(existing: &[String], desired: &[String], action: PathAction) -> Vec<String> {
    let mut values = match action {
        PathAction::Prepend => desired.iter().chain(existing).cloned().collect::<Vec<_>>(),
        PathAction::Append => {
            let desired_keys = desired
                .iter()
                .map(|entry| entry.to_lowercase())
                .collect::<HashSet<_>>();
            existing
                .iter()
                .filter(|entry| !desired_keys.contains(&entry.to_lowercase()))
                .chain(desired)
                .cloned()
                .collect::<Vec<_>>()
        }
        PathAction::Clobber => desired.to_vec(),
    };

    let mut seen = HashSet::new();
    values.retain(|entry| seen.insert(entry.to_lowercase()));
    values
}

fn operation_error(
    key: &str,
    variable: &EnvironmentVariable,
    error: &impl std::fmt::Display,
) -> EnvironmentError {
    EnvironmentError::Resource(
        t!(
            key,
            name = variable.name.as_str(),
            scope = variable.scope.to_string(),
            error = error.to_string()
        )
        .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::{merge_path, split_path};
    use crate::types::PathAction;

    #[test]
    fn prepends_and_deduplicates_case_insensitively() {
        let existing = vec!["C:\\Existing".to_string(), "C:\\Shared".to_string()];
        let desired = vec!["c:\\shared".to_string(), "C:\\New".to_string()];

        assert_eq!(
            merge_path(&existing, &desired, PathAction::Prepend),
            vec!["c:\\shared", "C:\\New", "C:\\Existing"]
        );
    }

    #[test]
    fn appends_entries_at_the_end() {
        let existing = vec!["C:\\Shared".to_string(), "C:\\Existing".to_string()];
        let desired = vec!["c:\\shared".to_string(), "C:\\New".to_string()];

        assert_eq!(
            merge_path(&existing, &desired, PathAction::Append),
            vec!["C:\\Existing", "c:\\shared", "C:\\New"]
        );
    }

    #[test]
    fn clobber_deduplicates_desired_entries() {
        let desired = vec!["C:\\One".to_string(), "c:\\one".to_string()];

        assert_eq!(
            merge_path(&[], &desired, PathAction::Clobber),
            vec!["C:\\One"]
        );
    }

    #[test]
    fn splitting_omits_empty_path_segments() {
        assert_eq!(split_path("C:\\One;;C:\\Two;"), vec!["C:\\One", "C:\\Two"]);
    }
}
