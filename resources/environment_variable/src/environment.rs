// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types::{
    EnvironmentPathVariable, EnvironmentPathVariableFilter, EnvironmentVariable,
    EnvironmentVariableFilter, EnvironmentVariableFilterItem, EnvironmentVariableFilterList,
    EnvironmentVariableItem, EnvironmentVariableList, Scope, SetAction,
};
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
        let state = registry_helper(variable.scope(), variable.name(), None)?
            .get()
            .map_err(|error| operation_error(OperationError::GetRead, variable, &error))?;
        let exists = state.exist != Some(false);
        let should_exist = variable.exist().unwrap_or(true);

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
        .any(|variable| variable.scope() == Scope::AllUsers)
        && get_security_context() != SecurityContext::Admin
    {
        return Err(EnvironmentError::ElevationRequired);
    }

    let mut environment_variables = Vec::with_capacity(input.environment_variables.len());
    for variable in &input.environment_variables {
        let helper = registry_helper(variable.scope(), variable.name(), None)?;
        if !variable.exist().unwrap_or(true) {
            helper
                .remove()
                .map_err(|error| operation_error(OperationError::SetRemove, variable, &error))?;
            environment_variables.push(missing_variable(variable));
            continue;
        }

        let current_data = helper
            .get()
            .map_err(|error| operation_error(OperationError::SetRead, variable, &error))?
            .value_data;
        let desired_value = desired_value(variable, current_data.as_ref());
        let value_data = registry_data(&desired_value, current_data.as_ref());
        registry_helper(
            variable.scope(),
            variable.name(),
            Some(value_data),
        )?
        .set()
        .map_err(|error| operation_error(OperationError::SetWrite, variable, &error))?;
        environment_variables.push(get_variable(variable)?);
    }

    Ok(EnvironmentVariableList {
        environment_variables,
        in_desired_state: None,
    })
}

pub fn export_variables(
    input: &EnvironmentVariableFilterList,
) -> Result<EnvironmentVariableList, EnvironmentError> {
    let mut environment_variables = Vec::new();

    for scope in [Scope::CurrentUser, Scope::AllUsers] {
        let helper = RegistryHelper::new(key_path(scope), None, None).map_err(|error| {
            EnvironmentError::Resource(
                t!(
                    "export.registryError",
                    scope = scope.to_string(),
                    error = error.to_string()
                )
                .to_string(),
            )
        })?;
        let values = helper.get_values().map_err(|error| {
            EnvironmentError::Resource(
                t!(
                    "export.readError",
                    scope = scope.to_string(),
                    error = error.to_string()
                )
                .to_string(),
            )
        })?;

        for (name, data) in values {
            if name.is_empty() {
                continue;
            }
            let value = match data {
                RegistryValueData::String(value) | RegistryValueData::ExpandString(value) => value,
                _ => continue,
            };

            if input.environment_variables.is_empty() {
                environment_variables.push(EnvironmentVariableItem::Scalar(
                    exported_scalar(scope, name, value),
                ));
                continue;
            }

            let matching_filter = input
                .environment_variables
                .iter()
                .filter(|filter| filter_matches(filter, scope, &name, &value))
                .find(|filter| matches!(filter, EnvironmentVariableFilterItem::Path(_)))
                .or_else(|| {
                    input
                        .environment_variables
                        .iter()
                        .find(|filter| filter_matches(filter, scope, &name, &value))
                });

            match matching_filter {
                Some(EnvironmentVariableFilterItem::Path(filter)) => {
                    let delimiter = filter.delimiter.as_deref().unwrap_or(";");
                    environment_variables.push(EnvironmentVariableItem::Path(
                        EnvironmentPathVariable {
                            scope,
                            name,
                            value: Some(split_path(&value, delimiter)),
                            delimiter: delimiter.to_string(),
                            set_action: SetAction::Clobber,
                            exist: Some(true),
                        },
                    ));
                }
                Some(EnvironmentVariableFilterItem::Scalar(_)) => {
                    environment_variables.push(EnvironmentVariableItem::Scalar(
                        exported_scalar(scope, name, value),
                    ));
                }
                None => {}
            }
        }
    }

    Ok(EnvironmentVariableList {
        environment_variables,
        in_desired_state: None,
    })
}

fn exported_scalar(scope: Scope, name: String, value: String) -> EnvironmentVariable {
    EnvironmentVariable {
        scope,
        name,
        value: Some(value),
        exist: Some(true),
    }
}

fn filter_matches(
    filter: &EnvironmentVariableFilterItem,
    scope: Scope,
    name: &str,
    value: &str,
) -> bool {
    match filter {
        EnvironmentVariableFilterItem::Scalar(filter) => {
            scalar_filter_matches(filter, scope, name, value)
        }
        EnvironmentVariableFilterItem::Path(filter) => {
            path_filter_matches(filter, scope, name, value)
        }
    }
}

fn scalar_filter_matches(
    filter: &EnvironmentVariableFilter,
    scope: Scope,
    name: &str,
    value: &str,
) -> bool {
    filter.scope.is_none_or(|expected| expected == scope)
        && filter
            .name
            .as_deref()
            .is_none_or(|pattern| matches_wildcard(name, pattern))
        && filter.value.as_deref().is_none_or(|expected| expected == value)
        && filter.exist.is_none_or(|expected| expected)
}

fn path_filter_matches(
    filter: &EnvironmentPathVariableFilter,
    scope: Scope,
    name: &str,
    value: &str,
) -> bool {
    let delimiter = filter.delimiter.as_deref().unwrap_or(";");
    filter.scope.is_none_or(|expected| expected == scope)
        && filter
            .name
            .as_deref()
            .is_none_or(|pattern| matches_wildcard(name, pattern))
        && filter.value.as_deref().is_none_or(|expected| {
            expected.is_empty() || path_values_equal(&split_path(value, delimiter), expected)
        })
        && filter.exist.is_none_or(|expected| expected)
}

fn matches_wildcard(text: &str, pattern: &str) -> bool {
    let text = text.to_lowercase();
    let pattern = pattern.to_lowercase();
    if !pattern.contains('*') {
        return text == pattern;
    }

    let parts = pattern.split('*').collect::<Vec<_>>();
    if !parts[0].is_empty() && !text.starts_with(parts[0]) {
        return false;
    }
    let mut position = parts[0].len();
    let suffix = parts.last().copied().unwrap_or_default();
    let end = if suffix.is_empty() {
        text.len()
    } else {
        if !text.ends_with(suffix) {
            return false;
        }
        text.len() - suffix.len()
    };
    for part in &parts[1..parts.len().saturating_sub(1)] {
        if part.is_empty() {
            continue;
        }
        let Some(index) = text.get(position..end).and_then(|text| text.find(part)) else {
            return false;
        };
        position += index + part.len();
    }
    position <= end
}

fn get_variable(
    variable: &EnvironmentVariableItem,
) -> Result<EnvironmentVariableItem, EnvironmentError> {
    let state = registry_helper(variable.scope(), variable.name(), None)?
        .get()
        .map_err(|error| operation_error(OperationError::GetRead, variable, &error))?;
    if state.exist == Some(false) {
        return Ok(missing_variable(variable));
    }

    let value = registry_string(variable, state.value_data.as_ref())?;
    Ok(match variable {
        EnvironmentVariableItem::Scalar(variable) => {
            EnvironmentVariableItem::Scalar(EnvironmentVariable {
                scope: variable.scope,
                name: variable.name.clone(),
                value: Some(value),
                exist: Some(true),
            })
        }
        EnvironmentVariableItem::Path(variable) => {
            EnvironmentVariableItem::Path(EnvironmentPathVariable {
                scope: variable.scope,
                name: variable.name.clone(),
                value: Some(split_path(&value, &variable.delimiter)),
                delimiter: variable.delimiter.clone(),
                set_action: SetAction::Clobber,
                exist: Some(true),
            })
        }
    })
}

fn missing_variable(variable: &EnvironmentVariableItem) -> EnvironmentVariableItem {
    match variable {
        EnvironmentVariableItem::Scalar(variable) => {
            EnvironmentVariableItem::Scalar(EnvironmentVariable {
                scope: variable.scope,
                name: variable.name.clone(),
                value: None,
                exist: Some(false),
            })
        }
        EnvironmentVariableItem::Path(variable) => {
            EnvironmentVariableItem::Path(EnvironmentPathVariable {
                scope: variable.scope,
                name: variable.name.clone(),
                value: None,
                delimiter: variable.delimiter.clone(),
                set_action: SetAction::Clobber,
                exist: Some(false),
            })
        }
    }
}

fn registry_helper(
    scope: Scope,
    name: &str,
    value_data: Option<RegistryValueData>,
) -> Result<RegistryHelper, EnvironmentError> {
    RegistryHelper::new(key_path(scope), Some(name.to_string()), value_data).map_err(|error| {
        EnvironmentError::Resource(
            t!(
                "main.registryError",
                name = name,
                scope = scope.to_string(),
                error = error.to_string()
            )
            .to_string(),
        )
    })
}

fn key_path(scope: Scope) -> &'static str {
    match scope {
        Scope::AllUsers => ALL_USERS_KEY,
        Scope::CurrentUser => CURRENT_USER_KEY,
    }
}

fn desired_value(
    variable: &EnvironmentVariableItem,
    current_data: Option<&RegistryValueData>,
) -> String {
    match variable {
        EnvironmentVariableItem::Scalar(variable) => {
            variable.value.clone().unwrap_or_default()
        }
        EnvironmentVariableItem::Path(variable) => {
            let desired = variable.value.as_deref().unwrap_or_default();
            let existing = match current_data {
                Some(RegistryValueData::String(value) | RegistryValueData::ExpandString(value)) => {
                    split_path(value, &variable.delimiter)
                }
                _ => Vec::new(),
            };
            merge_path(&existing, desired, variable.set_action).join(&variable.delimiter)
        }
    }
}

fn registry_data(value: &str, current_data: Option<&RegistryValueData>) -> RegistryValueData {
    if matches!(current_data, Some(RegistryValueData::ExpandString(_))) || value.contains('%') {
        RegistryValueData::ExpandString(value.to_string())
    } else {
        RegistryValueData::String(value.to_string())
    }
}

fn registry_string(
    variable: &EnvironmentVariableItem,
    current_data: Option<&RegistryValueData>,
) -> Result<String, EnvironmentError> {
    match current_data {
        Some(RegistryValueData::String(value) | RegistryValueData::ExpandString(value)) => {
            Ok(value.clone())
        }
        Some(_) => Err(EnvironmentError::Resource(
            t!(
                "get.unsupportedType",
                name = variable.name(),
                scope = variable.scope().to_string()
            )
            .to_string(),
        )),
        None => Ok(String::new()),
    }
}

fn value_in_desired_state(
    variable: &EnvironmentVariableItem,
    current_value: &str,
    current_data: Option<&RegistryValueData>,
) -> bool {
    match variable {
        EnvironmentVariableItem::Scalar(variable) => variable
            .value
            .as_deref()
            .is_some_and(|value| current_value == value),
        EnvironmentVariableItem::Path(variable) => {
            let projected = desired_value(
                &EnvironmentVariableItem::Path(variable.clone()),
                current_data,
            );
            path_values_equal(
                &split_path(current_value, &variable.delimiter),
                &split_path(&projected, &variable.delimiter),
            )
        }
    }
}

fn path_values_equal(left: &[String], right: &[String]) -> bool {
    left.iter()
        .map(|entry| entry.to_lowercase())
        .eq(right.iter().map(|entry| entry.to_lowercase()))
}

fn split_path(value: &str, delimiter: &str) -> Vec<String> {
    value
        .split(delimiter)
        .filter(|entry| !entry.is_empty())
        .map(str::to_string)
        .collect()
}

fn merge_path(existing: &[String], desired: &[String], action: SetAction) -> Vec<String> {
    let mut values = match action {
        SetAction::Prepend => desired.iter().chain(existing).cloned().collect::<Vec<_>>(),
        SetAction::Append => {
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
        SetAction::Clobber => desired.to_vec(),
    };

    let mut seen = HashSet::new();
    values.retain(|entry| seen.insert(entry.to_lowercase()));
    values
}

fn operation_error(
    operation: OperationError,
    variable: &EnvironmentVariableItem,
    error: &impl std::fmt::Display,
) -> EnvironmentError {
    let name = variable.name();
    let scope = variable.scope().to_string();
    let error = error.to_string();
    let message = match operation {
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
        CURRENT_USER_KEY, EnvironmentError, OperationError, export_variables, get_variables,
        key_path, matches_wildcard, merge_path, operation_error, path_filter_matches,
        scalar_filter_matches, set_variables, split_path, test_variables, value_in_desired_state,
    };
    use crate::types::{
        EnvironmentPathVariable, EnvironmentPathVariableFilter, EnvironmentVariable,
        EnvironmentVariableFilter, EnvironmentVariableFilterItem, EnvironmentVariableFilterList,
        EnvironmentVariableItem, EnvironmentVariableList, Scope, SetAction,
    };
    use dsc_lib_registry::{RegistryHelper, config::RegistryValueData};
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

    fn scalar(name: &str, value: Option<&str>) -> EnvironmentVariableItem {
        EnvironmentVariableItem::Scalar(EnvironmentVariable {
            scope: Scope::CurrentUser,
            name: name.to_string(),
            value: value.map(str::to_string),
            exist: None,
        })
    }

    fn path(name: &str, values: &[&str], action: SetAction) -> EnvironmentVariableItem {
        EnvironmentVariableItem::Path(EnvironmentPathVariable {
            scope: Scope::CurrentUser,
            name: name.to_string(),
            value: Some(values.iter().map(|value| (*value).to_string()).collect()),
            delimiter: ";".to_string(),
            set_action: action,
            exist: None,
        })
    }

    fn list(variable: EnvironmentVariableItem) -> EnvironmentVariableList {
        EnvironmentVariableList {
            environment_variables: vec![variable],
            in_desired_state: None,
        }
    }

    #[test]
    fn path_helpers_use_custom_delimiter_and_deduplicate() {
        assert_eq!(split_path("one::two::::three", "::"), vec!["one", "two", "three"]);
        assert_eq!(
            merge_path(
                &["One".to_string(), "Two".to_string()],
                &["one".to_string(), "Three".to_string()],
                SetAction::Append,
            ),
            vec!["Two", "one", "Three"]
        );
    }

    #[test]
    fn wildcard_matching_is_case_insensitive() {
        assert!(matches_wildcard("PSModulePath", "ps*path"));
        assert!(matches_wildcard("Path", "*"));
        assert!(!matches_wildcard("TEMP", "path*"));
    }

    #[test]
    fn scalar_filter_ands_specified_properties() {
        let filter = EnvironmentVariableFilter {
            scope: Some(Scope::CurrentUser),
            name: Some("DSC_*".to_string()),
            value: Some("expected".to_string()),
            exist: Some(true),
        };
        assert!(scalar_filter_matches(
            &filter,
            Scope::CurrentUser,
            "dsc_test",
            "expected"
        ));
        assert!(!scalar_filter_matches(
            &filter,
            Scope::CurrentUser,
            "dsc_test",
            "different"
        ));
    }

    #[test]
    fn path_filter_compares_split_values() {
        let filter = EnvironmentPathVariableFilter {
            scope: None,
            name: Some("*Path".to_string()),
            value: Some(vec!["C:\\One".to_string(), "C:\\Two".to_string()]),
            delimiter: None,
            _set_action: None,
            exist: None,
        };
        assert!(path_filter_matches(
            &filter,
            Scope::AllUsers,
            "Path",
            "c:\\one;C:\\TWO"
        ));
    }

    #[test]
    fn registry_operations_round_trip_scalar_and_removal() {
        let guard = RegistryValueGuard::new();
        let input = list(scalar(&guard.name, Some("expected")));

        let set = set_variables(&input).unwrap();
        let EnvironmentVariableItem::Scalar(set) = &set.environment_variables[0] else {
            panic!("expected scalar state");
        };
        assert_eq!(set.value.as_deref(), Some("expected"));

        let get = get_variables(&input).unwrap();
        let EnvironmentVariableItem::Scalar(get) = &get.environment_variables[0] else {
            panic!("expected scalar state");
        };
        assert_eq!(get.value.as_deref(), Some("expected"));
        assert_eq!(test_variables(&input).unwrap().in_desired_state, Some(true));

        assert_eq!(
            test_variables(&list(scalar(&guard.name, Some("different"))))
                .unwrap()
                .in_desired_state,
            Some(false)
        );

        let mut remove = scalar(&guard.name, None);
        let EnvironmentVariableItem::Scalar(variable) = &mut remove else {
            unreachable!();
        };
        variable.exist = Some(false);
        let removed = set_variables(&list(remove.clone())).unwrap();
        assert_eq!(removed.environment_variables[0].exist(), Some(false));
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
        let input = list(path(&guard.name, &[r"C:\New"], SetAction::Append));

        let set = set_variables(&input).unwrap();
        let EnvironmentVariableItem::Path(set) = &set.environment_variables[0] else {
            panic!("expected path state");
        };
        assert_eq!(
            set.value.as_deref(),
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
    fn exports_matching_registry_value_as_requested_shape() {
        let guard = RegistryValueGuard::new();
        guard.set(RegistryValueData::String("one::two".to_string()));
        let filters = EnvironmentVariableFilterList {
            environment_variables: vec![
                EnvironmentVariableFilterItem::Scalar(EnvironmentVariableFilter {
                    scope: Some(Scope::CurrentUser),
                    name: Some(guard.name.clone()),
                    value: None,
                    exist: None,
                }),
                EnvironmentVariableFilterItem::Path(EnvironmentPathVariableFilter {
                    scope: Some(Scope::CurrentUser),
                    name: Some(guard.name.clone()),
                    value: Some(Vec::new()),
                    delimiter: Some("::".to_string()),
                    _set_action: None,
                    exist: None,
                }),
            ],
        };

        let exported = export_variables(&filters).unwrap();
        assert_eq!(exported.environment_variables.len(), 1);
        let EnvironmentVariableItem::Path(variable) = &exported.environment_variables[0] else {
            panic!("path filter should control the exported shape");
        };
        assert_eq!(
            variable.value.as_deref(),
            Some(["one".to_string(), "two".to_string()].as_slice())
        );
    }

    #[test]
    fn unsupported_registry_type_returns_resource_error() {
        let guard = RegistryValueGuard::new();
        guard.set(RegistryValueData::DWord(42));
        let input = list(scalar(&guard.name, Some("expected")));

        assert!(
            get_variables(&input)
                .unwrap_err()
                .to_string()
                .contains(&guard.name)
        );
        assert!(
            test_variables(&input)
                .unwrap_err()
                .to_string()
                .contains(&guard.name)
        );
    }

    #[test]
    fn covers_path_projection_and_error_variants() {
        let prepend = path("Path", &[r"c:\shared", r"C:\New"], SetAction::Prepend);
        let current = RegistryValueData::String(r"c:\shared;C:\New;C:\Existing".to_string());
        assert!(value_in_desired_state(
            &prepend,
            r"c:\shared;C:\New;C:\Existing",
            Some(&current)
        ));
        assert!(!value_in_desired_state(
            &path("Path", &[r"C:\New"], SetAction::Prepend),
            r"C:\Existing",
            Some(&RegistryValueData::String(r"C:\Existing".to_string()))
        ));

        let variable = scalar("TestName", Some("value"));
        assert_eq!(key_path(Scope::CurrentUser), CURRENT_USER_KEY);
        assert!(key_path(Scope::AllUsers).starts_with("HKLM\\"));
        assert!(EnvironmentError::ElevationRequired.is_elevation_required());
        assert!(!EnvironmentError::Resource("message".to_string()).is_elevation_required());
        for operation in [
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
    fn merge_path_covers_all_actions() {
        let existing = vec![r"C:\Existing".to_string(), r"C:\Shared".to_string()];
        let desired = vec![r"c:\shared".to_string(), r"C:\New".to_string()];
        assert_eq!(
            merge_path(&existing, &desired, SetAction::Prepend),
            vec![r"c:\shared", r"C:\New", r"C:\Existing"]
        );
        assert_eq!(
            merge_path(&existing, &desired, SetAction::Append),
            vec![r"C:\Existing", r"c:\shared", r"C:\New"]
        );
        assert_eq!(
            merge_path(
                &[],
                &[r"C:\One".to_string(), r"c:\one".to_string()],
                SetAction::Clobber
            ),
            vec![r"C:\One"]
        );
    }
}
