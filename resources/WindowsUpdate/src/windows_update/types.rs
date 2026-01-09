// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInput {
    pub title: Option<String>,
    pub id: Option<String>,
    pub is_installed: Option<bool>,
    pub description: Option<String>,
    pub is_uninstallable: Option<bool>,
    pub kb_article_ids: Option<Vec<String>>,
    pub max_download_size: Option<i64>,
    pub msrc_severity: Option<MsrcSeverity>,
    pub security_bulletin_ids: Option<Vec<String>>,
    pub update_type: Option<UpdateType>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub title: String,
    pub is_installed: bool,
    pub description: String,
    pub id: String,
    pub is_uninstallable: bool,
    pub kb_article_ids: Vec<String>,
    pub max_download_size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msrc_severity: Option<MsrcSeverity>,
    pub security_bulletin_ids: Vec<String>,
    pub update_type: UpdateType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum MsrcSeverity {
    Critical,
    Important,
    Moderate,
    Low,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
