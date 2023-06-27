use chrono::Duration;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Error;

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
    pub type_keyword: ChannelTimeoutCombined,
    #[serde(deserialize_with = "parse_duration", serialize_with = "format_duration")]
    pub interval: Duration,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
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
    pub ipv4: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<String>,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
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
    pub host: String,
    pub port: PermitPort,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermitIpv4Keyword {
    pub ipv4: String,
    pub port: PermitPort,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermitIpv6Keyword {
    pub ipv6: String,
    pub port: PermitPort,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
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
    pub host: Option<String>,
    pub port: PermitPort,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
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

fn parse_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let input: &str = Deserialize::deserialize(deserializer)?;
    let mut number = String::new();
    let mut duration = Duration::seconds(0);

    for c in input.chars() {
        if c.is_numeric() {
            number.push(c);
        } else {
            if let Ok(parsed) = number.parse::<i64>() {
                match c {
                    's' | 'S' => { 
                        duration = duration + Duration::seconds(parsed);
                    },
                    'm' | 'M' => { 
                        duration = duration + Duration::minutes(parsed);
                    },
                    'h' | 'H' => { 
                        duration = duration + Duration::hours(parsed);
                    },
                    'd' | 'D' => { 
                        duration = duration + Duration::days(parsed);
                    },
                    'w' | 'W' => { 
                        duration = duration + Duration::weeks(parsed);
                    },
                    _ => { 
                        return Err(serde::de::Error::invalid_value(
                            de::Unexpected::Char(c),
                            &"Expected characters are: s, m, h, d, and w"
                        ));
                    }
                }
            } else {
                return Err(serde::de::Error::invalid_value(
                    de::Unexpected::Str(number.as_str()),
                    &"Expected an integer"
                ));
            }
            number = String::new();
        }
    }

    // parse after iterating, as no character input can also indicate seconds
    if !number.is_empty() {
        if let Ok(parsed) = number.parse::<i64>() {
            duration = duration + Duration::seconds(parsed);
        }
        else {
            return Err(serde::de::Error::invalid_value(
                de::Unexpected::Str(number.as_str()),
                &"Expected an integer"
            ));
        }
    }

    if duration.is_zero() {
        return Err(serde::de::Error::invalid_value(
            de::Unexpected::Str(input), 
            &"Expected a time interval"
        ));
    }

    Ok(duration)
}

fn format_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut duration = duration.clone();
    let mut duration_fmt = String::new();
    let weeks = duration.num_weeks();
    if weeks > 0 {
        duration = duration - Duration::weeks(weeks);
        duration_fmt.push_str(format!("{}w", weeks).as_str());
    }
    let days = duration.num_days();
    if days > 0 {
        duration = duration - Duration::days(days);
        duration_fmt.push_str(format!("{}d", days).as_str());
    }
    let hours = duration.num_hours();
    if hours > 0 {
        duration = duration - Duration::hours(hours);
        duration_fmt.push_str(format!("{}h", hours).as_str());
    }
    let mins = duration.num_minutes();
    if mins > 0 {
        duration = duration - Duration::minutes(mins);
        duration_fmt.push_str(format!("{}m", mins).as_str());
    }
    let seconds = duration.num_seconds();
    if seconds > 0 {
        duration_fmt.push_str(format!("{}s", seconds).as_str());
    }
    serializer.serialize_str(&duration_fmt)
}

#[test]
fn test_channel_timeout_interval_without_character() {
    let input_json: &str = r#"
    {
        "type": "agent-connection",
        "interval": "480"
    }
    "#;
    let channel_timeout: ChannelTimeout = serde_json::from_str(input_json).unwrap();
    assert_eq!(channel_timeout.interval, Duration::seconds(480));
    let channel_timeout_fmt = serde_json::to_string(&channel_timeout).unwrap();
    assert_eq!(channel_timeout_fmt, "{\"type\":\"agent-connection\",\"interval\":\"8m\"}")
}

#[test]
fn test_channel_timeout_interval_with_character() {
    let input_json: &str = r#"
    {
        "type": "agent-connection",
        "interval": "30m"
    }
    "#;
    let channel_timeout: ChannelTimeout = serde_json::from_str(input_json).unwrap();
    assert_eq!(channel_timeout.interval, Duration::minutes(30));
    let channel_timeout_fmt = serde_json::to_string(&channel_timeout).unwrap();
    assert_eq!(channel_timeout_fmt, "{\"type\":\"agent-connection\",\"interval\":\"30m\"}")
}

#[test]
fn test_channel_timeout_interval_with_characters() {
    let input_json: &str = r#"
    {
        "type": "agent-connection",
        "interval": "1W2d3H4m5S"
    }
    "#;
    let channel_timeout: ChannelTimeout = serde_json::from_str(input_json).unwrap();
    assert_eq!(
        channel_timeout.interval, 
        Duration::weeks(1) + Duration::days(2) + Duration::hours(3) + Duration::minutes(4) + Duration::seconds(5)
    );
    let channel_timeout_fmt = serde_json::to_string(&channel_timeout).unwrap();
    assert_eq!(channel_timeout_fmt, "{\"type\":\"agent-connection\",\"interval\":\"1w2d3h4m5s\"}")
}

#[test]
fn test_channel_timeout_invalid_character() {
    let input_json: &str = r#"
    {
        "type": "agent-connection",
        "interval": "z"
    }
    "#;
    let channel_timeout: Result<ChannelTimeout, Error> = serde_json::from_str(input_json);
    assert!(channel_timeout.is_err());
}

#[test]
fn test_channel_timeout_invalid_character_after_num() {
    let input_json: &str = r#"
    {
        "type": "agent-connection",
        "interval": "2x"
    }
    "#;
    let channel_timeout: Result<ChannelTimeout, Error> = serde_json::from_str(input_json);
    assert!(channel_timeout.is_err());
}

#[test]
fn test_channel_timeout_valid_character_no_num() {
    let input_json: &str = r#"
    {
        "type": "agent-connection",
        "interval": "2dh3s"
    }
    "#;
    let channel_timeout: Result<ChannelTimeout, Error> = serde_json::from_str(input_json);
    assert!(channel_timeout.is_err());
}

#[test]
fn test_channel_timeout_empty_input() {
    let input_json: &str = r#"
    {
        "type": "agent-connection",
        "interval": ""
    }
    "#;
    let channel_timeout: Result<ChannelTimeout, Error> = serde_json::from_str(input_json);
    assert!(channel_timeout.is_err());
}

#[test]
fn test_channel_timeout_decimal_input() {
    let input_json: &str = r#"
    {
        "type": "agent-connection",
        "interval": "10.5"
    }
    "#;
    let channel_timeout: Result<ChannelTimeout, Error> = serde_json::from_str(input_json);
    assert!(channel_timeout.is_err());
}

#[test]
fn test_channel_timeout_repeat_character() {
    let input_json: &str = r#"
    {
        "type": "agent-connection",
        "interval": "1h2h3m"
    }
    "#;
    let channel_timeout: ChannelTimeout = serde_json::from_str(input_json).unwrap();
    // sshd logic permits repeated (valid) characters
    assert_eq!(
        channel_timeout.interval, 
        Duration::hours(1) + Duration::hours(2) + Duration::minutes(3)
    );
    let channel_timeout_fmt = serde_json::to_string(&channel_timeout).unwrap();
    assert_eq!(channel_timeout_fmt, "{\"type\":\"agent-connection\",\"interval\":\"3h3m\"}")
}

#[test]
fn test_channel_timeout_negative_number() {
    let input_json: &str = r#"
    {
        "type": "agent-connection",
        "interval": "-2h2m"
    }
    "#;
    let channel_timeout: Result<ChannelTimeout, Error> = serde_json::from_str(input_json);
    assert!(channel_timeout.is_err());
}

#[test]
fn test_channel_timeout_zero_number() {
    let input_json: &str = r#"
    {
        "type": "agent-connection",
        "interval": "0h3m"
    }
    "#;
    let channel_timeout: ChannelTimeout = serde_json::from_str(input_json).unwrap();
    assert_eq!(
        channel_timeout.interval, 
        Duration::minutes(3)
    );
    let channel_timeout_fmt = serde_json::to_string(&channel_timeout).unwrap();
    assert_eq!(channel_timeout_fmt, "{\"type\":\"agent-connection\",\"interval\":\"3m\"}")
}
