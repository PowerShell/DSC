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
    #[serde(rename = "passwordauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_authentication: Option<RepeatKeyword>,
    #[serde(rename = "authorizedkeysfile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_file: Option<RepeatKeyword>,
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



