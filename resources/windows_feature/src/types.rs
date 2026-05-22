// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::util::{DismState, WildcardFilterable, matches_optional_exact, matches_optional_wildcard};

pub type FeatureState = DismState;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub what_if: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WindowsFeatureList {
    #[serde(rename = "_restartRequired", skip_serializing_if = "Option::is_none")]
    pub restart_required_meta: Option<Vec<Map<String, Value>>>,
    pub features: Vec<WindowsFeatureInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WindowsFeatureInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_name: Option<String>,
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<FeatureState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart_required: Option<RestartType>,
    /// Enable all features that the specified feature depends on, including child features.
    /// Passed as the `EnableAll` parameter to `DismEnableFeature`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_all: Option<bool>,
    /// Local source paths (e.g., a mounted Windows ISO or WIM) passed to `DismEnableFeature`
    /// as `SourcePaths`. Required on systems that cannot reach Windows Update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_paths: Option<Vec<String>>,
    /// When `true`, prevents DISM from contacting Windows Update even when `sourcePaths` is empty.
    /// Passed as the `LimitAccess` parameter to `DismEnableFeature`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_access: Option<bool>,
    #[serde(rename = "_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum RestartType {
    No,
    Possible,
    Required,
}

impl RestartType {
    pub fn from_dism(restart: i32) -> Option<Self> {
        match restart {
            0 => Some(RestartType::No),
            1 => Some(RestartType::Possible),
            2 => Some(RestartType::Required),
            _ => None,
        }
    }
}

impl WildcardFilterable for WindowsFeatureInfo {
    fn matches_filter(&self, filter: &Self) -> bool {
        matches_optional_wildcard(&self.feature_name, &filter.feature_name)
            && matches_optional_exact(&self.state, &filter.state)
            && matches_optional_wildcard(&self.display_name, &filter.display_name)
            && matches_optional_wildcard(&self.description, &filter.description)
    }
}
