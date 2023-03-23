use os_info::{Type, Bitness};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OsInfo {
    #[serde(rename = "$id")]
    pub id: String,
    #[serde(rename = "type")]
    os_type: Type,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    edition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    codename: Option<String>,
    bitness: Bitness,
    #[serde(skip_serializing_if = "Option::is_none")]
    architecture: Option<String>,
}

const ID: &str = "https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json";

impl OsInfo {
    pub fn new() -> Self {
        let os_info = os_info::get();
        let edition = match os_info.edition() {
            None => None,
            Some(edition) => Some(edition.to_string()),
        };
        let codename = match os_info.codename() {
            None => None,
            Some(codename) => Some(codename.to_string()),
        };
        let architecture = match os_info.architecture() {
            None => None,
            Some(architecture) => Some(architecture.to_string()),
        };
        Self {
            id: ID.to_string(),
            os_type: os_info.os_type(),
            version: os_info.version().to_string(),
            edition: edition,
            codename: codename,
            bitness: os_info.bitness(),
            architecture: architecture,
        }
    }
}
