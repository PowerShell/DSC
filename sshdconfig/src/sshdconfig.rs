pub struct Match {
    pub criteria: MatchType,
    pub keywords: KeywordType 
}

// incomplete
pub enum MatchType {
    User(String),
    Group(String),
    All,
}

// incomplete
pub enum KeywordType {
    Binary(bool),
}



