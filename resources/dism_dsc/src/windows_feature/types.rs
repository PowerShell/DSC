// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::util::{DismState, Filterable, matches_optional_exact, matches_optional_string};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_all: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_paths: Option<Vec<String>>,
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

impl Filterable for WindowsFeatureInfo {
    fn matches_filter(&self, filter: &Self) -> bool {
        matches_optional_string(&self.feature_name, &filter.feature_name)
            && matches_optional_exact(&self.state, &filter.state)
            && matches_optional_string(&self.display_name, &filter.display_name)
            && matches_optional_string(&self.description, &filter.description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feature() -> WindowsFeatureInfo {
        WindowsFeatureInfo {
            feature_name: Some("Web-Server".to_string()),
            exist: Some(true),
            state: Some(FeatureState::Installed),
            display_name: Some("Web Server (IIS)".to_string()),
            description: Some("Web Server role".to_string()),
            restart_required: Some(RestartType::No),
            enable_all: None,
            source_paths: None,
            limit_access: None,
            metadata: None,
        }
    }

    #[test]
    fn filter_matches_all_supported_fields() {
        let filter = WindowsFeatureInfo {
            feature_name: Some("web-server".to_string()),
            state: Some(FeatureState::Installed),
            display_name: Some("web server (iis)".to_string()),
            description: Some("web server role".to_string()),
            ..Default::default()
        };

        assert!(feature().matches_filter(&filter));
    }

    #[test]
    fn filter_uses_exact_string_and_state_matching() {
        let feature = feature();

        assert!(!feature.matches_filter(&WindowsFeatureInfo {
            feature_name: Some("Web*".to_string()),
            ..Default::default()
        }));
        assert!(!feature.matches_filter(&WindowsFeatureInfo {
            state: Some(FeatureState::NotPresent),
            ..Default::default()
        }));
    }
}
