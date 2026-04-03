// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum RuleDirection {
    Inbound,
    Outbound,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum RuleAction {
    Allow,
    Block,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FirewallRuleList {
    pub rules: Vec<FirewallRule>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FirewallRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// When `_exist` is `None` (omitted from JSON), the rule exists in the firewall store.
    /// When `_exist` is `Some(false)`, the rule was not found (get) or should be removed (set).
    /// This asymmetry is intentional: absence means "present" so existing rules produce clean
    /// output without a redundant `_exist: true` field.
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_ports: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_ports: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_addresses: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_addresses: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<RuleDirection>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<RuleAction>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub grouping: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface_types: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_traversal: Option<bool>,
}

impl FirewallRule {
    #[must_use]
    pub fn missing_from_input(&self) -> Self {
        let mut result = self.clone();
        result.exist = Some(false);
        result
    }

    #[must_use]
    pub fn selector_name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

#[derive(Debug)]
pub struct FirewallError {
    pub message: String,
}

impl std::fmt::Display for FirewallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FirewallError {}

impl From<String> for FirewallError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

#[cfg(windows)]
impl From<windows::core::Error> for FirewallError {
    fn from(error: windows::core::Error) -> Self {
        Self { message: error.to_string() }
    }
}
