use serde::{Deserialize, Serialize};

use crate::config::match_container::MatchContainer;
use crate::config::shared::{AddressFamilyObject, CompressionObject, FingerprintHashObject, GatewayPortsObject, IntObject, IgnoreRhostsObject, LogLevelObject,
    PermitRootLoginObject, PermitTunnelObject, PubkeyAuthOptionsObject, RepeatKeywordInt, RepeatKeywordString, StringObject, SysLogFacilityObject, TCPFwdObject, YesNoObject};

// The main struct for sshd_config data -
// contains all keywords permitted in a sshd_config file, and their corresponding arg type
// In general, most keywords fall into one of the following types: Yes/No, String, or Repeated.
// Most keyword types are objects to allow for input formatted as either keyword-value or keyword-value & _ensure
// some keywords (like Compression) have a subset of permitted values so they are explicitly defined enums,
// some keywords (like Port) can be repeated so they have an explicitly defined struct.
// The Match keyword has its own struct, that is defined in match_container.rs
// There is also a section for defaults, which will point to another SshdConfig struct
// that only contains the keywords and values that are set by SSHD 
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SshdConfig {
    #[serde(rename = "acceptEnv", alias = "AcceptEnv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // multiple environment variables can be seaprated by whitespace or
    // spread across multiple AcceptEnv directives
    pub accept_env: Option<Vec<RepeatKeywordString>>,
    #[serde(rename = "addressFamily", alias = "AddressFamily")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_family: Option<AddressFamilyObject>,
    #[serde(rename = "allowAgentForwarding", alias = "AllowAgentForwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_agent_forwarding: Option<YesNoObject>,
    #[serde(rename = "allowGroups", alias = "AllowGroups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is list of group name patterns separated by whitespace
    pub allow_groups: Option<StringObject>,
    #[serde(rename = "allowStreamLocalForwarding", alias = "AllowStreamLocalForwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_stream_local_forwarding: Option<TCPFwdObject>,
    #[serde(rename = "allowTcpForwarding", alias = "AllowTcpForwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_tcp_forwarding: Option<TCPFwdObject>,
    #[serde(rename = "allowUsers", alias = "AllowUsers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is list of user name patterns separated by whitespace
    pub allow_users: Option<StringObject>,
    #[serde(rename = "authenticationMethods", alias = "AuthenticationMethods")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is one or more comma-separated lists, each list separated by whitespace 
    pub authentication_methods: Option<StringObject>,
    #[serde(rename = "authorizedKeysCommand", alias = "AuthorizedKeysCommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_command: Option<StringObject>,
    #[serde(rename = "authorizedKeysCommandUser", alias = "AuthorizedKeysCommandUser")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_command_user: Option<StringObject>,
    #[serde(rename = "authorizedKeysFile", alias = "AuthorizedKeysFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is one or more files names each separated by whitespace or "none"
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
    #[serde(rename = "caSignatureAlgorithms", alias = "CASignatureAlgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is a comma separated list, starting with + or -
    pub ca_signature_algorithms: Option<StringObject>,
    #[serde(rename = "challengeresponseauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge_response_authentication: Option<YesNoObject>,
    #[serde(rename = "channelTimeout", alias = "ChannelTimeout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is "type=interval" format each separated by whitespace
    pub channel_timeout: Option<StringObject>,
    #[serde(rename = "chrootDirectory", alias = "ChrootDirectory")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chroot_directory: Option<StringObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is a comma separated list, starting with +,-,^
    pub ciphers: Option<StringObject>,
    #[serde(rename = "clientAliveCountMax", alias = "ClientAliveCountMax")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_alive_count_max: Option<IntObject>,
    #[serde(rename = "clientAliveInterval", alias = "ClientAliveInterval")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_alive_interval: Option<IntObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<CompressionObject>,
    #[serde(rename = "denyGroups", alias = "DenyGroups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is list of user group patterns separated by whitespace
    pub deny_groups: Option<StringObject>,
    #[serde(rename = "denyUsers", alias = "DenyUsers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is list of user name patterns separated by whitespace
    pub deny_users: Option<StringObject>,
    #[serde(rename = "disableForwarding", alias = "DisableForwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_forwarding: Option<YesNoObject>,
    #[serde(rename = "dsaauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dsa_authentication: Option<StringObject>,
    #[serde(rename = "exposeAuthInfo", alias = "ExposeAuthInfo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expose_auth_info: Option<YesNoObject>,
    #[serde(rename = "fingerprintHash", alias = "FingerprintHash")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint_hash: Option<FingerprintHashObject>,
    #[serde(rename = "forceCommand", alias = "ForceCommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_command: Option<StringObject>,
    #[serde(rename = "gatewayPorts", alias = "GatewayPorts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_ports: Option<GatewayPortsObject>,
    #[serde(rename = "gssapiauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gss_authentication: Option<YesNoObject>,
    #[serde(rename = "gssapicleanupcreds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gss_cleanup_creds: Option<YesNoObject>,
    #[serde(rename = "gssapistrictacceptor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gss_strict_acceptor: Option<YesNoObject>,
    #[serde(rename = "hostbasedAcceptedAlgorithms", alias = "HostbasedAcceptedAlgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is a comma separated list, starting with +,-,^
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
    #[serde(rename = "hostCertificate", alias = "HostCertificate")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_certificate: Option<StringObject>,
    #[serde(rename = "hostkey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_key_file: Option<StringObject>,
    #[serde(rename = "hostdsakey")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_dsa_key_file: Option<StringObject>,
    #[serde(rename = "hostKeyAgent", alias = "HostKeyAgent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_key_agent: Option<StringObject>,
    #[serde(rename = "hostKeyAlgorithms", alias = "HostKeyAlgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is a comma separated list
    pub host_key_algorithms: Option<StringObject>,
    #[serde(rename = "ignoreRhosts", alias = "IgnoreRhosts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_rhosts: Option<IgnoreRhostsObject>,
    #[serde(rename = "ignoreUserKnownHosts", alias = "IgnoreUserKnownHosts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_user_known_hosts: Option<YesNoObject>,
    #[serde(rename = "Include", alias = "include")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<StringObject>,
    #[serde(rename = "iPQoS", alias = "IPQoS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // accepts one or two arguments, separated by whitespace
    pub ipq_o_s: Option<StringObject>,
    #[serde(rename = "kbdInteractiveAuthentication", alias = "KbdInteractiveAuthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kbd_interactive_authentication: Option<YesNoObject>,
    #[serde(rename = "kerberosAuthentication", alias = "KerberosAuthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_authentication: Option<YesNoObject>,
    #[serde(rename = "kerberosGetAFSToken", alias = "KerberosGetAFSToken")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_get_afs_token: Option<YesNoObject>,
    #[serde(rename = "kerberosOrLocalPasswd", alias = "KerberosOrLocalPasswd")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_or_local_passwd: Option<YesNoObject>,
    #[serde(rename = "kerberosTicketCleanup", alias = "KerberosTicketCleanup")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_ticket_cleanup: Option<YesNoObject>,
    #[serde(rename = "kexAlgorithms", alias = "KexAlgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is a comma separated list, starting with +,-,^
    pub kex_algorithms: Option<StringObject>,
    #[serde(rename = "listenAddress", alias = "ListenAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_address: Option<StringObject>,
    #[serde(rename = "loginGraceTime", alias = "LoginGraceTime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login_grace_time: Option<IntObject>,
    #[serde(rename = "logLevel", alias = "LogLevel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<LogLevelObject>,
    #[serde(rename = "logVerbose", alias = "LogVerbose")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_verbose: Option<StringObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is a comma separated list, starting with +,-,^
    pub macs: Option<StringObject>,
    #[serde(rename = "match", alias = "Match")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _match: Option<Vec<MatchContainer>>,
    #[serde(rename = "maxAuthTries", alias = "MaxAuthTries")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_auth_tries: Option<IntObject>,
    #[serde(rename = "maxSessions", alias = "MaxSessions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_sessions: Option<IntObject>,
    #[serde(rename = "maxStartups", alias = "MaxStartups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // format is start:rate:full
    pub max_startups: Option<StringObject>,
    #[serde(rename = "moduliFile", alias = "ModuliFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moduli_file: Option<StringObject>,
    #[serde(rename = "passwordAuthentication", alias = "PasswordAuthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_authentication: Option<YesNoObject>,
    #[serde(rename = "permitEmptyPasswords", alias = "PermitEmptyPasswords")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty_passwd: Option<StringObject>,
    #[serde(rename = "permitListen", alias = "PermitListen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // format of port or host:port with multiple entries separated by whitespace
    pub permit_listen: Option<StringObject>,
    #[serde(rename = "permitOpen", alias = "PermitOpen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // format of host:port, IPv4_addr:port, [IPV6_addr]:port with multiple entries separated by whitespace
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
    #[serde(rename = "permitUserEnvironment", alias = "PermitUserEnvironment")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // valid options are yes, no or a pattern-list specifying which environment variable names to accept
    pub permit_user_environment: Option<StringObject>,
    #[serde(rename = "permitUserRC", alias = "PermitUserRC")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_user_rc: Option<YesNoObject>,
    #[serde(rename = "perSourceMaxStartups", alias = "PerSourceMaxStartups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // number of unauthenticated connections allowed from a given source address, or “none” if there is no limit
    pub per_source_max_startups: Option<StringObject>,
    #[serde(rename = "perSourceNetBlockSize", alias = "PerSourceNetBlockSize")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // Values for IPv4 and optionally IPv6 may be specified, separated by a colon
    pub per_source_net_block_size: Option<StringObject>,
    #[serde(rename = "pidFile", alias = "PidFile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid_file: Option<StringObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<Vec<RepeatKeywordInt>>,
    #[serde(rename = "printLastLog", alias = "PrintLastLog")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_last_log: Option<YesNoObject>,
    #[serde(rename = "printMotd", alias = "PrintMotd")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_motd: Option<YesNoObject>,
    #[serde(rename = "pubkeyAcceptedAlgorithms", alias = "PubkeyAcceptedAlgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // input is a comma separated list, starting with +,-,^
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
    #[serde(rename = "securityKeyProvider", alias = "SecurityKeyProvider")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_key_provider: Option<StringObject>,
    #[serde(rename = "setEnv", alias = "SetEnv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    // input in format of "NAME=VALUE" separated by whitespace
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
    #[serde(rename = "strictModes", alias = "StrictModes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_modes: Option<YesNoObject>,
    #[serde(rename = "Subsystem", alias = "subsystem")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subsystem: Option<Vec<RepeatKeywordString>>,
    #[serde(rename = "syslogfacility")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syslog_facility: Option<SysLogFacilityObject>,
    #[serde(rename = "tCPKeepAlive", alias = "TCPKeepAlive")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_keep_alive: Option<YesNoObject>,
    #[serde(rename = "trustedUserCAKeys", alias = "TrustedUserCAKeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_user_ca_keys: Option<StringObject>,
    #[serde(rename = "unusedConnectionTimeout", alias = "UnusedConnectionTimeout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unused_connection_timeout: Option<StringObject>,
    #[serde(rename = "useDNS", alias = "UseDNS")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_dns: Option<YesNoObject>,
    #[serde(rename = "versionAddendum", alias = "VersionAddendum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_addendum: Option<StringObject>,
    #[serde(rename = "x11DisplayOffset", alias = "X11DisplayOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_display_offset: Option<IntObject>,
    #[serde(rename = "x11Forwarding", alias = "X11Forwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_forwarding: Option<YesNoObject>,
    #[serde(rename = "x11UseLocalhost", alias = "X11UseLocalhost")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_use_localhost: Option<YesNoObject>,
    #[serde(rename = "xAuthLocation", alias = "XAuthLocation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_auth_location: Option<StringObject>,
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
