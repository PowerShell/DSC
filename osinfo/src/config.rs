use os_info::{Type, Bitness};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OsInfo {
    #[serde(rename = "$id")]
    pub id: String,
    family: Family,
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Family {
    Linux,
    MacOS,
    Windows,
}

const ID: &str = "https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json";

impl OsInfo {
    pub fn new() -> Self {
        let os_info = os_info::get();
        let edition = os_info.edition().map(|edition| edition.to_string());
        let codename = os_info.codename().map(|codename| codename.to_string());
        let architecture = os_info.architecture().map(|architecture| architecture.to_string());
        let family = match os_info.os_type() {
            Type::Macos => Family::MacOS,
            Type::Windows => Family::Windows,
            _ => Family::Linux,
        };
        Self {
            id: ID.to_string(),
            family,
            os_type: os_info.os_type(),
            version: os_info.version().to_string(),
            edition,
            codename,
            bitness: os_info.bitness(),
            architecture,
        }
    }
}
