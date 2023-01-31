pub struct Match {
    pub conditional: MatchType,
    pub keywords: KeywordType 
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

// incomplete
pub enum KeywordType {
    Binary(bool),
}


