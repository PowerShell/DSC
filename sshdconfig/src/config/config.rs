use serde::{Deserialize, Serialize};

use crate::config::match_config::MatchContainer;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnsureKind {
    Present,
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum YesNo {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
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
pub enum AddressFamily {
    #[serde(rename = "inet")]
    INet,
    #[serde(rename = "inet6")]
    INet6,
    #[serde(rename = "any")]
    Any,
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
pub enum Compression {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
    #[serde(rename = "delayed")]
    Delayed,
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
pub struct ConfigValidation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    pub exit_code: i32,
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
pub struct RepeatKeywordInt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub value: i32,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
}

// single value, boolean, repeat, match
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SshdConfig {
    #[serde(rename = "acceptEnv", alias = "AcceptEnv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_env: Option<String>,
    #[serde(rename = "addressFamily", alias = "AddressFamily")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_family: Option<AddressFamily>,
    #[serde(rename = "allowAgentForwarding", alias = "AllowAgentForwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_agent_forwarding: Option<String>,
    #[serde(rename = "allowGroups", alias = "AllowGroups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_groups: Option<String>,
    #[serde(rename = "allowStreamLocalForwarding", alias = "AllowStreamLocalForwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_stream_local_forwarding: Option<TCPFwd>,
    #[serde(rename = "allowTcpForwarding", alias = "AllowTcpForwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_tcp_forwarding: Option<TCPFwd>,
    #[serde(rename = "allowUsers", alias = "AllowUsers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_users: Option<String>,
    #[serde(rename = "authenticationMethods", alias = "AuthenticationMethods")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication_methods: Option<String>,
    #[serde(rename = "authorizedKeysCommand", alias = "AuthorizedKeysCommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_command: Option<String>,
    #[serde(rename = "authorizedKeysCommandUser", alias = "AuthorizedKeysCommandUser")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_command_user: Option<String>,
    #[serde(rename = "authorizedKeysFile", alias = "AuthorizedKeysFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_file: Option<String>,
    #[serde(rename = "authorizedPrincipalsCommand", alias = "AuthorizedPrincipalsCommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_principals_command: Option<String>,
    #[serde(rename = "authorizedPrincipalsCommandUser", alias = "AuthorizedPrincipalsCommandUser")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_principals_command_user: Option<String>,
    #[serde(rename = "authorizedPrincipalsFile", alias = "AuthorizedPrincipalsFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_principals_file: Option<String>,
    #[serde(rename = "Banner", alias = "banner")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    #[serde(rename = "cASignatureAlgorithms", alias = "CASignatureAlgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_signature_algorithms: Option<String>,
    #[serde(rename = "challengeresponseauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge_response_authentication: Option<String>,
    #[serde(rename = "channelTimeout", alias = "ChannelTimeout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_timeout: Option<String>,
    #[serde(rename = "chrootDirectory", alias = "ChrootDirectory")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chroot_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ciphers: Option<String>,
    #[serde(rename = "clientAliveCountMax", alias = "ClientAliveCountMax")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_alive_count_max: Option<String>,
    #[serde(rename = "clientAliveInterval", alias = "ClientAliveInterval")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_alive_interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<Compression>,
    #[serde(rename = "denyGroups", alias = "DenyGroups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_groups: Option<String>,
    #[serde(rename = "denyUsers", alias = "DenyUsers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_users: Option<String>,
    #[serde(rename = "disableForwarding", alias = "DisableForwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_forwarding: Option<String>,
    #[serde(rename = "dsaauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dsa_authentication: Option<String>,
    #[serde(rename = "exposeAuthInfo", alias = "ExposeAuthInfo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expose_auth_info: Option<String>,
    #[serde(rename = "fingerprintHash", alias = "FingerprintHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint_hash: Option<String>,
    #[serde(rename = "forceCommand", alias = "ForceCommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_command: Option<String>,
    #[serde(rename = "gatewayPorts", alias = "GatewayPorts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_ports: Option<GatewayPorts>,
    #[serde(rename = "gssapiauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gss_authentication: Option<String>,
    #[serde(rename = "gssapicleanupcreds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gss_cleanup_creds: Option<String>,
    #[serde(rename = "gssapistrictacceptor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gss_strict_acceptor: Option<String>,
    #[serde(rename = "hostbasedAcceptedAlgorithms", alias = "HostbasedAcceptedAlgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_accepted_algorithms: Option<String>,
    #[serde(rename = "hostbasedacceptedkeytypes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_accepted_key_types: Option<String>,
    #[serde(rename = "hostbasedAuthentication", alias = "HostbasedAuthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_authentication: Option<String>,
    #[serde(rename = "hostbasedUsesNameFromPacketOnly", alias = "HostbasedUsesNameFromPacketOnly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_uses_name_from_packet_only: Option<String>,
    #[serde(rename = "hostCertificate", alias = "HostCertificate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_certificate: Option<String>,
    #[serde(rename = "hostkey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_key_file: Option<String>,
    #[serde(rename = "hostdsakey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_dsa_key_file: Option<String>,
    #[serde(rename = "hostKeyAgent", alias = "HostKeyAgent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_key_agent: Option<String>,
    #[serde(rename = "hostKeyAlgorithms", alias = "HostKeyAlgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_key_algorithms: Option<String>,
    #[serde(rename = "ignoreRhosts", alias = "IgnoreRhosts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_rhosts: Option<IgnoreRhosts>,
    #[serde(rename = "ignoreUserKnownHosts", alias = "IgnoreUserKnownHosts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_user_known_hosts: Option<String>,
    #[serde(rename = "Include", alias = "include")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<String>,
    #[serde(rename = "iPQoS", alias = "IPQoS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipq_o_s: Option<String>,
    #[serde(rename = "kbdInteractiveAuthentication", alias = "KbdInteractiveAuthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kbd_interactive_authentication: Option<String>,
    #[serde(rename = "kerberosAuthentication", alias = "KerberosAuthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_authentication: Option<String>,
    #[serde(rename = "kerberosGetAFSToken", alias = "KerberosGetAFSToken")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_get_afs_token: Option<String>,
    #[serde(rename = "kerberosOrLocalPasswd", alias = "KerberosOrLocalPasswd")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_or_local_passwd: Option<String>,
    #[serde(rename = "kerberosTicketCleanup", alias = "KerberosTicketCleanup")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_ticket_cleanup: Option<String>,
    #[serde(rename = "kexAlgorithms", alias = "KexAlgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kex_algorithms: Option<String>,
    #[serde(rename = "listenAddress", alias = "ListenAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_address: Option<String>,
    #[serde(rename = "loginGraceTime", alias = "LoginGraceTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_grace_time: Option<String>,
    #[serde(rename = "logLevel", alias = "LogLevel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    #[serde(rename = "logVerbose", alias = "LogVerbose")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_verbose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macs: Option<String>,
    #[serde(rename = "match", alias = "Match")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _match: Option<Vec<MatchContainer>>,
    #[serde(rename = "maxAuthTries", alias = "MaxAuthTries")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_auth_tries: Option<String>,
    #[serde(rename = "maxSessions", alias = "MaxSessions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_sessions: Option<String>,
    #[serde(rename = "maxStartups", alias = "MaxStartups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_startups: Option<String>,
    #[serde(rename = "moduliFile", alias = "ModuliFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moduli_file: Option<String>,
    #[serde(rename = "passwordAuthentication", alias = "PasswordAuthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_authentication: Option<YesNo>,
    #[serde(rename = "permitemptypasswords")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty_passwd: Option<String>,
    #[serde(rename = "permitListen", alias = "PermitListen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_listen: Option<String>,
    #[serde(rename = "permitOpen", alias = "PermitOpen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_open: Option<String>,
    #[serde(rename = "permitRootLogin", alias = "PermitRootLogin")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_root_login: Option<PermitRootLogin>,
    #[serde(rename = "permitTTY", alias = "PermitTTY")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_tty: Option<String>,
    #[serde(rename = "permitTunnel", alias = "PermitTunnel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_tunnel: Option<String>,
    #[serde(rename = "permitUserEnvironment", alias = "PermitUserEnvironment")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_user_environment: Option<String>,
    #[serde(rename = "permitUserRC", alias = "PermitUserRC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_user_rc: Option<String>,
    #[serde(rename = "perSourceMaxStartups", alias = "PerSourceMaxStartups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_source_max_startups: Option<String>,
    #[serde(rename = "perSourceNetBlockSize", alias = "PerSourceNetBlockSize")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_source_net_block_size: Option<String>,
    #[serde(rename = "pidFile", alias = "PidFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<Vec<RepeatKeywordInt>>,
    #[serde(rename = "printLastLog", alias = "PrintLastLog")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_last_log: Option<String>,
    #[serde(rename = "printMotd", alias = "PrintMotd")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_motd: Option<String>,
    #[serde(rename = "pubkeyAcceptedAlgorithms", alias = "PubkeyAcceptedAlgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_accepted_algorithms: Option<String>,
    #[serde(rename = "pubkeyacceptedkeytypes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_accepted_key_types: Option<String>,
    #[serde(rename = "pubkeyAuthentication", alias = "PubkeyAuthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_authentication: Option<String>,
    #[serde(rename = "pubkeyAuthOptions", alias = "PubkeyAuthOptions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_auth_options: Option<String>,
    #[serde(rename = "rDomain", alias = "RDomain")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r_domain: Option<String>,
    #[serde(rename = "rekeyLimit", alias = "RekeyLimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rekey_limit: Option<String>,
    #[serde(rename = "requiredRSASize", alias = "RequiredRSASize")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_rsa_size: Option<String>,
    #[serde(rename = "revokedKeys", alias = "RevokedKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_keys: Option<String>,
    #[serde(rename = "securityKeyProvider", alias = "SecurityKeyProvider")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_key_provider: Option<String>,
    #[serde(rename = "setEnv", alias = "SetEnv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_env: Option<String>,
    #[serde(rename = "skeyauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skey_authentication: Option<String>,
    #[serde(rename = "streamLocalBindMask", alias = "StreamLocalBindMask")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_local_bind_mask: Option<String>,
    #[serde(rename = "streamLocalBindUnlink", alias = "StreamLocalBindUnlink")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_local_bind_unlink: Option<String>,
    #[serde(rename = "strictModes", alias = "StrictModes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_modes: Option<String>,
    #[serde(rename = "Subsystem", alias = "subsystem")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsystem: Option<Vec<RepeatKeywordString>>,
    #[serde(rename = "syslogfacility")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syslog_facility: Option<String>,
    #[serde(rename = "tCPKeepAlive", alias = "TCPKeepAlive")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_keep_alive: Option<String>,
    #[serde(rename = "trustedUserCAKeys", alias = "TrustedUserCAKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_user_ca_keys: Option<String>,
    #[serde(rename = "unusedConnectionTimeout", alias = "UnusedConnectionTimeout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unused_connection_timeout: Option<String>,
    #[serde(rename = "useDNS", alias = "UseDNS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_dns: Option<String>,
    #[serde(rename = "versionAddendum", alias = "VersionAddendum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_addendum: Option<String>,
    #[serde(rename = "x11DisplayOffset", alias = "X11DisplayOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_display_offset: Option<String>,
    #[serde(rename = "x11Forwarding", alias = "X11Forwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_forwarding: Option<String>,
    #[serde(rename = "x11UseLocalhost", alias = "X11UseLocalhost")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_use_localhost: Option<String>,
    #[serde(rename = "xAuthLocation", alias = "XAuthLocation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_auth_location: Option<String>,
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

