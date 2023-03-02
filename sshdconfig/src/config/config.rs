use serde::{Deserialize, Serialize};

use crate::config::match_config::MatchData;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnsureKind {
    Present,
    Absent,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum YesNo {
    Yes,
    No,
    #[default]
    None
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepeatData {
    #[serde(rename = "name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "value")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
}

// single value, boolean, repeat, match
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SshdConfig {
    #[serde(rename = "passwordauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_authentication: Option<YesNo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<Vec<RepeatData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsystem: Option<Vec<RepeatData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syslogfacility: Option<String>,
    #[serde(rename = "match")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _match: Option<MatchData>,
    #[serde(rename = "_purge")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purge: Option<bool>, 
}

impl SshdConfig {
    pub fn to_json(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Failed to serialize to JSON: {}", e);
                String::new()
            }
        }
    }
}

