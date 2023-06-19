use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Actions {
    /// Represents adding corresponding values to existing values
    #[serde(rename = "add")]
    Add,
    /// Represents removing corresponding values from existing values
    #[serde(rename = "remove")]
    Remove,
    /// Represents inserting corresponding values in front of existing values
    #[serde(rename = "insert")]
    Insert,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddRemove {
    /// Represents adding corresponding values to existing values
    #[serde(rename = "add")]
    Add,
    /// Represents removing corresponding values from existing values
    #[serde(rename = "remove")]
    Remove,
}

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
pub enum AnyNone {
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChannelTimeoutCombined {
    Keyword(ChannelTimeoutKeywords),
    SessionSubsystem(ChannelTimeoutSubsystem)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelTimeoutKeywords {
    /// Represents channels to the ssh-agent
    #[serde(rename = "agent-connection")]
    AgentConnection,
    /// Represents channels established via local or dynamic forwarding
    #[serde(rename = "direct-tcpip")]
    DirectTcpIp,
    /// Represents channels established via remote forwarding
    #[serde(rename = "forwarded-tcpip")]
    ForwardedTcpIp,
    /// Represents channels established for command execution sessions
    #[serde(rename = "session:command")]
    SessionCommand,
    /// Represents channels established for interactive shell sessions
    #[serde(rename = "session:shell")]
    SessionShell,
    /// Represents channels established for x11 forwarding
    #[serde(rename = "x11-connection")]
    X11Connection,
    /// Represents channels established for these session types: command, shell and subsystem
    #[serde(rename = "session:*")]
    WildcardSession,
    /// Represents both direct-tcpip and forwarded-tcpip sessions
    #[serde(rename = "*-tcpip")]
    WildcardTcpIp,
    /// Represents both agent-connection and x11-connection sessions
    #[serde(rename = "*-connection")]
    WildcardConnection,
    /// Represents all channel types
    #[serde(rename = "*")]
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelTimeoutSubsystem {
    pub subsystem: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelTimeout {
    #[serde(rename = "type")]
    type_keyword: ChannelTimeoutCombined,
    #[serde(skip_serializing_if = "Option::is_none")]
    weeks: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hours: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    minutes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seconds: Option<u32>,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ensure: Option<EnsureKind>,
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
pub enum IPQoSKeywords {
    /// Represents assured forwarding class, lowest service assurance
    #[serde(rename = "assuredForwardingLevel11", alias = "af11")]
    AssuredForwardingLevel11,
    /// Represents low-latency, lower service assurance compared to AF13 and AF14
    #[serde(rename = "assuredForwardingLevel12", alias = "af12")]
    AssuredForwardingLevel12,
    /// Represents low-latency, lower service assurance compared to AF14
    #[serde(rename = "assuredForwardingLevel13", alias = "af13")] 
    AssuredForwardingLevel13, 
    /// Represents low-latency; default value for interactive sessions
    #[serde(rename = "assuredForwardingLevel21", alias = "af21")]
    AssuredForwardingLevel21, 
    /// Represents lower service assurance compared to AF23
    #[serde(rename = "assuredForwardingLevel22", alias = "af22")]
    AssuredForwardingLevel22, 
    // Represents lower service assurance compared to AF31
    #[serde(rename = "assuredForwardingLevel23", alias = "af23")]
    AssuredForwardingLevel23, 
    /// Represents moderate level of service assurance
    #[serde(rename = "assuredForwardingLevel31", alias = "af31")]
    AssuredForwardingLevel31, 
    /// Represents moderate level of service assurance
    #[serde(rename = "assuredForwardingLevel32", alias = "af32")]
    AssuredForwardingLevel32, 
    /// Represents moderate level of service assurance
    #[serde(rename = "assuredForwardingLevel33", alias = "af33")]
    AssuredForwardingLevel33, 
    /// Represents high level of service assurance
    #[serde(rename = "assuredForwardingLevel41", alias = "af41")]
    AssuredForwardingLevel41,
    /// Represents high level of service assurance
    #[serde(rename = "assuredForwardingLevel42", alias = "af42")] 
    AssuredForwardingLevel42, 
    /// Represents highest level of service assurance from AF class
    #[serde(rename = "assuredForwardingLevel43", alias = "af43")]
    AssuredForwardingLevel43, 
    /// Represents lowest level of service quality of class selector values
    #[serde(rename = "classSelectorLevel0", alias = "cs0")]
    ClassSelectorLevel0, 
    /// Represents lower effort; default value effort for non-interactive sessions
    #[serde(rename = "classSelectorLevel1", alias = "cs1")]
    ClassSelectorLevel1, 
    /// Represents low level of service quality
    #[serde(rename = "classSelectorLevel2", alias = "cs2")]
    ClassSelectorLevel2, 
    /// Represents medium level of service quality
    #[serde(rename = "classSelectorLevel3", alias = "cs3")]
    ClassSelectorLevel3, 
    /// Represents medium level of service quality
    #[serde(rename = "classSelectorLevel4", alias = "cs4")]
    ClassSelectorLevel4, 
    /// Represents high level of service quality
    #[serde(rename = "classSelectorLevel5", alias = "cs5")]
    ClassSelectorLevel5, 
    /// Represents high level of service quality
    #[serde(rename = "classSelectorLevel6", alias = "cs6")]
    ClassSelectorLevel6, 
    #[serde(rename = "classSelectorLevel7", alias = "cs7")]
    /// Represents highest level of service quality of class selector values
    ClassSelectorLevel7, 
    #[serde(rename = "expeditedForwarding", alias = "ef")]
    /// Represents expedited forwarding class
    ExpeditedForwarding, 
    #[serde(rename = "lowerEffort", alias = "le")]
    /// Represents lower effort transport
    LowerEffort, 
    #[serde(rename = "lowDelay", alias = "lowdelay")]
    /// Represents a deprecated alias for EF class
    LowDelay, 
    #[serde(rename = "throughput")]
    /// Represents preference for high throughput
    Throughput, 
    /// Represents preference for reliable delivery
    #[serde(rename = "reliability")]
    Reliability,
    /// Represents preference to use OS default
    #[serde(rename = "none")]
    None,
}

/// Combining untagged enum (IPQoSKeywords) & tagged enum (Int(u32)) for parsing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IPQoSCombined {
    Keyword(IPQoSKeywords),
    Int(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IPQoS {
    Single{
        #[serde(rename = "allSessions")]
        all_sessions: IPQoSCombined,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>
    },
    Double{
        #[serde(rename = "interactiveSessions")]
        interactive_sessions: IPQoSCombined,
        #[serde(rename = "nonInteractiveSessions")]
        non_interactive_sessions: IPQoSCombined,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListenAddress {
    Hostname{
        hostname: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        address: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        port: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rdomain: Option<String>,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    IPv4{
        ipv4: String,
        port: u32,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Lists {
    pub action: Actions,
    pub values: Vec<String>,
    pub ensure: Option<EnsureKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ListsAddRemove {
    pub action: AddRemove,
    pub values: Vec<String>,
    pub ensure: Option<EnsureKind>,
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
pub struct MaxStartups {
    pub start: u32,
    pub rate: u32,
    pub full: u32,
    pub ensure: Option<EnsureKind>,
  }

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetBlockSize {
    ipv4: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ipv6: Option<String>,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ensure: Option<EnsureKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum None {
    #[serde(alias = "none")]
    None
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
pub struct PermitHostKeyword {
    host: String,
    port: PermitPort,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ensure: Option<EnsureKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermitIpv4Keyword {
    ipv4: String,
    port: PermitPort,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ensure: Option<EnsureKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermitIpv6Keyword {
    ipv6: String,
    port: PermitPort,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ensure: Option<EnsureKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PermitPort {
    Wildcard(Wildcard),
    Int(u32)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermitListenKeyword {
    #[serde(skip_serializing_if = "Option::is_none")]
    host: Option<String>,
    port: PermitPort,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    ensure: Option<EnsureKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PermitListen {
    Keyword(Vec<PermitListenKeyword>),
    AnyNone(AnyNone),
    AnyNoneEnsure {
        value: AnyNone,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PermitOpenKeyword {
    Host(PermitHostKeyword),
    Ipv4(PermitIpv4Keyword),
    Ipv6(PermitIpv6Keyword),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PermitOpen {
    Keyword(Vec<PermitOpenKeyword>),
    AnyNone(AnyNone),
    AnyNoneEnsure {
        value: AnyNone,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    }
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
#[serde(untagged)]
pub enum PerSourceMaxStartupsKeyword {
    /// Represents no limit on number of authenticated connections allowed from a given source address
    None(None),
    /// Represents limit on number of authenticated connections allowed from a given source address
    Int(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PerSourceMaxStartups {
    Object{
        value: PerSourceMaxStartupsKeyword,
        #[serde(rename = "_ensure")]
        #[serde(skip_serializing_if = "Option::is_none")]
        ensure: Option<EnsureKind>,
    },
    Line(PerSourceMaxStartupsKeyword),
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
    pub name: String,
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
pub enum Wildcard {
    #[serde(rename = "*")]
    Wildcard
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
