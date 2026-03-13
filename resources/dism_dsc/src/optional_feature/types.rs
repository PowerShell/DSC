// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionalFeatureList {
    pub features: Vec<OptionalFeatureInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct OptionalFeatureInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_name: Option<String>,
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
pub enum FeatureState {
    NotPresent,
    UninstallPending,
    Staged,
    Removed,
    Installed,
    InstallPending,
    Superseded,
    PartiallyInstalled,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum RestartType {
    No,
    Possible,
    Required,
}

impl fmt::Display for FeatureState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeatureState::NotPresent => write!(f, "NotPresent"),
            FeatureState::UninstallPending => write!(f, "UninstallPending"),
            FeatureState::Staged => write!(f, "Staged"),
            FeatureState::Removed => write!(f, "Removed"),
            FeatureState::Installed => write!(f, "Installed"),
            FeatureState::InstallPending => write!(f, "InstallPending"),
            FeatureState::Superseded => write!(f, "Superseded"),
            FeatureState::PartiallyInstalled => write!(f, "PartiallyInstalled"),
        }
    }
}

impl FeatureState {
    pub fn from_dism(state: i32) -> Option<Self> {
        match state {
            0 => Some(FeatureState::NotPresent),
            1 => Some(FeatureState::UninstallPending),
            2 => Some(FeatureState::Staged),
            3 => Some(FeatureState::Removed),
            4 => Some(FeatureState::Installed),
            5 => Some(FeatureState::InstallPending),
            6 => Some(FeatureState::Superseded),
            7 => Some(FeatureState::PartiallyInstalled),
            _ => None,
        }
    }
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
