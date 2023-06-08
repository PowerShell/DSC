use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddressFamilyKeyword {
    #[serde(rename = "inet")]
    INet,
    #[serde(rename = "inet6")]
    INet6,
    #[serde(rename = "any")]
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AddressFamily {
    Object{
        value: AddressFamilyKeyword,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(AddressFamilyKeyword),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompressionKeyword {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
    #[serde(rename = "delayed")]
    Delayed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Compression {
    Object{
        value: CompressionKeyword,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(CompressionKeyword),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnsureKind {
    Present,
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FingerprintHashKeyword {
    #[serde(rename = "md5")]
    Md5,
    #[serde(rename = "sha256")]
    Sha256,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FingerprintHash {
    Object{
        value: FingerprintHashKeyword,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(FingerprintHashKeyword),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GatewayPortsKeyword {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
    #[serde(rename = "clientspecified")]
    ClientSpecified,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GatewayPorts {
    Object{
        value: GatewayPortsKeyword,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(GatewayPortsKeyword),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IgnoreRhostsKeyword {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
    #[serde(rename = "shosts-only")]
    SHostsOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IgnoreRhosts {
    Object{
        value: IgnoreRhostsKeyword,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(IgnoreRhostsKeyword),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Numeric {
    Object{
        value: u32, 
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>
    },
    Int(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LogLevelKeyword {
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
pub enum LogLevel {
    Object{
        value: LogLevelKeyword,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(LogLevelKeyword),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermitRootLoginKeyword {
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
pub enum PermitRootLogin {
    Object{
        value: PermitRootLoginKeyword,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(PermitRootLoginKeyword),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermitTunnelKeyword {
    #[serde(rename = "ethernet")]
    Ethernet,
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PermitTunnel {
    Object{
        value: PermitTunnelKeyword,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(PermitTunnelKeyword),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PubkeyAuthOptionsKeyword {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "touch-required")]
    TouchRequired,
    #[serde(rename = "verify-required")]
    VerifyRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PubkeyAuthOptions {
    Object{
        value: PubkeyAuthOptionsKeyword,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(PubkeyAuthOptionsKeyword),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepeatNumericKeyword {
    pub value: u32,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepeatTextKeyword {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub value: String,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SysLogFacilityKeyword {
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
pub enum SysLogFacility {
    Object{
        value: SysLogFacilityKeyword,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(SysLogFacilityKeyword),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TCPFwdKeyword {
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
pub enum TCPFwd {
    Object{
        value: TCPFwdKeyword,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    String(TCPFwdKeyword),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Text {
    Object{
        value: String, 
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>
    },
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum YesNoKeyword {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum YesNo {
    Object{
        value: YesNoKeyword,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    YesNo(YesNoKeyword),
}
