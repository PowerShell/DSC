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

#[derive(Clone, Copy)]
enum OperationError {
    Registry,
    GetRead,
    SetRead,
    SetWrite,
    SetRemove,
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
        in_desired_state: None,
    })
}

pub fn test_variables(
    input: &EnvironmentVariableList,
) -> Result<EnvironmentVariableList, EnvironmentError> {
    let mut in_desired_state = true;
    let mut environment_variables = Vec::with_capacity(input.environment_variables.len());

    for variable in &input.environment_variables {
        let state = registry_helper(variable, None)?
            .get()
            .map_err(|error| operation_error(OperationError::GetRead, variable, &error))?;
        let exists = state.exist != Some(false);
        let should_exist = variable.exist.unwrap_or(true);

        if exists != should_exist {
            in_desired_state = false;
        } else if exists {
            let current_data = state.value_data.as_ref();
            let current_value = registry_string(variable, current_data)?;
            if !value_in_desired_state(variable, &current_value, current_data) {
                in_desired_state = false;
            }
        }

        environment_variables.push(get_variable(variable)?);
    }

    Ok(EnvironmentVariableList {
        environment_variables,
        in_desired_state: Some(in_desired_state),
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
                .map_err(|error| operation_error(OperationError::SetRemove, variable, &error))?;
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
            .map_err(|error| operation_error(OperationError::SetRead, variable, &error))?
            .value_data;
        let desired_value = desired_value(variable, current_data.as_ref());
        let value_data = registry_data(&desired_value, current_data.as_ref());
        registry_helper(variable, Some(value_data))?
            .set()
            .map_err(|error| operation_error(OperationError::SetWrite, variable, &error))?;
        environment_variables.push(get_variable(variable)?);
    }

    Ok(EnvironmentVariableList {
        environment_variables,
        in_desired_state: None,
    })
}

fn get_variable(variable: &EnvironmentVariable) -> Result<EnvironmentVariable, EnvironmentError> {
    let state = registry_helper(variable, None)?
        .get()
        .map_err(|error| operation_error(OperationError::GetRead, variable, &error))?;

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
    .map_err(|error| operation_error(OperationError::Registry, variable, &error))
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

fn registry_string(
    variable: &EnvironmentVariable,
    current_data: Option<&RegistryValueData>,
) -> Result<String, EnvironmentError> {
    match current_data {
        Some(RegistryValueData::String(value) | RegistryValueData::ExpandString(value)) => {
            Ok(value.clone())
        }
        Some(_) => Err(EnvironmentError::Resource(
            t!(
                "get.unsupportedType",
                name = variable.name.as_str(),
                scope = variable.scope.to_string()
            )
            .to_string(),
        )),
        None => Ok(String::new()),
    }
}

fn value_in_desired_state(
    variable: &EnvironmentVariable,
    current_value: &str,
    current_data: Option<&RegistryValueData>,
) -> bool {
    if let Some(value) = &variable.value {
        return current_value == value;
    }

    let projected = desired_value(variable, current_data);
    split_path(current_value)
        .iter()
        .map(|entry| entry.to_lowercase())
        .eq(split_path(&projected)
            .iter()
            .map(|entry| entry.to_lowercase()))
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
    operation: OperationError,
    variable: &EnvironmentVariable,
    error: &impl std::fmt::Display,
) -> EnvironmentError {
    let name = variable.name.as_str();
    let scope = variable.scope.to_string();
    let error = error.to_string();
    let message = match operation {
        OperationError::Registry => t!(
            "main.registryError",
            name = name,
            scope = scope,
            error = error
        ),
        OperationError::GetRead => t!("get.readError", name = name, scope = scope, error = error),
        OperationError::SetRead => t!("set.readError", name = name, scope = scope, error = error),
        OperationError::SetWrite => t!("set.writeError", name = name, scope = scope, error = error),
        OperationError::SetRemove => {
            t!("set.removeError", name = name, scope = scope, error = error)
        }
    };
    EnvironmentError::Resource(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        CURRENT_USER_KEY, EnvironmentError, OperationError, get_variables, key_path, merge_path,
        operation_error, set_variables, split_path, test_variables, value_in_desired_state,
    };
    use crate::types::{EnvironmentVariable, EnvironmentVariableList, PathAction, Scope};
    use dsc_lib_registry::RegistryHelper;
    use dsc_lib_registry::config::RegistryValueData;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_ID: AtomicUsize = AtomicUsize::new(0);

    struct RegistryValueGuard {
        name: String,
    }

    impl RegistryValueGuard {
        fn new() -> Self {
            let id = TEST_ID.fetch_add(1, Ordering::Relaxed);
            Self {
                name: format!("DSC_Environment_RustTest_{}_{id}", std::process::id()),
            }
        }

        fn set(&self, data: RegistryValueData) {
            RegistryHelper::new(CURRENT_USER_KEY, Some(self.name.clone()), Some(data))
                .unwrap()
                .set()
                .unwrap();
        }
    }

    impl Drop for RegistryValueGuard {
        fn drop(&mut self) {
            RegistryHelper::new(CURRENT_USER_KEY, Some(self.name.clone()), None)
                .unwrap()
                .remove()
                .unwrap();
        }
    }

    fn variable(name: &str) -> EnvironmentVariable {
        EnvironmentVariable {
            scope: Scope::CurrentUser,
            name: name.to_string(),
            value: Some("expected".to_string()),
            path_value: None,
            path_action: None,
            exist: None,
        }
    }

    fn list(variable: EnvironmentVariable) -> EnvironmentVariableList {
        EnvironmentVariableList {
            environment_variables: vec![variable],
            in_desired_state: None,
        }
    }

    #[test]
    fn registry_operations_round_trip_scalar_and_removal() {
        let guard = RegistryValueGuard::new();
        let input = list(variable(&guard.name));

        let set = set_variables(&input).unwrap();
        assert_eq!(
            set.environment_variables[0].value.as_deref(),
            Some("expected")
        );

        let get = get_variables(&input).unwrap();
        assert_eq!(
            get.environment_variables[0].value.as_deref(),
            Some("expected")
        );

        let test = test_variables(&input).unwrap();
        assert_eq!(test.in_desired_state, Some(true));

        let mut different = variable(&guard.name);
        different.value = Some("different".to_string());
        assert_eq!(
            test_variables(&list(different)).unwrap().in_desired_state,
            Some(false)
        );

        let mut remove = variable(&guard.name);
        remove.value = None;
        remove.exist = Some(false);
        let removed = set_variables(&list(remove.clone())).unwrap();
        assert_eq!(removed.environment_variables[0].exist, Some(false));
        assert_eq!(
            test_variables(&list(remove)).unwrap().in_desired_state,
            Some(true)
        );
    }

    #[test]
    fn path_operations_preserve_expand_string_type() {
        let guard = RegistryValueGuard::new();
        guard.set(RegistryValueData::ExpandString(
            r"%SystemRoot%\Existing".to_string(),
        ));
        let input = list(EnvironmentVariable {
            scope: Scope::CurrentUser,
            name: guard.name.clone(),
            value: None,
            path_value: Some(vec![r"C:\New".to_string()]),
            path_action: Some(PathAction::Append),
            exist: None,
        });

        let set = set_variables(&input).unwrap();
        assert_eq!(
            set.environment_variables[0].path_value.as_deref(),
            Some([r"%SystemRoot%\Existing".to_string(), r"C:\New".to_string()].as_slice())
        );
        assert_eq!(test_variables(&input).unwrap().in_desired_state, Some(true));

        let stored = RegistryHelper::new(CURRENT_USER_KEY, Some(guard.name.clone()), None)
            .unwrap()
            .get()
            .unwrap();
        assert!(matches!(
            stored.value_data,
            Some(RegistryValueData::ExpandString(_))
        ));
    }

    #[test]
    fn unsupported_registry_type_returns_resource_error() {
        let guard = RegistryValueGuard::new();
        guard.set(RegistryValueData::DWord(42));
        let input = list(variable(&guard.name));

        let get_error = get_variables(&input).unwrap_err();
        assert!(get_error.to_string().contains(&guard.name));

        let test_error = test_variables(&input).unwrap_err();
        assert!(test_error.to_string().contains(&guard.name));
    }

    #[test]
    fn formats_error_variants_and_scope_paths() {
        let variable = variable("TestName");
        let elevation = EnvironmentError::ElevationRequired;
        assert!(elevation.is_elevation_required());
        assert!(!elevation.to_string().is_empty());

        let resource = EnvironmentError::Resource("message".to_string());
        assert!(!resource.is_elevation_required());
        assert_eq!(resource.to_string(), "message");

        assert_eq!(key_path(Scope::CurrentUser), CURRENT_USER_KEY);
        assert!(key_path(Scope::AllUsers).starts_with("HKLM\\"));

        for operation in [
            OperationError::Registry,
            OperationError::GetRead,
            OperationError::SetRead,
            OperationError::SetWrite,
            OperationError::SetRemove,
        ] {
            assert!(
                operation_error(operation, &variable, &"failure")
                    .to_string()
                    .contains("failure")
            );
        }
    }

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

    #[test]
    fn prepend_is_in_desired_state_after_projecting_same_value() {
        let variable = EnvironmentVariable {
            scope: Scope::CurrentUser,
            name: "Path".to_string(),
            value: None,
            path_value: Some(vec!["c:\\shared".to_string(), "C:\\New".to_string()]),
            path_action: Some(PathAction::Prepend),
            exist: None,
        };
        let current = RegistryValueData::String("c:\\shared;C:\\New;C:\\Existing".to_string());

        assert!(value_in_desired_state(
            &variable,
            "c:\\shared;C:\\New;C:\\Existing",
            Some(&current)
        ));
    }

    #[test]
    fn prepend_is_not_in_desired_state_before_projection() {
        let variable = EnvironmentVariable {
            scope: Scope::CurrentUser,
            name: "Path".to_string(),
            value: None,
            path_value: Some(vec!["C:\\New".to_string()]),
            path_action: Some(PathAction::Prepend),
            exist: None,
        };
        let current = RegistryValueData::String("C:\\Existing".to_string());

        assert!(!value_in_desired_state(
            &variable,
            "C:\\Existing",
            Some(&current)
        ));
    }
}
