use serde::{Deserialize, Serialize};

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
pub enum FingerprintHash {
    #[serde(rename = "md5")]
    Md5,
    #[serde(rename = "sha256")]
    Sha256,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FingerprintHashObject {
    Object{
        value: FingerprintHash,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(FingerprintHash),
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
#[serde(untagged)]
pub enum IntObject {
    Object{
        value: i32, 
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>
    },
    Int(i32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LogLevel {
    QUIET,
    FATAL,
    ERROR,
    INFO,
    VERBOSE,
    DEBUG,
    DEBUG1,
    DEBUG2,
    DEBUG3
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LogLevelObject {
    Object{
        value: LogLevel,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(LogLevel),
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
pub enum PermitTunnel {
    #[serde(rename = "ethernet")]
    Ethernet,
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PermitTunnelObject {
    Object{
        value: PermitTunnel,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(PermitTunnel),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PubkeyAuthOptions {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "touch-required")]
    TouchRequired,
    #[serde(rename = "verify-required")]
    VerifyRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PubkeyAuthOptionsObject {
    Object{
        value: PubkeyAuthOptions,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(PubkeyAuthOptions),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepeatKeywordInt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub value: u32,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RepeatKeywordIntObject {
    Vec(Vec<RepeatKeywordInt>),
    Int(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepeatKeywordString {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub value: String,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RepeatKeywordStringObject {
    Vec(Vec<RepeatKeywordString>),
    String(String),
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
pub enum SysLogFacility {
    DAEMON, 
    USER, 
    AUTH, 
    LOCAL0, 
    LOCAL1, 
    LOCAL2, 
    LOCAL3, 
    LOCAL4, 
    LOCAL5, 
    LOCAL6, 
    LOCAL7
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SysLogFacilityObject {
    Object{
        value: SysLogFacility,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(SysLogFacility),
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
