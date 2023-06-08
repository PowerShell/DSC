use serde::{Deserialize, Serialize};

use crate::config::shared::{EnsureKind, GatewayPortsObject, IntObject, IgnoreRhostsObject, LogLevelObject,
    PermitRootLoginObject, PermitTunnelObject, PubkeyAuthOptionsObject, RepeatKeywordStringObject, StringObject, TCPFwdObject, YesNoObject};
/// An enum representing different arguments to Match
///
/// # Examples
///
/// ```
/// let user = MatchConditional::User;
/// let group = MatchConditional::Group;
///
/// assert_eq!(user, MatchConditional::User);
/// assert_eq!(group, MatchConditional::Group;
/// ```
///
/// # Variants
///
/// * `User`: match on the user's name
/// * `Group`: match on the user's group
/// * `Host`: match on the host machine name
/// * `LocalAddress`: match on the local address
/// * `LocalPort`: match on the local port
/// * `RDomain`: match on the rdomain on which connection was recevied
/// * `Address`: match on the address
/// * `All`: matches on all criteria
///
/// # Note
///
/// A Match conditional must include one of the Variants described above
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchConditional {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "group")]
    Group,
    #[serde(rename = "host")]
    Host,
    #[serde(rename = "localaddress")]
    LocalAddress,
    #[serde(rename = "localport")]
    LocalPort,
    #[serde(rename = "rdomain")]
    RDomain,
    #[serde(rename = "address")]
    Address,
    #[serde(rename = "all")]
    All,
}

/// A struct representing sshd_config keywords applicable inside a Match block
///
/// # Examples
///
/// ```
/// let match_data = MatchSubContainer { passwordAuthentication: "no", maxSessions: 18};
/// assert_eq!(match_data.password_authentication, "no");
/// assert_eq!(match_data.max_sessions, 18);
/// ```
///
/// # Fields
///
/// * Each keyword permitted inside a Match block in sshd_config is an optional field: https://man.openbsd.org/sshd_config.5
/// 
/// # Note
///
/// All keywords listed here are also applicable to SshdConfig struct
// TODO: is there a good way to reuse sshdconfig struct with only keywords that apply to match?
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MatchSubContainer {
    #[serde(rename = "acceptEnv", alias = "AcceptEnv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_env: Option<RepeatKeywordStringObject>,
    #[serde(rename = "allowAgentForwarding", alias = "AllowAgentForwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_agent_forwarding: Option<YesNoObject>,
    #[serde(rename = "allowGroups", alias = "AllowGroups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_groups: Option<StringObject>,
    #[serde(rename = "allowStreamLocalForwarding", alias = "AllowStreamLocalForwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_stream_local_forwarding: Option<TCPFwdObject>,
    #[serde(rename = "allowTcpForwarding", alias = "AllowTcpForwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_tcp_forwarding: Option<TCPFwdObject>,
    #[serde(rename = "allowUsers", alias = "AllowUsers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_users: Option<StringObject>,
    #[serde(rename = "authenticationMethods", alias = "AuthenticationMethods")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication_methods: Option<StringObject>,
    #[serde(rename = "authorizedKeysCommand", alias = "AuthorizedKeysCommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_command: Option<StringObject>,
    #[serde(rename = "authorizedKeysCommandUser", alias = "AuthorizedKeysCommandUser")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_command_user: Option<StringObject>,
    #[serde(rename = "authorizedKeysFile", alias = "AuthorizedKeysFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_file: Option<StringObject>,
    #[serde(rename = "authorizedPrincipalsCommand", alias = "AuthorizedPrincipalsCommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_principals_command: Option<StringObject>,
    #[serde(rename = "authorizedPrincipalsCommandUser", alias = "AuthorizedPrincipalsCommandUser")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_principals_command_user: Option<StringObject>,
    #[serde(rename = "authorizedPrincipalsFile", alias = "AuthorizedPrincipalsFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_principals_file: Option<StringObject>,
    #[serde(rename = "Banner", alias = "banner")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<StringObject>,
    #[serde(rename = "cASignatureAlgorithms", alias = "CASignatureAlgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_signature_algorithms: Option<StringObject>,
    #[serde(rename = "challengeresponseauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge_response_authentication: Option<YesNoObject>,
    #[serde(rename = "channelTimeout", alias = "ChannelTimeout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_timeout: Option<StringObject>,
    #[serde(rename = "chrootDirectory", alias = "ChrootDirectory")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chroot_directory: Option<StringObject>,
    #[serde(rename = "clientAliveCountMax", alias = "ClientAliveCountMax")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_alive_count_max: Option<IntObject>,
    #[serde(rename = "clientAliveInterval", alias = "ClientAliveInterval")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_alive_interval: Option<IntObject>,
    #[serde(rename = "denyGroups", alias = "DenyGroups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_groups: Option<StringObject>,
    #[serde(rename = "denyUsers", alias = "DenyUsers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_users: Option<StringObject>,
    #[serde(rename = "disableForwarding", alias = "DisableForwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_forwarding: Option<YesNoObject>,
    #[serde(rename = "exposeAuthInfo", alias = "ExposeAuthInfo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expose_auth_info: Option<YesNoObject>,
    #[serde(rename = "forceCommand", alias = "ForceCommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_command: Option<StringObject>,
    #[serde(rename = "gatewayPorts", alias = "GatewayPorts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_ports: Option<GatewayPortsObject>,
    #[serde(rename = "gssapiauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gss_authentication: Option<YesNoObject>,
    #[serde(rename = "hostbasedAcceptedAlgorithms", alias = "HostbasedAcceptedAlgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_accepted_algorithms: Option<StringObject>,
    #[serde(rename = "hostbasedacceptedkeytypes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_accepted_key_types: Option<StringObject>,
    #[serde(rename = "hostbasedAuthentication", alias = "HostbasedAuthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_authentication: Option<YesNoObject>,
    #[serde(rename = "hostbasedUsesNameFromPacketOnly", alias = "HostbasedUsesNameFromPacketOnly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_uses_name_from_packet_only: Option<YesNoObject>,
    #[serde(rename = "ignoreRhosts", alias = "IgnoreRhosts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_rhosts: Option<IgnoreRhostsObject>,
    #[serde(rename = "Include", alias = "include")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<StringObject>,
    #[serde(rename = "iPQoS", alias = "IPQoS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipq_o_s: Option<StringObject>,
    #[serde(rename = "kbdInteractiveAuthentication", alias = "KbdInteractiveAuthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kbd_interactive_authentication: Option<YesNoObject>,
    #[serde(rename = "kerberosAuthentication", alias = "KerberosAuthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_authentication: Option<YesNoObject>,
    #[serde(rename = "logLevel", alias = "LogLevel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<LogLevelObject>,
    #[serde(rename = "logVerbose", alias = "LogVerbose")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_verbose: Option<StringObject>,
    #[serde(rename = "maxAuthTries", alias = "MaxAuthTries")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_auth_tries: Option<IntObject>,
    #[serde(rename = "maxSessions", alias = "MaxSessions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_sessions: Option<IntObject>,
    #[serde(rename = "passwordAuthentication", alias = "PasswordAuthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_authentication: Option<YesNoObject>,
    #[serde(rename = "permitEmptyPasswords", alias = "PermitEmptyPasswords")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty_passwd: Option<StringObject>,
    #[serde(rename = "permitListen", alias = "PermitListen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_listen: Option<StringObject>,
    #[serde(rename = "permitOpen", alias = "PermitOpen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_open: Option<StringObject>,
    #[serde(rename = "permitRootLogin", alias = "PermitRootLogin")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_root_login: Option<PermitRootLoginObject>,
    #[serde(rename = "permitTTY", alias = "PermitTTY")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_tty: Option<YesNoObject>,
    #[serde(rename = "permitTunnel", alias = "PermitTunnel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_tunnel: Option<PermitTunnelObject>,
    #[serde(rename = "permitUserRC", alias = "PermitUserRC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_user_rc: Option<YesNoObject>,
    #[serde(rename = "pubkeyAcceptedAlgorithms", alias = "PubkeyAcceptedAlgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_accepted_algorithms: Option<StringObject>,
    #[serde(rename = "pubkeyacceptedkeytypes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_accepted_key_types: Option<StringObject>,
    #[serde(rename = "pubkeyAuthentication", alias = "PubkeyAuthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_authentication: Option<YesNoObject>,
    #[serde(rename = "pubkeyAuthOptions", alias = "PubkeyAuthOptions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_auth_options: Option<PubkeyAuthOptionsObject>,
    #[serde(rename = "rDomain", alias = "RDomain")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r_domain: Option<StringObject>,
    #[serde(rename = "rekeyLimit", alias = "RekeyLimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rekey_limit: Option<StringObject>,
    #[serde(rename = "requiredRSASize", alias = "RequiredRSASize")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_rsa_size: Option<IntObject>,
    #[serde(rename = "revokedKeys", alias = "RevokedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_keys: Option<StringObject>,
    #[serde(rename = "setEnv", alias = "SetEnv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_env: Option<StringObject>,
    #[serde(rename = "skeyauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skey_authentication: Option<StringObject>,
    #[serde(rename = "streamLocalBindMask", alias = "StreamLocalBindMask")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_local_bind_mask: Option<StringObject>,
    #[serde(rename = "streamLocalBindUnlink", alias = "StreamLocalBindUnlink")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_local_bind_unlink: Option<YesNoObject>,
    #[serde(rename = "trustedUserCAKeys", alias = "TrustedUserCAKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_user_ca_keys: Option<StringObject>,
    #[serde(rename = "unusedConnectionTimeout", alias = "UnusedConnectionTimeout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unused_connection_timeout: Option<StringObject>,
    #[serde(rename = "x11DisplayOffset", alias = "X11DisplayOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_display_offset: Option<IntObject>,
    #[serde(rename = "x11Forwarding", alias = "X11Forwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_forwarding: Option<YesNoObject>,
    #[serde(rename = "x11UseLocalhost", alias = "X11UseLocalhost")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_use_localhost: Option<YesNoObject>    
}

/// A struct representing a Match block within sshd_config
///
/// # Examples
///
/// ```
/// let match_block = MatchContainer { conditionalKey: "group", conditionalValue: "Administrators", data: {"passwordAuthentication": "no", "maxSessions": 18}};
/// assert_eq!(match_block.conditionalKey, "group");
/// assert_eq!(match_block.conditionalValue, Administrators);
/// assert_eq!(match_block.data.passwordAuthentication, "no");
/// assert_eq!(match_block.data.maxSessions, 18);
/// ```
///
/// # Fields
///
/// * `conditional_key`: the word after "Match" in sshd_config
/// * `conditional_value`: the last word in the Match line in sshd_config
/// * `data`: the lines following the Match conditional line that should override global settings when the criteria is met
/// * `ensure`: optional field, determines whether the Match block should be Present or Absent in sshd_config
/// 
///
/// # Note
///
/// sshd_config is order sensitive regarding Match
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MatchContainer {
    #[serde(rename = "conditionalKey")]
    pub conditional_key: MatchConditional,
    #[serde(rename = "conditionalValue")]
    pub conditional_value: String,
    pub data: MatchSubContainer,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
}
