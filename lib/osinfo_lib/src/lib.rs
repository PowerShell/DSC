// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::Serialize;
use std::fmt::Display;
use std::string::ToString;

/// Returns information about the operating system.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OsInfo {
    family: Family,
    /// Defines the version of the operating system as a string.
    version: String,
    /// Defines the Windows operating system edition, like `Windows 11` or `Windows Server 2016`.
    #[serde(skip_serializing_if = "Option::is_none")]
    edition: Option<String>,
    /// Defines the codename for the operating system as returned from `lsb_release --codename`.
    #[serde(skip_serializing_if = "Option::is_none")]
    codename: Option<String>,
    bitness: Bitness,
    /// Defines the processor architecture as reported by `uname -m` on the operating system.
    #[serde(skip_serializing_if = "Option::is_none")]
    architecture: Option<String>,
    #[serde(rename = "_name", skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

/// Defines whether the operating system is a 32-bit or 64-bit operating system.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Bitness {
    #[serde(rename = "32")]
    Bit32,
    #[serde(rename = "64")]
    Bit64,
    #[serde(rename = "unknown")]
    Unknown,
}

/// Defines whether the operating system is Linux, macOS, or Windows.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Family {
    Linux,
    #[serde(rename = "macOS")]
    MacOS,
    Windows,
}

impl Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Family::Linux => write!(f, "Linux"),
            Family::MacOS => write!(f, "macOS"),
            Family::Windows => write!(f, "Windows"),
        }
    }
}

impl OsInfo {
    pub fn new(include_name: bool) -> Self {
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
        let version = os_info.version().to_string();
        let name = if include_name {
            Some(
                match &architecture {
                    Some(arch) => format!("{family} {version} {arch}"),
                    None => format!("{family:?} {version}"),
                }
            )
        } else {
            None
        };
        Self {
            family,
            version,
            edition,
            codename,
            bitness: bits,
            architecture,
            name,
        }
    }
}
