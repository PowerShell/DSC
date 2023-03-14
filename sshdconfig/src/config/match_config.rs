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
/// MatchData will hold the following:
/// Group & a vector of group's matchcontainers, 
/// User & a vector of user's matchcontainer
/// a matchcontainer within Group will hold:
/// criteria: administrator & associated info
/// Similarly, a matchcontainer within User will hold:
/// critieria: anoncvs & associated info

/// MatchContainer holds the key-value 
/// pairs from sshd_config
/// TODO: need to confirm if all the accepted Match keywords
/// are "normal" keywords with just values
/// or if any can be repeated
/// So far, testing match keywords that accept multiple values
/// have found that they require the values to be on the same line
/// and separated by whitespace
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MatchContainer {
    // TODO: is there a good way to use sshdconfig struct with only words that apply to match?
    pub criteria: String,
    #[serde(rename = "passwordauthentication")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_authentication: Option<RepeatKeyword>,
    #[serde(rename = "authorizedkeysfile")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys_file: Option<RepeatKeyword>,
    #[serde(rename = "_ensure")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ensure: Option<EnsureKind>,
}

/// MatchData is the highest level struct that 
/// can hold all the different match criteria
/// and nests their values in the MatchContainer
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MatchData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<Vec<MatchContainer>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<Vec<MatchContainer>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<Vec<MatchContainer>>,
    #[serde(rename = "localAddress")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_address: Option<Vec<MatchContainer>>,
    #[serde(rename = "localPort")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_port: Option<Vec<MatchContainer>>,
    #[serde(rename = "rDomain")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r_domain: Option<Vec<MatchContainer>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Vec<MatchContainer>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<Vec<MatchContainer>>,
}
