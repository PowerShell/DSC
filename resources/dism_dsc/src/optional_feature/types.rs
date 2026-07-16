// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::util::DismState;

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

