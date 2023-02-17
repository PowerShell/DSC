pub mod config_data;
pub mod const_keywords;
pub mod match_data;
pub mod subcontainer;
pub mod utils;

use config_data::*;
use utils::*;

pub struct SshdManager {
    config_container: ConfigData,
}

impl SshdManager {
    pub fn new() -> Self {
        Self {
            config_container: ConfigData::new(),
        }
    }

    pub fn import_sshd_config(&self, data: &String) {
        self.config_container.import_sshd_config(data);
    }

    pub fn import_json(&self, data: &String) {
        self.config_container.import_json(data);
    }

    pub fn get(&self, keywords: &Option<Vec<String>>) {
        self.config_container.get(&keywords);
    }

    pub fn set(&self, other: &SshdManager) {
        self.config_container.set(&other.config_container);
    }

    pub fn test(&self, other: &SshdManager) {
        self.config_container.test(&other.config_container);
    }

    pub fn get_keywords_from_text(&self, data: &String) -> Vec<String> {
        get_keywords_from_text(data)
    }

    pub fn get_keywords_from_json(&self, data: &String) -> Vec<String> {
        get_keywords_from_json(data)
    }
}

impl Default for SshdManager {
    fn default() -> Self {
        Self::new()
    }
}