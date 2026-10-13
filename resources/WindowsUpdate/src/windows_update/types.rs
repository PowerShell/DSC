// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateList {
    #[serde(rename = "_restartRequired", skip_serializing_if = "Option::is_none")]
    pub restart_required: Option<Vec<Map<String, Value>>>,
    pub updates: Vec<UpdateInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    #[serde(rename = "whatIf", skip_serializing_if = "Option::is_none")]
    pub what_if: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    #[serde(rename = "_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installation_behavior: Option<InstallationBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_installed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_uninstallable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kb_article_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msrc_severity: Option<MsrcSeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_hard_disk_space: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_bulletin_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
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

/// Represents the installation behavior reboot options from Windows Update
/// These values indicate what reboot behavior can be expected from an update
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum InstallationBehavior {
    /// Never requires a reboot
    NeverReboots,
    /// Always requires a reboot
    AlwaysRequiresReboot,
    /// Can request a reboot
    CanRequestReboot,
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

#[cfg(windows)]
use windows::{
    core::*,
    Win32::System::UpdateAgent::*,
};

/// Extract complete update information from an IUpdate interface
#[cfg(windows)]
pub fn extract_update_info(update: &IUpdate) -> Result<UpdateInfo> {

    unsafe {
        let title = update.Title()?.to_string();
        let identity = update.Identity()?;
        let update_id = identity.UpdateID()?.to_string();
        let is_installed = update.IsInstalled()?.as_bool();
        let description = update.Description()?.to_string();
        let is_uninstallable = update.IsUninstallable()?.as_bool();

        // Get KB Article IDs
        let kb_articles = update.KBArticleIDs()?;
        let kb_count = kb_articles.Count()?;
        let mut kb_article_ids = Vec::new();
        for j in 0..kb_count {
            if let Ok(kb_str) = kb_articles.get_Item(j) {
                kb_article_ids.push(kb_str.to_string());
            }
        }

        // Get recommended hard disk space in MB
        let recommended_hard_disk_space = match update.RecommendedHardDiskSpace() {
            Ok(space) => space as i64,
            Err(_) => 0i64,
        };

        // Get MSRC Severity
        let msrc_severity = if let Ok(severity_str) = update.MsrcSeverity() {
            match severity_str.to_string().as_str() {
                "Critical" => Some(MsrcSeverity::Critical),
                "Important" => Some(MsrcSeverity::Important),
                "Moderate" => Some(MsrcSeverity::Moderate),
                "Low" => Some(MsrcSeverity::Low),
                _ => None,
            }
        } else {
            None
        };

        // Get Security Bulletin IDs
        let security_bulletins = update.SecurityBulletinIDs()?;
        let bulletin_count = security_bulletins.Count()?;
        let mut security_bulletin_ids = Vec::new();
        for j in 0..bulletin_count {
            if let Ok(bulletin_str) = security_bulletins.get_Item(j) {
                security_bulletin_ids.push(bulletin_str.to_string());
            }
        }

        // Determine update type using proper enum comparison
        let update_type = {
            use windows::Win32::System::UpdateAgent::UpdateType as WinUpdateType;
            let ut = update.Type()?;
            // utDriver = 2, utSoftware = 1
            if ut == WinUpdateType(2) {
                UpdateType::Driver
            } else {
                UpdateType::Software
            }
        };

        // Get installation behavior reboot setting
        let installation_behavior = if let Ok(behavior) = update.InstallationBehavior() {
            if let Ok(reboot_behavior) = behavior.RebootBehavior() {
                // InstallRebootBehavior values:
                // 0 = irbNeverReboots - Never requires reboot
                // 1 = irbAlwaysRequiresReboot - Always requires reboot
                // 2 = irbCanRequestReboot - Can request reboot
                match reboot_behavior.0 {
                    0 => Some(InstallationBehavior::NeverReboots),
                    1 => Some(InstallationBehavior::AlwaysRequiresReboot),
                    2 => Some(InstallationBehavior::CanRequestReboot),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(UpdateInfo {
            metadata: None,
            title: Some(title),
            is_installed: Some(is_installed),
            description: Some(description),
            id: Some(update_id),
            is_uninstallable: Some(is_uninstallable),
            kb_article_ids: Some(kb_article_ids),
            recommended_hard_disk_space: Some(recommended_hard_disk_space),
            msrc_severity,
            security_bulletin_ids: Some(security_bulletin_ids),
            update_type: Some(update_type),
            installation_behavior,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Metadata, UpdateInfo, UpdateList};

    #[test]
    fn metadata_serializes_with_what_if_field() {
        let metadata = Metadata {
            what_if: Some(vec!["Would install update 'Test'".to_string()]),
        };

        let json = serde_json::to_string(&metadata).expect("serialization should succeed");
        assert_eq!(json, r#"{"whatIf":["Would install update 'Test'"]}"#);
    }

    #[test]
    fn metadata_skips_empty_what_if_field() {
        let metadata = Metadata { what_if: None };

        let json = serde_json::to_string(&metadata).expect("serialization should succeed");
        assert_eq!(json, "{}");
    }

    #[test]
    fn update_info_serializes_metadata_as_underscore_metadata() {
        let info = UpdateInfo {
            metadata: Some(Metadata {
                what_if: Some(vec!["Would install update 'Test'".to_string()]),
            }),
            description: None,
            id: Some("some-id".to_string()),
            installation_behavior: None,
            is_installed: Some(true),
            is_uninstallable: None,
            kb_article_ids: None,
            msrc_severity: None,
            recommended_hard_disk_space: None,
            security_bulletin_ids: None,
            title: None,
            update_type: None,
        };

        let value = serde_json::to_value(&info).expect("serialization should succeed");
        assert_eq!(value["_metadata"]["whatIf"][0], "Would install update 'Test'");
        assert_eq!(value["id"], "some-id");
        assert_eq!(value["isInstalled"], true);
    }

    #[test]
    fn update_info_skips_metadata_when_none() {
        let info = UpdateInfo {
            metadata: None,
            description: None,
            id: Some("some-id".to_string()),
            installation_behavior: None,
            is_installed: None,
            is_uninstallable: None,
            kb_article_ids: None,
            msrc_severity: None,
            recommended_hard_disk_space: None,
            security_bulletin_ids: None,
            title: None,
            update_type: None,
        };

        let value = serde_json::to_value(&info).expect("serialization should succeed");
        assert!(value.get("_metadata").is_none());
    }

    #[test]
    fn update_list_round_trips_metadata() {
        let json = r#"{"updates":[{"id":"some-id","_metadata":{"whatIf":["message"]}}]}"#;

        let list: UpdateList = serde_json::from_str(json).expect("deserialization should succeed");
        let metadata = list.updates[0].metadata.as_ref().expect("metadata should be present");
        assert_eq!(metadata.what_if.as_deref(), Some(&["message".to_string()][..]));

        let round_tripped = serde_json::to_string(&list).expect("serialization should succeed");
        let reparsed: UpdateList = serde_json::from_str(&round_tripped).expect("round-trip should succeed");
        assert_eq!(reparsed.updates[0].id.as_deref(), Some("some-id"));
    }
}
