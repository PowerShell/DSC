use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddressFamilyKeyword {
    /// Represents using IPv4 only
    #[serde(rename = "inet")]
    INet,
    /// Represents using IPv6 only
    #[serde(rename = "inet6")]
    INet6,
    /// Represents the default value
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
    /// Represents enabling compression; default value
    #[serde(rename = "yes")]
    Yes,
    /// Represents disabling compression
    #[serde(rename = "no")]
    No,
    /// Represents the legacy synonym for yes
    #[serde(rename = "delayed")]
    Delayed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnsureKind {
    Present,
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FingerprintHashKeyword {
    /// Represents using MD5 hash algorithm when logging key fingerprints
    #[serde(rename = "md5", alias = "MD5")]
    Md5,
    /// Represents using SHA256 hash algorithm when logging key fingerprints; default value
    #[serde(rename = "sha256", alias = "SHA256")]
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
    /// Represents forcing remote port forwardings to bind to the wildcard address
    #[serde(rename = "yes")]
    Yes,
    /// Represents forcing remote port forwardings to be available to the local host only; default value
    #[serde(rename = "no")]
    No,
    /// Represents allowing the client to select the address to which the forwarding is bound
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
    /// Represents ignoring all per-user files during HostbasedAuthentication; default value
    #[serde(rename = "yes")]
    Yes,
    /// Represents allowing use of both .shosts and .rhosts during HostbasedAuthentication
    #[serde(rename = "no")]
    No,
    /// Represents allowing use of .shorts during HostbasedAuthentication
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
    /// Represents the default value
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
    /// Represents a deprecated alias of prohibit-password
    #[serde(rename = "without-password")]
    WithoutPassword,
    /// Represents disabling password and keyboard-interactive authentication for root
    #[serde(rename = "prohibit-password")]
    ProhibitPassword,
    /// Represents allowing root login with public key authentication, but only if the command option is also specified
    #[serde(rename = "forced-commands-only")]
    ForcedCommandsOnly,
    /// Represents allowing root login using ssh
    #[serde(rename = "yes")]
    Yes,
    /// Represents not allowing root login using ssh
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
    /// Represents permitting tun device forwarding for layer 2
    #[serde(rename = "ethernet")]
    Ethernet,
    /// Represents permitting tun device forwarding for later 3
    #[serde(rename = "point-to-point")]
    PointToPoint,
    /// Represents permitting tun device forwarding for both point-to-point and ethernet
    #[serde(rename = "yes")]
    Yes,
    /// Represents not permitting tun device fowarding; default value
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
    /// Indicates no additional options are enabled; default value
    #[serde(rename = "none")]
    None,
    /// Represents requiring a signature to attest that a physically present user 
    /// explicitly confirmed the authentication for FIDO authenticator algorithms
    #[serde(rename = "touch-required")]
    TouchRequired,
    /// Represents requiring a FIDO key signature attesting that the user was verified, e.g. via a PIN
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
    /// Represents the default value 
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
    /// Represents permitting TCP forwarding; default value
    #[serde(rename = "yes")]
    Yes,
    /// Represents preventing all TCP forwarding
    #[serde(rename = "no")]
    No,
    /// Represents permitting all TCP forwarding
    #[serde(rename = "all")]
    All,
    /// Represents permitting only remote TCP forwarding, from the perspective of ssh
    #[serde(rename = "remote")]
    Remote,
    /// Represents permitting only local TCP forwarding, from the perspective of ssh
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
