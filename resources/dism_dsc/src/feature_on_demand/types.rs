// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::util::DismState;

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
