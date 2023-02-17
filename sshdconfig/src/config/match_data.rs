use std::collections::HashMap;

/// This file defines structs and enums
/// related to the match keyword and how it will
/// be represented within the config_data struct
/// #Example
/// an sshd_config file with the following:
/// Match Group administrators
///     AuthorizedKeysFile C:\\programdata\\ssh\\administrators_authorized_keys
/// Match User anoncvs
///     PermitListen 1234
/// MatchData will hold a hashmap with the following:
/// Group & group's matchcontainer, User & user's matchcontainer
/// group's matchcontainer will hold a hashmap with:
/// Administrators & administrator's matchsubcontainer
/// Administrator's matchsubcontainer will hold a hashmap with:
/// AuthorizedKeysFile, C:\\programdata\\ssh\\administrators_authorized_keys
/// Similarly, user's matchcontainer would replicate the
/// corresponding matchsubcontainer & hashmap for anoncvs

/// MatchData is the highest level struct that 
/// can hold all the different match types
/// and nests their arguments in the MatchContainer
pub struct MatchData {
    pub match_lookup: HashMap<MatchType, MatchContainer>,
}

/// MatchContainer holds the arg 
/// for the match type & it's subcontainer
pub struct MatchContainer {
    pub container: HashMap<String, MatchSubContainer>
}

/// MatchSubContainer holds the key-value 
/// pairs from sshd_config
/// TODO: need to confirm if all the accepted Match keywords
/// are "normal" keywords with just values
/// or if any can be repeated
/// So far, testing match keywords that accept multiple values
/// have found that they require the values to be on the same line
/// and separated by whitespace
pub struct MatchSubContainer {
    pub subcontainer: HashMap<String, String>
}

pub enum MatchType {
    User(String),
    Group(String),
    Host(String), 
    LocalAddress(String),
    LocalPort(String), 
    RDomain(String),
    Address(String),
    All(String),
}
