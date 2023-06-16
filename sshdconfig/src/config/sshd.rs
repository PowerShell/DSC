use serde::{Deserialize, Serialize};

use crate::config::match_container::MatchContainer;
use crate::config::keywords::{AddressFamily, ChannelTimeout, Compression, FingerprintHash, GatewayPorts, 
    IgnoreRhosts, IPQoS, Lists, ListsAddRemove, ListenAddress, LogLevel, MaxStartups, NetBlockSize, 
    Numeric, PermitListen, PermitOpen, PermitRootLogin, PermitTunnel, PerSourceMaxStartups, PubkeyAuthOptions, 
    RepeatNumericKeyword, RepeatTextKeyword, Text, SysLogFacility, TCPFwd, YesNo};

/// A struct representing sshd_config data
///
/// # Examples
///
/// ```
/// let sshd_config = SshdConfig { passwordAuthentication: "no", port: 23 };
/// assert_eq!(sshd_config.password_authentication, "no");
/// assert_eq!(sshd_config.port, 23);
/// ```
///
/// # Fields
///
/// * Each keyword permitted in sshd_config is an optional field: https://man.openbsd.org/sshd_config.5
/// * `purge`: an optional boolean for set commands, will clobber existing config when set to true
/// * `defaults`: points to another SshdConfig struct that only contains keywords-values set by SSHD
/// 
///
/// # Note
///
/// In general, most sshd_config keywords fall into one of the following types: Yes/No, String, or Repeated.
/// Most keyword types are objects to allow for input formatted as either keyword-value or keyword-value & _ensure.
/// Some keywords (like Compression) have a subset of permitted values so they are explicitly defined enums.
/// Some keywords (like Port) can be repeated so they have an explicitly defined struct.
/// The Match keyword has its own struct, that is defined in match_container.rs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SshdConfig {
    #[serde(rename = "acceptEnv", alias = "AcceptEnv", alias = "acceptenv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// multiple environment variables can be separated by whitespace or
    /// spread across multiple AcceptEnv directives but process all as vec
    pub accept_env: Option<Vec<Text>>,

    #[serde(rename = "addressFamily", alias = "AddressFamily", alias = "addressfamily")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_family: Option<AddressFamily>,

    #[serde(rename = "allowAgentForwarding", alias = "AllowAgentForwarding", alias = "allowagentforwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_agent_forwarding: Option<YesNo>,

    #[serde(rename = "allowGroups", alias = "AllowGroups", alias = "allowgroups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// input is list of group name patterns separated by whitespace
    pub allow_groups: Option<Vec<Text>>,

    #[serde(rename = "allowStreamLocalForwarding", alias = "AllowStreamLocalForwarding", alias = "allowstreamlocalforwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_stream_local_forwarding: Option<TCPFwd>,

    #[serde(rename = "allowTcpForwarding", alias = "AllowTcpForwarding", alias = "allowtcpforwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_tcp_forwarding: Option<TCPFwd>,

    #[serde(rename = "allowUsers", alias = "AllowUsers", alias = "allowusers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// input is list of user name patterns separated by whitespace
    pub allow_users: Option<Vec<Text>>,

    #[serde(rename = "authenticationMethods", alias = "AuthenticationMethods", alias = "authenticationmethods")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// input is one or more comma-separated lists, each list separated by whitespace 
    pub authentication_methods: Option<Vec<Text>>,

    #[serde(rename = "authorizedKeysCommand", alias = "AuthorizedKeysCommand", alias = "authorizedkeyscommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_command: Option<Text>,

    #[serde(rename = "authorizedKeysCommandUser", alias = "AuthorizedKeysCommandUser", alias = "authorizedkeyscommanduser")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_command_user: Option<Text>,

    #[serde(rename = "authorizedKeysFile", alias = "AuthorizedKeysFile", alias = "authorizedkeysfile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// input is one or more files names each separated by whitespace or "none"
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
    /// input is a comma separated list, starting with + or -
    pub ca_signature_algorithms: Option<ListsAddRemove>,

    #[serde(rename = "challengeResponseAuthentication", alias = "ChallengeResponseAuthentication", alias = "challengeresponseauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge_response_authentication: Option<YesNo>,

    #[serde(rename = "channelTimeout", alias = "ChannelTimeout", alias = "channeltimeout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// input is "type=interval" format each separated by whitespace
    pub channel_timeout: Option<Vec<ChannelTimeout>>,

    #[serde(rename = "chrootDirectory", alias = "ChrootDirectory", alias = "chrootdirectory")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chroot_directory: Option<Text>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// input is a comma separated list, starting with +,-,^
    pub ciphers: Option<Lists>,

    #[serde(rename = "clientAliveCountMax", alias = "ClientAliveCountMax", alias = "clientalivecountmax")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_alive_count_max: Option<Numeric>,

    #[serde(rename = "clientAliveInterval", alias = "ClientAliveInterval", alias = "clientaliveinterval")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_alive_interval: Option<Numeric>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "Compression")]
    pub compression: Option<Compression>,

    #[serde(rename = "denyGroups", alias = "DenyGroups", alias = "denygroups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// input is list of user group patterns separated by whitespace
    pub deny_groups: Option<Vec<Text>>,

    #[serde(rename = "denyUsers", alias = "DenyUsers", alias = "denyusers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// input is list of user name patterns separated by whitespace
    pub deny_users: Option<Vec<Text>>,

    #[serde(rename = "disableForwarding", alias = "DisableForwarding", alias = "disableforwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_forwarding: Option<YesNo>,

    #[serde(rename = "dsaAuthentication", alias = "DsaAuthentication", alias = "dsaauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dsa_authentication: Option<Text>,

    #[serde(rename = "exposeAuthInfo", alias = "ExposeAuthInfo", alias = "exposeauthinfo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expose_auth_info: Option<YesNo>,

    #[serde(rename = "fingerprintHash", alias = "FingerprintHash", alias = "fingerprinthash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint_hash: Option<FingerprintHash>,

    #[serde(rename = "forceCommand", alias = "ForceCommand", alias = "forcecommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_command: Option<Text>,

    #[serde(rename = "gatewayPorts", alias = "GatewayPorts", alias = "gatewayports")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_ports: Option<GatewayPorts>,

    #[serde(rename = "gssApiAuthentication", alias = "GSSApiAuthentication", alias = "gssapiauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gss_authentication: Option<YesNo>,

    #[serde(rename = "gssApiCleanupCredentials", alias = "GSSApiCleanupCredentials", alias = "gssapicleanupcredentials")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gss_cleanup_credentials: Option<YesNo>,

    #[serde(rename = "gssApiStrictAcceptor", alias = "GSSApiStrictAcceptor", alias = "gssapistrictacceptor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gss_strict_acceptor: Option<YesNo>,

    #[serde(rename = "hostbasedAcceptedAlgorithms", alias = "HostbasedAcceptedAlgorithms", alias = "hostbasedacceptedalgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// input is a comma separated list, starting with +,-,^
    pub hostbased_accepted_algorithms: Option<Lists>,

    #[serde(rename = "hostbasedAcceptedKeyTypes", alias = "HostbasedAcceptedKeyTypes", alias = "hostbasedacceptedkeytypes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_accepted_key_types: Option<Text>,

    #[serde(rename = "hostbasedAuthentication", alias = "HostbasedAuthentication", alias = "hostbasedauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_authentication: Option<YesNo>,

    #[serde(rename = "hostbasedUsesNameFromPacketOnly", alias = "HostbasedUsesNameFromPacketOnly", alias = "hostbasedusesnamefrompacketonly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_uses_name_from_packet_only: Option<YesNo>,

    #[serde(rename = "hostCertificate", alias = "HostCertificate", alias = "hostcertificate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_certificate: Option<Text>,

    #[serde(rename = "hostKey", alias = "HostKey", alias = "hostkey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_key_file: Option<Vec<Text>>,

    #[serde(rename = "hostDsaKey", alias = "HostDsaKey", alias = "hostdsakey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_dsa_key_file: Option<Vec<Text>>,

    #[serde(rename = "hostKeyAgent", alias = "HostKeyAgent", alias = "hostkeyagent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_key_agent: Option<Text>,

    #[serde(rename = "hostKeyAlgorithms", alias = "HostKeyAlgorithms", alias = "hostkeyalgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// input is a comma separated list
    pub host_key_algorithms: Option<Text>,

    #[serde(rename = "ignoreRhosts", alias = "IgnoreRhosts", alias = "ignorerhosts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_rhosts: Option<IgnoreRhosts>,

    #[serde(rename = "ignoreUserKnownHosts", alias = "IgnoreUserKnownHosts", alias = "ignoreuserknownhosts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_user_known_hosts: Option<YesNo>,

    #[serde(rename = "Include", alias = "include")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Text>,

    #[serde(rename = "iPQoS", alias = "IPQoS", alias = "ipqos")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// accepts one or two arguments, separated by whitespace
    pub ipq_o_s: Option<IPQoS>,

    #[serde(rename = "kbdInteractiveAuthentication", alias = "KbdInteractiveAuthentication", alias = "kbdinteractiveauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kbd_interactive_authentication: Option<YesNo>,

    #[serde(rename = "kerberosAuthentication", alias = "KerberosAuthentication", alias = "kerberosauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_authentication: Option<YesNo>,

    #[serde(rename = "kerberosGetAFSToken", alias = "KerberosGetAFSToken", alias = "kerberosgetafstoken")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_get_afs_token: Option<YesNo>,

    #[serde(rename = "kerberosOrLocalPasswd", alias = "KerberosOrLocalPasswd", alias = "kerberosorlocalpasswd")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_or_local_passwd: Option<YesNo>,

    #[serde(rename = "kerberosTicketCleanup", alias = "KerberosTicketCleanup", alias = "kerberosticketcleanup")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_ticket_cleanup: Option<YesNo>,

    #[serde(rename = "kexAlgorithms", alias = "KexAlgorithms", alias = "kexalgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// input is a comma separated list, starting with +,-,^
    pub kex_algorithms: Option<Lists>,

    #[serde(rename = "listenAddress", alias = "ListenAddress", alias = "listenaddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_address: Option<Vec<ListenAddress>>,

    #[serde(rename = "loginGraceTime", alias = "LoginGraceTime", alias = "logingracetime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_grace_time: Option<Numeric>,

    #[serde(rename = "logLevel", alias = "LogLevel", alias = "loglevel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<LogLevel>,

    #[serde(rename = "logVerbose", alias = "LogVerbose", alias = "logverbose")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_verbose: Option<Text>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// input is a comma separated list, starting with +,-,^
    pub macs: Option<Lists>,

    #[serde(rename = "match", alias = "Match", alias = "match")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub match_keyword: Option<Vec<MatchContainer>>,

    #[serde(rename = "maxAuthTries", alias = "MaxAuthTries", alias = "maxauthtries")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_auth_tries: Option<Numeric>,

    #[serde(rename = "maxSessions", alias = "MaxSessions", alias = "maxsessions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_sessions: Option<Numeric>,

    #[serde(rename = "maxStartups", alias = "MaxStartups", alias = "maxstartups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// format is start:rate:full
    pub max_startups: Option<MaxStartups>,

    #[serde(rename = "moduliFile", alias = "ModuliFile", alias = "modulifile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moduli_file: Option<Text>,

    #[serde(rename = "passwordAuthentication", alias = "PasswordAuthentication", alias = "passwordauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_authentication: Option<YesNo>,

    #[serde(rename = "permitEmptyPasswords", alias = "PermitEmptyPasswords", alias = "permitemptypasswords")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty_passwd: Option<Text>,

    #[serde(rename = "permitListen", alias = "PermitListen", alias = "permitlisten")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// format of port or host:port with multiple entries separated by whitespace
    pub permit_listen: Option<PermitListen>,

    #[serde(rename = "permitOpen", alias = "PermitOpen", alias = "permitopen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// format of host:port, IPv4_addr:port, [IPV6_addr]:port with multiple entries separated by whitespace
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

    #[serde(rename = "permitUserEnvironment", alias = "PermitUserEnvironment", alias = "permituserenvironment")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // /valid options are yes, no or a pattern-list specifying which environment variable names to accept
    pub permit_user_environment: Option<Text>,

    #[serde(rename = "permitUserRC", alias = "PermitUserRC", alias = "permituserrc")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_user_rc: Option<YesNo>,

    #[serde(rename = "perSourceMaxStartups", alias = "PerSourceMaxStartups", alias = "persourcemaxstartups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// number of unauthenticated connections allowed from a given source address, or “none” if there is no limit
    pub per_source_max_startups: Option<PerSourceMaxStartups>,

    #[serde(rename = "perSourceNetBlockSize", alias = "PerSourceNetBlockSize", alias = "persourcenetblocksize")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Values for IPv4 and optionally IPv6 may be specified, separated by a colon
    pub per_source_net_block_size: Option<NetBlockSize>,

    #[serde(rename = "pidFile", alias = "PidFile", alias = "pidfile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid_file: Option<Text>,

    #[serde(alias = "Port")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<Vec<RepeatNumericKeyword>>,

    #[serde(rename = "printLastLog", alias = "PrintLastLog", alias = "printlastlog")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_last_log: Option<YesNo>,

    #[serde(rename = "printMotd", alias = "PrintMotd", alias = "printmotd")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_motd: Option<YesNo>,

    #[serde(rename = "pubkeyAcceptedAlgorithms", alias = "PubkeyAcceptedAlgorithms", alias = "pubkeyacceptedalgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// input is a comma separated list, starting with +,-,^
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

    #[serde(rename = "securityKeyProvider", alias = "SecurityKeyProvider", alias = "securitykeyprovider")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_key_provider: Option<Text>,

    #[serde(rename = "setEnv", alias = "SetEnv", alias = "setenv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// input in format of "NAME=VALUE" separated by whitespace
    pub set_env: Option<Vec<RepeatTextKeyword>>,

    #[serde(rename = "sKeyAuthentication", alias = "skeyauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skey_authentication: Option<Text>,

    #[serde(rename = "streamLocalBindMask", alias = "StreamLocalBindMask", alias = "streamlocalbindmask")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_local_bind_mask: Option<Text>,

    #[serde(rename = "streamLocalBindUnlink", alias = "StreamLocalBindUnlink", alias = "streamlocalbindunlink")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_local_bind_unlink: Option<YesNo>,

    #[serde(rename = "strictModes", alias = "StrictModes", alias = "strictmodes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_modes: Option<YesNo>,

    #[serde(rename = "Subsystem")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsystem: Option<Vec<RepeatTextKeyword>>,

    #[serde(rename = "syslogFacility", alias = "SyslogFacility", alias = "syslogfacility")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syslog_facility: Option<SysLogFacility>,

    #[serde(rename = "tcpKeepAlive", alias = "TCPKeepAlive", alias = "tcpkeepalive")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_keep_alive: Option<YesNo>,

    #[serde(rename = "trustedUserCAKeys", alias = "TrustedUserCAKeys", alias = "trustedusercakeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_user_ca_keys: Option<Text>,

    #[serde(rename = "unusedConnectionTimeout", alias = "UnusedConnectionTimeout", alias = "unusedconnectiontimeout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unused_connection_timeout: Option<Text>,

    #[serde(rename = "useDNS", alias = "UseDNS", alias = "usedns")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_dns: Option<YesNo>,

    #[serde(rename = "versionAddendum", alias = "VersionAddendum", alias = "versionaddendum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_addendum: Option<Text>,

    #[serde(rename = "x11DisplayOffset", alias = "X11DisplayOffset", alias = "x11displayoffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_display_offset: Option<Numeric>,

    #[serde(rename = "x11Forwarding", alias = "X11Forwarding", alias = "x11forwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_forwarding: Option<YesNo>,

    #[serde(rename = "x11UseLocalhost", alias = "X11UseLocalhost", alias = "x11uselocalhost")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_use_localhost: Option<YesNo>,

    #[serde(rename = "xAuthLocation", alias = "XAuthLocation", alias = "xauthlocation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_auth_location: Option<Text>,

    #[serde(rename = "_purge")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purge: Option<bool>, 

    #[serde(rename = "_defaults")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defaults: Option<Box<SshdConfig>>    
}

impl SshdConfig {
    pub fn to_json(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Failed to serialize to JSON: {e}");
                String::new()
            }
        }
    }
}