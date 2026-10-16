// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde::{Deserialize, Serialize};

/// How the `identity` list is applied by `set` and judged by `test`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SetAction {
    /// The listed accounts become the only holders of the right.
    #[default]
    Replace,
    /// The listed accounts are granted the right; other holders are kept.
    Add,
    /// The listed accounts lose the right; other holders are kept.
    Remove,
}

/// Desired or actual state of one user right (`Microsoft.Windows/UserRightsAssignment`).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UserRightsAssignment {
    /// The user right constant, e.g. `SeServiceLogonRight`. Matched case-insensitively and echoed as given.
    pub right: String,

    /// The accounts that hold the right. Required for `set` and `test`; an empty list means "No One".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<Vec<String>>,

    /// Write-only: how `identity` is applied. Never serialized.
    #[serde(default, skip_serializing)]
    pub set_action: SetAction,

    /// Read-only: whether the current holders satisfy `identity` and `set_action`. Set by `test`.
    #[serde(rename = "_inDesiredState", skip_serializing_if = "Option::is_none")]
    pub in_desired_state: Option<bool>,

    /// Metadata returned by what-if operations describing the changes that would be applied.
    #[serde(rename = "_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

/// Metadata returned by what-if operations.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// Human-readable descriptions of the changes that would be applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub what_if: Option<Vec<String>>,
}

impl Metadata {
    /// Wrap what-if messages, returning `None` when there is nothing to report.
    #[must_use]
    pub fn from_messages(messages: Vec<String>) -> Option<Self> {
        if messages.is_empty() {
            None
        } else {
            Some(Self { what_if: Some(messages) })
        }
    }
}

/// Errors from the security policy resource.
#[derive(Debug)]
pub enum SecurityPolicyError {
    /// The operation needs an elevated process.
    ElevationRequired,
    /// The input is well-formed JSON but not usable (unknown right, unknown account, missing property).
    InvalidInput(String),
    /// A Win32 or LSA operation failed.
    Policy(String),
}

impl std::fmt::Display for SecurityPolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ElevationRequired => f.write_str(&t!("main.elevationRequired")),
            Self::InvalidInput(message) | Self::Policy(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for SecurityPolicyError {}

impl From<String> for SecurityPolicyError {
    fn from(message: String) -> Self {
        Self::Policy(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_action_defaults_to_replace_and_uses_camel_case() {
        assert_eq!(SetAction::default(), SetAction::Replace);
        assert_eq!(serde_json::to_string(&SetAction::Replace).unwrap(), "\"replace\"");
        assert_eq!(serde_json::from_str::<SetAction>("\"add\"").unwrap(), SetAction::Add);
        assert_eq!(serde_json::from_str::<SetAction>("\"remove\"").unwrap(), SetAction::Remove);
        assert!(serde_json::from_str::<SetAction>("\"clobber\"").is_err());
    }

    #[test]
    fn input_requires_right_and_rejects_unknown_fields() {
        assert!(serde_json::from_str::<UserRightsAssignment>(r#"{"identity":[]}"#).is_err());
        assert!(serde_json::from_str::<UserRightsAssignment>(r#"{"right":"SeDebugPrivilege","bogus":1}"#).is_err());
        let input: UserRightsAssignment = serde_json::from_str(r#"{"right":"SeDebugPrivilege"}"#).unwrap();
        assert_eq!(input.right, "SeDebugPrivilege");
        assert!(input.identity.is_none());
        assert_eq!(input.set_action, SetAction::Replace);
    }

    #[test]
    fn set_action_is_read_but_never_written() {
        let input: UserRightsAssignment =
            serde_json::from_str(r#"{"right":"SeDebugPrivilege","identity":["Administrators"],"setAction":"add"}"#).unwrap();
        assert_eq!(input.set_action, SetAction::Add);
        let json = serde_json::to_string(&input).unwrap();
        assert_eq!(json, r#"{"right":"SeDebugPrivilege","identity":["Administrators"]}"#);
    }

    #[test]
    fn empty_identity_serializes_as_an_empty_array() {
        let state = UserRightsAssignment {
            right: "SeTcbPrivilege".to_string(),
            identity: Some(Vec::new()),
            ..UserRightsAssignment::default()
        };
        assert_eq!(serde_json::to_string(&state).unwrap(), r#"{"right":"SeTcbPrivilege","identity":[]}"#);
    }

    #[test]
    fn optional_outputs_only_serialize_when_present() {
        let mut state = UserRightsAssignment {
            right: "SeTcbPrivilege".to_string(),
            identity: Some(vec!["BUILTIN\\Administrators".to_string()]),
            ..UserRightsAssignment::default()
        };
        assert!(!serde_json::to_string(&state).unwrap().contains("_inDesiredState"));
        state.in_desired_state = Some(true);
        state.metadata = Metadata::from_messages(vec!["Would grant".to_string()]);
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains(r#""_inDesiredState":true"#));
        assert!(json.contains(r#""_metadata":{"whatIf":["Would grant"]}"#));
        assert!(Metadata::from_messages(Vec::new()).is_none());
    }
}
