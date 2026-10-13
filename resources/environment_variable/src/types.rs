// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Get,
    Set,
    Test,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    AllUsers,
    #[default]
    CurrentUser,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SetAction {
    Prepend,
    Append,
    #[default]
    Clobber,
}

fn default_delimiter() -> String {
    ";".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentVariable {
    #[serde(default)]
    pub scope: Scope,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentPathVariable {
    #[serde(default)]
    pub scope: Scope,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Vec<String>>,
    #[serde(default = "default_delimiter", skip_serializing)]
    pub delimiter: String,
    #[serde(default, skip_serializing)]
    pub set_action: SetAction,
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EnvironmentVariableItem {
    Scalar(EnvironmentVariable),
    Path(EnvironmentPathVariable),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentVariableList {
    #[serde(default)]
    pub environment_variables: Vec<EnvironmentVariableItem>,
    #[serde(rename = "_inDesiredState", skip_serializing_if = "Option::is_none")]
    pub in_desired_state: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentVariableFilter {
    pub scope: Option<Scope>,
    pub name: Option<String>,
    pub value: Option<String>,
    #[serde(rename = "_exist")]
    pub exist: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentPathVariableFilter {
    pub scope: Option<Scope>,
    pub name: Option<String>,
    pub value: Option<Vec<String>>,
    pub delimiter: Option<String>,
    #[serde(rename = "setAction")]
    pub _set_action: Option<SetAction>,
    #[serde(rename = "_exist")]
    pub exist: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum EnvironmentVariableFilterItem {
    Scalar(EnvironmentVariableFilter),
    Path(EnvironmentPathVariableFilter),
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentVariableFilterList {
    #[serde(default)]
    pub environment_variables: Vec<EnvironmentVariableFilterItem>,
}

impl EnvironmentVariableItem {
    pub fn scope(&self) -> Scope {
        match self {
            Self::Scalar(variable) => variable.scope,
            Self::Path(variable) => variable.scope,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Scalar(variable) => &variable.name,
            Self::Path(variable) => &variable.name,
        }
    }

    pub fn exist(&self) -> Option<bool> {
        match self {
            Self::Scalar(variable) => variable.exist,
            Self::Path(variable) => variable.exist,
        }
    }

    pub fn validate(&self, operation: Operation) -> Result<(), String> {
        match self {
            Self::Scalar(variable) => variable.validate(operation),
            Self::Path(variable) => variable.validate(operation),
        }
    }
}

impl From<EnvironmentVariable> for EnvironmentVariableList {
    fn from(variable: EnvironmentVariable) -> Self {
        Self {
            environment_variables: vec![EnvironmentVariableItem::Scalar(variable)],
            in_desired_state: None,
        }
    }
}

impl From<EnvironmentPathVariable> for EnvironmentVariableList {
    fn from(variable: EnvironmentPathVariable) -> Self {
        Self {
            environment_variables: vec![EnvironmentVariableItem::Path(variable)],
            in_desired_state: None,
        }
    }
}

impl EnvironmentVariableList {
    pub fn validate(&self, operation: Operation) -> Result<(), String> {
        if self.environment_variables.is_empty() {
            return Err(t!("validation.emptyList").to_string());
        }

        let mut identities = HashSet::new();
        for variable in &self.environment_variables {
            variable.validate(operation)?;
            let identity = (variable.scope(), variable.name().to_lowercase());
            if !identities.insert(identity) {
                return Err(t!(
                    "validation.duplicate",
                    name = variable.name(),
                    scope = variable.scope().to_string()
                )
                .to_string());
            }
        }

        Ok(())
    }
}

impl EnvironmentVariableFilterList {
    pub fn validate(&self) -> Result<(), String> {
        for filter in &self.environment_variables {
            let (name, delimiter, values) = match filter {
                EnvironmentVariableFilterItem::Scalar(filter) => {
                    (filter.name.as_deref(), None, None)
                }
                EnvironmentVariableFilterItem::Path(filter) => (
                    filter.name.as_deref(),
                    filter.delimiter.as_deref(),
                    filter.value.as_deref(),
                ),
            };
            if name.is_some_and(|name| name.contains('\0')) {
                return Err(
                    t!("validation.invalidName", name = name.unwrap_or_default()).to_string(),
                );
            }
            if let Some(delimiter) = delimiter
                && (delimiter.is_empty() || delimiter.contains('\0'))
            {
                return Err(t!(
                    "validation.invalidDelimiter",
                    name = name.unwrap_or("*")
                )
                .to_string());
            }
            if let Some(values) = values {
                let delimiter = delimiter.unwrap_or(";");
                if values.iter().any(|entry| {
                    entry.is_empty() || entry.contains(delimiter) || entry.contains('\0')
                }) {
                    return Err(t!(
                        "validation.invalidPathEntry",
                        name = name.unwrap_or("*"),
                        delimiter = delimiter
                    )
                    .to_string());
                }
            }
        }
        Ok(())
    }
}

fn validate_common(
    name: &str,
    value_is_some: bool,
    exist: Option<bool>,
    operation: Operation,
) -> Result<(), String> {
    if name.is_empty() {
        return Err(t!("validation.emptyName").to_string());
    }
    if name.contains('\0') {
        return Err(t!("validation.invalidName", name = name).to_string());
    }
    if matches!(operation, Operation::Set | Operation::Test)
        && exist.unwrap_or(true)
        && !value_is_some
    {
        return Err(t!("validation.missingValue", name = name).to_string());
    }
    Ok(())
}

impl EnvironmentVariable {
    pub fn validate(&self, operation: Operation) -> Result<(), String> {
        validate_common(
            &self.name,
            self.value.is_some(),
            self.exist,
            operation,
        )
    }
}

impl EnvironmentPathVariable {
    pub fn validate(&self, operation: Operation) -> Result<(), String> {
        validate_common(
            &self.name,
            self.value.is_some(),
            self.exist,
            operation,
        )?;
        if self.delimiter.is_empty() || self.delimiter.contains('\0') {
            return Err(t!(
                "validation.invalidDelimiter",
                name = self.name.as_str()
            )
            .to_string());
        }
        if let Some(entries) = &self.value
            && entries.iter().any(|entry| {
                entry.is_empty() || entry.contains(&self.delimiter) || entry.contains('\0')
            })
        {
            return Err(t!(
                "validation.invalidPathEntry",
                name = self.name.as_str(),
                delimiter = self.delimiter.as_str()
            )
            .to_string());
        }
        Ok(())
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllUsers => write!(formatter, "allUsers"),
            Self::CurrentUser => write!(formatter, "currentUser"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EnvironmentPathVariable, EnvironmentVariable, EnvironmentVariableItem,
        EnvironmentVariableList, Operation, Scope, SetAction,
    };

    fn scalar(name: &str) -> EnvironmentVariableItem {
        EnvironmentVariableItem::Scalar(EnvironmentVariable {
            scope: Scope::CurrentUser,
            name: name.to_string(),
            value: Some("value".to_string()),
            exist: None,
        })
    }

    #[test]
    fn rejects_duplicate_identity_across_variants() {
        let list = EnvironmentVariableList {
            environment_variables: vec![
                scalar("Test_Name"),
                EnvironmentVariableItem::Path(EnvironmentPathVariable {
                    scope: Scope::CurrentUser,
                    name: "TEST_NAME".to_string(),
                    value: Some(vec![r"C:\Path".to_string()]),
                    delimiter: ";".to_string(),
                    set_action: SetAction::Clobber,
                    exist: None,
                }),
            ],
            in_desired_state: None,
        };

        assert!(list.validate(Operation::Set).is_err());
    }

    #[test]
    fn allows_empty_export_filter() {
        let list = EnvironmentVariableList {
            environment_variables: Vec::new(),
            in_desired_state: None,
        };

        assert!(list.validate(Operation::Get).is_err());
    }

    #[test]
    fn rejects_invalid_path_delimiter_and_entries() {
        for (delimiter, value) in [
            ("", vec!["value"]),
            (";", vec!["C:\\One;C:\\Two"]),
            ("::", vec!["one::two"]),
        ] {
            let item = EnvironmentVariableItem::Path(EnvironmentPathVariable {
                scope: Scope::CurrentUser,
                name: "Path".to_string(),
                value: Some(value.into_iter().map(str::to_string).collect()),
                delimiter: delimiter.to_string(),
                set_action: SetAction::Clobber,
                exist: None,
            });
            assert!(item.validate(Operation::Set).is_err());
        }
    }

    #[test]
    fn untagged_items_select_value_shape() {
        let scalar: EnvironmentVariableItem =
            serde_json::from_str(r#"{"name":"One","value":"text"}"#).unwrap();
        let path: EnvironmentVariableItem =
            serde_json::from_str(r#"{"name":"Path","value":["C:\\One"]}"#).unwrap();

        assert!(matches!(scalar, EnvironmentVariableItem::Scalar(_)));
        assert!(matches!(path, EnvironmentVariableItem::Path(_)));
    }
}
