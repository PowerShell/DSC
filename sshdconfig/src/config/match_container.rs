use serde::{Deserialize, Serialize};

use crate::config::keywords::{ChannelTimeout, EnsureKind, GatewayPorts, IgnoreRhosts, IPQoS, Lists, ListsSubset, LogLevel, 
    Numeric, PermitListen, PermitOpen, PermitRootLogin, PermitTunnel, PubkeyAuthOptions, Text, TCPFwd, YesNo};

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
    #[serde(rename = "acceptEnv", alias = "AcceptEnv", alias = "acceptenv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_env: Option<Vec<Text>>,

    #[serde(rename = "allowAgentForwarding", alias = "AllowAgentForwarding", alias = "allowagentforwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_agent_forwarding: Option<YesNo>,

    #[serde(rename = "allowGroups", alias = "AllowGroups", alias = "allowgroups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_groups: Option<Vec<Text>>,

    #[serde(rename = "allowStreamLocalForwarding", alias = "AllowStreamLocalForwarding", alias = "allowstreamlocalforwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_stream_local_forwarding: Option<TCPFwd>,

    #[serde(rename = "allowTcpForwarding", alias = "AllowTcpForwarding", alias = "allowtcpforwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_tcp_forwarding: Option<TCPFwd>,

    #[serde(rename = "allowUsers", alias = "AllowUsers", alias = "allowusers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_users: Option<Vec<Text>>,

    #[serde(rename = "authenticationMethods", alias = "AuthenticationMethods", alias = "authenticationmethods")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication_methods: Option<Text>,

    #[serde(rename = "authorizedKeysCommand", alias = "AuthorizedKeysCommand", alias = "authorizedkeyscommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_command: Option<Text>,

    #[serde(rename = "authorizedKeysCommandUser", alias = "AuthorizedKeysCommandUser", alias = "authorizedkeyscommanduser")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_command_user: Option<Text>,

    #[serde(rename = "authorizedKeysFile", alias = "AuthorizedKeysFile", alias = "authorizedkeysfile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_file: Option<Vec<Text>>,

    #[serde(rename = "authorizedPrincipalsCommand", alias = "AuthorizedPrincipalsCommand", alias = "authorizedprincipalscommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_principals_command: Option<Text>,

    #[serde(rename = "authorizedPrincipalsCommandUser", alias = "AuthorizedPrincipalsCommandUser", alias = "authorizedprincipalscommanduser")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_principals_command_user: Option<Text>,

    #[serde(rename = "authorizedPrincipalsFile", alias = "AuthorizedPrincipalsFile", alias = "authorizedprincipalsfile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_principals_file: Option<Text>,

    #[serde(alias = "Banner")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<Text>,

    #[serde(rename = "caSignatureAlgorithms", alias = "CASignatureAlgorithms", alias = "casignaturealgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_signature_algorithms: Option<ListsSubset>,

    #[serde(rename = "challengeResponseAuthentication", alias = "ChallengeResponseAuthentication", alias = "challengeresponseauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge_response_authentication: Option<YesNo>,

    #[serde(rename = "channelTimeout", alias = "ChannelTimeout", alias = "channeltimeout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_timeout: Option<Vec<ChannelTimeout>>,

    #[serde(rename = "chrootDirectory", alias = "ChrootDirectory", alias = "chrootdirectory")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chroot_directory: Option<Text>,

    #[serde(rename = "clientAliveCountMax", alias = "ClientAliveCountMax", alias = "clientalivecountmax")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_alive_count_max: Option<Numeric>,

    #[serde(rename = "clientAliveInterval", alias = "ClientAliveInterval", alias = "clientaliveinterval")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_alive_interval: Option<Numeric>,

    #[serde(rename = "denyGroups", alias = "DenyGroups", alias = "denygroups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_groups: Option<Vec<Text>>,

    #[serde(rename = "denyUsers", alias = "DenyUsers", alias = "denyusers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_users: Option<Vec<Text>>,

    #[serde(rename = "disableForwarding", alias = "DisableForwarding", alias = "disableforwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_forwarding: Option<YesNo>,

    #[serde(rename = "exposeAuthInfo", alias = "ExposeAuthInfo", alias = "exposeauthinfo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expose_auth_info: Option<YesNo>,

    #[serde(rename = "forceCommand", alias = "ForceCommand", alias = "forcecommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_command: Option<Text>,

    #[serde(rename = "gatewayPorts", alias = "GatewayPorts", alias = "gatewayports")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_ports: Option<GatewayPorts>,

    #[serde(rename = "gssApiAuthentication", alias = "GSSApiAuthentication", alias = "gssapiauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gss_authentication: Option<YesNo>,

    #[serde(rename = "hostbasedAcceptedAlgorithms", alias = "HostbasedAcceptedAlgorithms", alias = "hostbasedacceptedalgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_accepted_algorithms: Option<Lists>,

    #[serde(rename = "hostbasedacceptedkeytypes", alias = "HostbasedAcceptedKeyTypes", alias = "hostbasedacceptedkeytypes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_accepted_key_types: Option<Text>,

    #[serde(rename = "hostbasedAuthentication", alias = "HostbasedAuthentication", alias = "hostbasedauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_authentication: Option<YesNo>,

    #[serde(rename = "hostbasedUsesNameFromPacketOnly", alias = "HostbasedUsesNameFromPacketOnly", alias = "hostbasedusesnamefrompacketonly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_uses_name_from_packet_only: Option<YesNo>,

    #[serde(rename = "ignoreRhosts", alias = "IgnoreRhosts", alias = "ignorerhosts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_rhosts: Option<IgnoreRhosts>,

    #[serde(rename = "Include")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Text>,

    #[serde(rename = "iPQoS", alias = "IPQoS", alias = "ipqos")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipq_o_s: Option<IPQoS>,

    #[serde(rename = "kbdInteractiveAuthentication", alias = "KbdInteractiveAuthentication", alias = "kbdinteractiveauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kbd_interactive_authentication: Option<YesNo>,

    #[serde(rename = "kerberosAuthentication", alias = "KerberosAuthentication", alias = "kerberosauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_authentication: Option<YesNo>,

    #[serde(rename = "logLevel", alias = "LogLevel", alias = "loglevel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<LogLevel>,

    #[serde(rename = "logVerbose", alias = "LogVerbose", alias = "logverbose")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_verbose: Option<Text>,

    #[serde(rename = "maxAuthTries", alias = "MaxAuthTries", alias = "maxauthtries")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_auth_tries: Option<Numeric>,

    #[serde(rename = "maxSessions", alias = "MaxSessions", alias = "maxsessions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_sessions: Option<Numeric>,

    #[serde(rename = "passwordAuthentication", alias = "PasswordAuthentication", alias = "passwordauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_authentication: Option<YesNo>,

    #[serde(rename = "permitEmptyPasswords", alias = "PermitEmptyPasswords", alias = "permitemptypasswords")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty_passwd: Option<Text>,

    #[serde(rename = "permitListen", alias = "PermitListen", alias = "permitlisten")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_listen: Option<PermitListen>,

    #[serde(rename = "permitOpen", alias = "PermitOpen", alias = "permitopen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_open: Option<PermitOpen>,

    #[serde(rename = "permitRootLogin", alias = "PermitRootLogin", alias = "permitrootlogin")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_root_login: Option<PermitRootLogin>,

    #[serde(rename = "permitTTY", alias = "PermitTTY", alias = "permittty")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_tty: Option<YesNo>,

    #[serde(rename = "permitTunnel", alias = "PermitTunnel", alias = "permittunnel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_tunnel: Option<PermitTunnel>,

    #[serde(rename = "permitUserRC", alias = "PermitUserRC", alias = "permituserrc")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_user_rc: Option<YesNo>,

    #[serde(rename = "pubkeyAcceptedAlgorithms", alias = "PubkeyAcceptedAlgorithms", alias = "pubkeyacceptedalgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_accepted_algorithms: Option<Lists>,
    
    #[serde(rename = "pubkeyAcceptedKeyTypes", alias = "PubkeyAcceptedKeyTypes", alias = "pubkeyacceptedkeytypes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_accepted_key_types: Option<Text>,

    #[serde(rename = "pubkeyAuthentication", alias = "PubkeyAuthentication", alias = "pubkeyauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_authentication: Option<YesNo>,

    #[serde(rename = "pubkeyAuthOptions", alias = "PubkeyAuthOptions", alias = "pubkeyauthoptions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_auth_options: Option<PubkeyAuthOptions>,

    #[serde(rename = "rDomain", alias = "RDomain", alias = "rdomain")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r_domain: Option<Text>,

    #[serde(rename = "rekeyLimit", alias = "RekeyLimit", alias = "rekeylimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rekey_limit: Option<Text>,

    #[serde(rename = "requiredRSASize", alias = "RequiredRSASize", alias = "requiredrsasize")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_rsa_size: Option<Numeric>,

    #[serde(rename = "revokedKeys", alias = "RevokedKeys", alias = "revokedkeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_keys: Option<Text>,

    #[serde(rename = "setEnv", alias = "SetEnv", alias = "setenv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_env: Option<Vec<Text>>,

    #[serde(rename = "sKeyAuthentication", alias = "skeyauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skey_authentication: Option<Text>,

    #[serde(rename = "streamLocalBindMask", alias = "StreamLocalBindMask", alias = "streamlocalbindmask")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_local_bind_mask: Option<Text>,

    #[serde(rename = "streamLocalBindUnlink", alias = "StreamLocalBindUnlink", alias = "streamlocalbindunlink")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_local_bind_unlink: Option<YesNo>,

    #[serde(rename = "trustedUserCAKeys", alias = "TrustedUserCAKeys", alias = "trustedusercakeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_user_ca_keys: Option<Text>,

    #[serde(rename = "unusedConnectionTimeout", alias = "UnusedConnectionTimeout", alias = "unusedconnectiontimeout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unused_connection_timeout: Option<Text>,

    #[serde(rename = "x11DisplayOffset", alias = "X11DisplayOffset", alias = "x11displayoffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_display_offset: Option<Numeric>,

    #[serde(rename = "x11Forwarding", alias = "X11Forwarding", alias = "x11forwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_forwarding: Option<YesNo>,
    
    #[serde(rename = "x11UseLocalhost", alias = "X11UseLocalhost", alias = "x11uselocalhost")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_use_localhost: Option<YesNo>    
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
