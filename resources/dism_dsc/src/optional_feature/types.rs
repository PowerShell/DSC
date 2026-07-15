// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::util::{DismState, Filterable, matches_optional_exact, matches_optional_string};

pub type FeatureState = DismState;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionalFeatureList {
    #[serde(rename = "_restartRequired", skip_serializing_if = "Option::is_none")]
    pub restart_required_meta: Option<Vec<Map<String, Value>>>,
    pub features: Vec<OptionalFeatureInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptionalFeatureInfo {
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

impl Filterable for OptionalFeatureInfo {
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

    fn feature() -> OptionalFeatureInfo {
        OptionalFeatureInfo {
            feature_name: Some("TelnetClient".to_string()),
            exist: Some(true),
            state: Some(FeatureState::Installed),
            display_name: Some("Telnet Client".to_string()),
            description: Some("Telnet client feature".to_string()),
            restart_required: Some(RestartType::No),
        }
    }

    #[test]
    fn filter_matches_all_supported_fields() {
        let filter = OptionalFeatureInfo {
            feature_name: Some("telnetclient".to_string()),
            state: Some(FeatureState::Installed),
            display_name: Some("telnet client".to_string()),
            description: Some("telnet client feature".to_string()),
            ..Default::default()
        };

        assert!(feature().matches_filter(&filter));
    }

    #[test]
    fn filter_uses_exact_string_and_state_matching() {
        let feature = feature();

        assert!(!feature.matches_filter(&OptionalFeatureInfo {
            feature_name: Some("Telnet*".to_string()),
            ..Default::default()
        }));
        assert!(!feature.matches_filter(&OptionalFeatureInfo {
            state: Some(FeatureState::NotPresent),
            ..Default::default()
        }));
    }
}
