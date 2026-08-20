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
pub enum PathAction {
    Prepend,
    Append,
    #[default]
    Clobber,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentVariableList {
    pub environment_variables: Vec<EnvironmentVariable>,
    #[serde(rename = "_inDesiredState", skip_serializing_if = "Option::is_none")]
    pub in_desired_state: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvironmentVariable {
    #[serde(default)]
    pub scope: Scope,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_value: Option<Vec<String>>,
    #[serde(default, skip_serializing)]
    pub path_action: Option<PathAction>,
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
}

impl EnvironmentVariableList {
    pub fn validate(&self, operation: Operation) -> Result<(), String> {
        if self.environment_variables.is_empty() {
            return Err(t!("validation.emptyList").to_string());
        }

        let mut identities = HashSet::new();
        for variable in &self.environment_variables {
            variable.validate(operation)?;
            let identity = (variable.scope, variable.name.to_lowercase());
            if !identities.insert(identity) {
                return Err(t!(
                    "validation.duplicate",
                    name = variable.name.as_str(),
                    scope = variable.scope.to_string()
                )
                .to_string());
            }
        }

        Ok(())
    }
}

impl EnvironmentVariable {
    fn validate(&self, operation: Operation) -> Result<(), String> {
        if self.name.is_empty() {
            return Err(t!("validation.emptyName").to_string());
        }
        if self.name.contains('\0') {
            return Err(t!("validation.invalidName", name = self.name.as_str()).to_string());
        }
        if self.value.is_some() && self.path_value.is_some() {
            return Err(t!("validation.valueConflict", name = self.name.as_str()).to_string());
        }
        if self.path_action.is_some() && self.path_value.is_none() {
            return Err(t!(
                "validation.pathActionWithoutValue",
                name = self.name.as_str()
            )
            .to_string());
        }
        if let Some(entries) = &self.path_value
            && entries
                .iter()
                .any(|entry| entry.is_empty() || entry.contains(';') || entry.contains('\0'))
        {
            return Err(t!("validation.invalidPathEntry", name = self.name.as_str()).to_string());
        }
        if matches!(operation, Operation::Set | Operation::Test)
            && self.exist.unwrap_or(true)
            && self.value.is_none()
            && self.path_value.is_none()
        {
            return Err(t!("validation.missingValue", name = self.name.as_str()).to_string());
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
    use super::{EnvironmentVariable, EnvironmentVariableList, Operation, PathAction, Scope};

    fn variable(name: &str) -> EnvironmentVariable {
        EnvironmentVariable {
            scope: Scope::CurrentUser,
            name: name.to_string(),
            value: Some("value".to_string()),
            path_value: None,
            path_action: None,
            exist: None,
        }
    }

    #[test]
    fn rejects_duplicate_identity_case_insensitively() {
        let mut second = variable("TEST_NAME");
        second.scope = Scope::CurrentUser;
        let list = EnvironmentVariableList {
            environment_variables: vec![variable("Test_Name"), second],
            in_desired_state: None,
        };

        assert!(list.validate(Operation::Set).is_err());
    }

    #[test]
    fn allows_same_name_in_different_scopes() {
        let mut second = variable("Test_Name");
        second.scope = Scope::AllUsers;
        let list = EnvironmentVariableList {
            environment_variables: vec![variable("Test_Name"), second],
            in_desired_state: None,
        };

        assert!(list.validate(Operation::Set).is_ok());
    }

    #[test]
    fn rejects_path_action_without_path_value() {
        let mut input = variable("Test_Name");
        input.path_action = Some(PathAction::Append);
        let list = EnvironmentVariableList {
            environment_variables: vec![input],
            in_desired_state: None,
        };

        assert!(list.validate(Operation::Set).is_err());
    }

    #[test]
    fn rejects_invalid_inputs() {
        let empty = EnvironmentVariableList {
            environment_variables: Vec::new(),
            in_desired_state: None,
        };
        assert!(empty.validate(Operation::Get).is_err());

        for name in ["", "invalid\0name"] {
            assert!(
                EnvironmentVariableList {
                    environment_variables: vec![variable(name)],
                    in_desired_state: None,
                }
                .validate(Operation::Get)
                .is_err()
            );
        }

        let mut conflicting = variable("Test_Name");
        conflicting.path_value = Some(vec!["C:\\Path".to_string()]);
        assert!(
            EnvironmentVariableList {
                environment_variables: vec![conflicting],
                in_desired_state: None,
            }
            .validate(Operation::Set)
            .is_err()
        );

        for entry in ["", "C:\\One;C:\\Two", "invalid\0path"] {
            let mut invalid_path = variable("Test_Name");
            invalid_path.value = None;
            invalid_path.path_value = Some(vec![entry.to_string()]);
            assert!(
                EnvironmentVariableList {
                    environment_variables: vec![invalid_path],
                    in_desired_state: None,
                }
                .validate(Operation::Set)
                .is_err()
            );
        }

        let mut missing_value = variable("Test_Name");
        missing_value.value = None;
        let list = EnvironmentVariableList {
            environment_variables: vec![missing_value],
            in_desired_state: None,
        };
        assert!(list.validate(Operation::Set).is_err());
        assert!(list.validate(Operation::Test).is_err());
        assert!(list.validate(Operation::Get).is_ok());
    }

    #[test]
    fn formats_scope_values_as_camel_case() {
        assert_eq!(Scope::AllUsers.to_string(), "allUsers");
        assert_eq!(Scope::CurrentUser.to_string(), "currentUser");
    }
}
