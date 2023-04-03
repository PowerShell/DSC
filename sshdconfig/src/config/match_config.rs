use serde::{Deserialize, Serialize};

use crate::config::config::*;

/// This file defines structs
/// related to the match keyword and how it will
/// be represented within the config_data struct
/// #Example
/// an sshd_config file with the following:
/// Match Group administrators
///     AuthorizedKeysFile C:\\programdata\\ssh\\administrators_authorized_keys
/// Match User anoncvs
///     PermitListen 1234
/// Each block is represented by the MatchContainer struct
/// in order to preserve the order when writing to sshd_config
/// the keywords within each match block are
/// represented by the MatchSubContainer struct

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchConditional {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "group")]
    Group,
    #[serde(rename = "hosts")]
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

/// MatchSubContainer holds the key-value 
/// pairs from sshd_config
/// TODO: need to confirm if all the accepted Match keywords
/// are "normal" keywords with just values
/// or if any can be repeated
/// So far, testing match keywords that accept multiple values
/// have found that they require the values to be on the same line
/// and separated by whitespace , example: PermitListen
/// TODO: is there a good way to reuse sshdconfig struct with only keywords that apply to match
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MatchSubContainer {
    #[serde(rename = "acceptenv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_env: Option<RepeatKeyword>,
    #[serde(rename = "allowagentforwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_agent_forwarding: Option<RepeatKeyword>,
    #[serde(rename = "allowgroups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_groups: Option<RepeatKeyword>,
    #[serde(rename = "allowstreamlocalforwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_stream_local_forwarding: Option<RepeatKeyword>,
    #[serde(rename = "allowtcpforwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_tcp_forwarding: Option<RepeatKeyword>,
    #[serde(rename = "allowusers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_users: Option<RepeatKeyword>,
    #[serde(rename = "authenticationmethods")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication_methods: Option<RepeatKeyword>,
    #[serde(rename = "authorizedkeyscommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_command: Option<RepeatKeyword>,
    #[serde(rename = "authorizedkeyscommanduser")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_command_user: Option<RepeatKeyword>,
    #[serde(rename = "authorizedkeysfile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_file: Option<RepeatKeyword>,
    #[serde(rename = "authorizedprincipalscommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_principals_command: Option<RepeatKeyword>,
    #[serde(rename = "authorizedprincipalscommanduser")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_principals_command_user: Option<RepeatKeyword>,
    #[serde(rename = "authorizedprincipalsfile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_principals_file: Option<RepeatKeyword>,
    #[serde(rename = "banner")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<RepeatKeyword>,
    #[serde(rename = "casignaturealgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_signature_algorithms: Option<RepeatKeyword>,
    #[serde(rename = "challengeresponseauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge_response_authentication: Option<RepeatKeyword>,
    #[serde(rename = "channeltimeout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_timeout: Option<RepeatKeyword>,
    #[serde(rename = "chrootdirectory")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chroot_directory: Option<RepeatKeyword>,
    #[serde(rename = "clientalivecountmax")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_alive_count_max: Option<RepeatKeyword>,
    #[serde(rename = "clientaliveinterval")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_alive_interval: Option<RepeatKeyword>,
    #[serde(rename = "denygroups")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_groups: Option<RepeatKeyword>,
    #[serde(rename = "denyusers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_users: Option<RepeatKeyword>,
    #[serde(rename = "disableforwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_forwarding: Option<RepeatKeyword>,
    #[serde(rename = "exposeauthinfo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expose_auth_info: Option<RepeatKeyword>,
    #[serde(rename = "forcecommand")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_command: Option<RepeatKeyword>,
    #[serde(rename = "gatewayports")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_ports: Option<RepeatKeyword>,
    #[serde(rename = "gssapiauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gss_authentication: Option<RepeatKeyword>,
    #[serde(rename = "hostbasedacceptedalgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_accepted_algorithms: Option<RepeatKeyword>,
    #[serde(rename = "hostbasedacceptedkeytypes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_accepted_key_types: Option<RepeatKeyword>,
    #[serde(rename = "hostbasedauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_authentication: Option<RepeatKeyword>,
    #[serde(rename = "hostbasedusesnamefrompacketonly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostbased_uses_name_from_packet_only: Option<RepeatKeyword>,
    #[serde(rename = "ignorerhosts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_rhosts: Option<RepeatKeyword>,
    #[serde(rename = "include")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<RepeatKeyword>,
    #[serde(rename = "ipqos")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipq_o_s: Option<RepeatKeyword>,
    #[serde(rename = "kbdinteractiveauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kbd_interactive_authentication: Option<RepeatKeyword>,
    #[serde(rename = "kerberosauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kerberos_authentication: Option<RepeatKeyword>,
    #[serde(rename = "loglevel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<RepeatKeyword>,
    #[serde(rename = "logverbose")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_verbose: Option<RepeatKeyword>,
    #[serde(rename = "maxauthtries")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_auth_tries: Option<RepeatKeyword>,
    #[serde(rename = "maxsessions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_sessions: Option<RepeatKeyword>,
    #[serde(rename = "passwordauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_authentication: Option<RepeatKeyword>,
    #[serde(rename = "permitemptypasswords")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub empty_passwd: Option<RepeatKeyword>,
    #[serde(rename = "permitlisten")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_listen: Option<RepeatKeyword>,
    #[serde(rename = "permitopen")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_open: Option<RepeatKeyword>,
    #[serde(rename = "permitrootlogin")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_root_login: Option<RepeatKeyword>,
    #[serde(rename = "permittty")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_tty: Option<RepeatKeyword>,
    #[serde(rename = "permittunnel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_tunnel: Option<RepeatKeyword>,
    #[serde(rename = "permituserrc")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_user_rc: Option<RepeatKeyword>,
    #[serde(rename = "pubkeyacceptedalgorithms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_accepted_algorithms: Option<RepeatKeyword>,
    #[serde(rename = "pubkeyacceptedkeytypes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_accepted_key_types: Option<RepeatKeyword>,
    #[serde(rename = "pubkeyauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_authentication: Option<RepeatKeyword>,
    #[serde(rename = "pubkeyauthoptions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey_auth_options: Option<RepeatKeyword>,
    #[serde(rename = "rdomain")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r_domain: Option<RepeatKeyword>,
    #[serde(rename = "rekeylimit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rekey_limit: Option<RepeatKeyword>,
    #[serde(rename = "requiredrsasize")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_rsa_size: Option<RepeatKeyword>,
    #[serde(rename = "revokedkeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_keys: Option<RepeatKeyword>,
    #[serde(rename = "setenv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_env: Option<RepeatKeyword>,
    #[serde(rename = "skeyauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skey_authentication: Option<RepeatKeyword>,
    #[serde(rename = "streamlocalbindmask")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_local_bind_mask: Option<RepeatKeyword>,
    #[serde(rename = "streamlocalbindunlink")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_local_bind_unlink: Option<RepeatKeyword>,
    #[serde(rename = "trustedusercakeys")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_user_ca_keys: Option<RepeatKeyword>,
    #[serde(rename = "unusedconnectiontimeout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unused_connection_timeout: Option<RepeatKeyword>,
    #[serde(rename = "x11displayoffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_display_offset: Option<RepeatKeyword>,
    #[serde(rename = "x11forwarding")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_forwarding: Option<RepeatKeyword>,
    #[serde(rename = "x11uselocalhost")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x11_use_localhost: Option<RepeatKeyword>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MatchContainer {
    pub conditional: MatchConditional,
    pub criteria: String,
    pub data: MatchSubContainer,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
}



