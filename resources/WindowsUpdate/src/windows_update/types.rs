// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateList {
    pub updates: Vec<UpdateInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_installed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_uninstallable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kb_article_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_download_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msrc_severity: Option<MsrcSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_bulletin_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_type: Option<UpdateType>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum MsrcSeverity {
    Critical,
    Important,
    Moderate,
    Low,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum UpdateType {
    Software,
    Driver,
}

impl std::fmt::Display for MsrcSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MsrcSeverity::Critical => write!(f, "Critical"),
            MsrcSeverity::Important => write!(f, "Important"),
            MsrcSeverity::Moderate => write!(f, "Moderate"),
            MsrcSeverity::Low => write!(f, "Low"),
        }
    }
}

impl std::fmt::Display for UpdateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateType::Software => write!(f, "Software"),
            UpdateType::Driver => write!(f, "Driver"),
        }
    }
}
