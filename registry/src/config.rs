use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum EnsureKind {
    Present,
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum RegistryValueData {
    String(String),
    ExpandString(String),
    Binary(Vec<u8>),
    DWord(u32),
    MultiString(Vec<String>),
    QWord(u64),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "Registry")]
pub struct RegistryConfig {
    #[serde(rename = "$id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "keyPath")]
    pub key_path: String,
    #[serde(rename = "valueName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_name: Option<String>,
    #[serde(rename = "valueData")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_data: Option<RegistryValueData>,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
    #[serde(rename = "_clobber")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clobber: Option<bool>,
}

impl RegistryConfig {
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
