use crate::config::match_data::MatchData;

pub enum KeywordType {
    KeywordValue(String),
    MatchValue(MatchData)
}

pub struct SubContainer {
    pub keyword: String,
    pub args: KeywordType,
    pub is_default: bool,
}




