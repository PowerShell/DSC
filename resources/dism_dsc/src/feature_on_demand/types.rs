// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::util::{DismState, Filterable, matches_optional_exact, matches_optional_string};

pub type CapabilityState = DismState;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeatureOnDemandList {
    #[serde(rename = "_restartRequired", skip_serializing_if = "Option::is_none")]
    pub restart_required_meta: Option<Vec<Map<String, Value>>>,
    pub capabilities: Vec<FeatureOnDemandInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct FeatureOnDemandInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<String>,
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<CapabilityState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_size: Option<u32>,
}

impl Filterable for FeatureOnDemandInfo {
    fn matches_filter(&self, filter: &Self) -> bool {
        matches_optional_string(&self.identity, &filter.identity)
            && matches_optional_exact(&self.state, &filter.state)
            && matches_optional_string(&self.display_name, &filter.display_name)
            && matches_optional_string(&self.description, &filter.description)
            && matches_optional_exact(&self.download_size, &filter.download_size)
            && matches_optional_exact(&self.install_size, &filter.install_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capability() -> FeatureOnDemandInfo {
        FeatureOnDemandInfo {
            identity: Some("OpenSSH.Client~~~~0.0.1.0".to_string()),
            exist: Some(true),
            state: Some(CapabilityState::Installed),
            display_name: Some("OpenSSH Client".to_string()),
            description: Some("OpenSSH client capability".to_string()),
            download_size: Some(1_024),
            install_size: Some(2_048),
        }
    }

    #[test]
    fn filter_matches_all_supported_fields() {
        let filter = FeatureOnDemandInfo {
            identity: Some("openssh.client~~~~0.0.1.0".to_string()),
            state: Some(CapabilityState::Installed),
            display_name: Some("openssh client".to_string()),
            description: Some("openssh client capability".to_string()),
            download_size: Some(1_024),
            install_size: Some(2_048),
            ..Default::default()
        };

        assert!(capability().matches_filter(&filter));
    }

    #[test]
    fn filter_uses_exact_string_and_numeric_matching() {
        let capability = capability();

        assert!(!capability.matches_filter(&FeatureOnDemandInfo {
            identity: Some("OpenSSH*".to_string()),
            ..Default::default()
        }));
        assert!(!capability.matches_filter(&FeatureOnDemandInfo {
            download_size: Some(2_048),
            ..Default::default()
        }));
    }
}
