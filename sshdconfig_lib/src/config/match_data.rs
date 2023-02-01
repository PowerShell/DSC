use std::collections::HashMap;

pub struct MatchSubContainer {
    pub keyword: String,
    pub args: String,
}

// incomplete
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

pub struct MatchData {
    pub match_type: MatchType,
    pub match_lookup: HashMap<String, MatchSubContainer>,
}