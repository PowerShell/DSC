use serde::{Deserialize, Serialize};

use crate::config::*;
use crate::sshdconfig_error::SshdConfigError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddressFamily {
    #[serde(rename = "inet")]
    INet,
    #[serde(rename = "inet6")]
    INet6,
    #[serde(rename = "any")]
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddressFamilyObject {
    Object{
        value: AddressFamily,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(AddressFamily),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Compression {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
    #[serde(rename = "delayed")]
    Delayed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompressionObject {
    Object{
        value: Compression,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(Compression),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnsureKind {
    Present,
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GatewayPorts {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
    #[serde(rename = "clientspecified")]
    ClientSpecified,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GatewayPortsObject {
    Object{
        value: GatewayPorts,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(GatewayPorts),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IgnoreRhosts {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
    #[serde(rename = "shosts-only")]
    SHostsOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IgnoreRhostsObject {
    Object{
        value: IgnoreRhosts,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(IgnoreRhosts),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermitRootLogin {
    #[serde(rename = "without-password")]
    WithoutPassword,
    #[serde(rename = "prohibit-password")]
    ProhibitPassword,
    #[serde(rename = "forced-commands-only")]
    ForcedCommandsOnly,
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PermitRootLoginObject {
    Object{
        value: PermitRootLogin,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(PermitRootLogin),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringObject {
    Object{
        value: String, 
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>
    },
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TCPFwd {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
    #[serde(rename = "all")]
    All,
    #[serde(rename = "remote")]
    Remote,
    #[serde(rename = "local")]
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TCPFwdObject {
    Object{
        value: TCPFwd,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(TCPFwd),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum YesNo {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum YesNoObject {
    Object{
        value: YesNo,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    YesNo(YesNo),
}
