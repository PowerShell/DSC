// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::Serialize;
use std::string::ToString;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OsInfo {
    #[serde(rename = "$id")]
    pub id: String,
    family: Family,
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
pub enum Bitness {
    #[serde(rename = "32")]
    Bit32,
    #[serde(rename = "64")]
    Bit64,
    #[serde(rename = "unknown")]
    Unknown,
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
        let edition = os_info.edition().map(ToString::to_string);
        let codename = os_info.codename().map(ToString::to_string);
        let architecture = os_info.architecture().map(ToString::to_string);
        let family = match os_info.os_type() {
            os_info::Type::Macos => Family::MacOS,
            os_info::Type::Windows => Family::Windows,
            _ => Family::Linux,
        };
        let bits: Bitness = match os_info.bitness() {
            os_info::Bitness::X32 => Bitness::Bit32,
            os_info::Bitness::X64 => Bitness::Bit64,
            _ => Bitness::Unknown,
        };
        Self {
            id: ID.to_string(),
            family,
            version: os_info.version().to_string(),
            edition,
            codename,
            bitness: bits,
            architecture,
        }
    }
}
