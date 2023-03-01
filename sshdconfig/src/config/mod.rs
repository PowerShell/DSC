use std::path::PathBuf;

pub mod config_data;
pub mod const_keywords;
pub mod match_data;
pub mod subcontainer;
pub mod utils;

use config_data::*;
use utils::*;

use crate::sshdconfig_error::SshdConfigError;

pub struct SshdManager {
    config_container: ConfigData,
}

impl SshdManager {
    pub fn new() -> Self {
        Self {
            config_container: ConfigData::new(),
        }
    }

    pub fn import_sshd_config(&self, filepath: &PathBuf) -> Result<(), SshdConfigError> {
        self.config_container.import_sshd_config(filepath)
    }

    pub fn import_json(&self, data: &String) -> Result<(), SshdConfigError> {
        self.config_container.import_json(data)
    }

    pub fn get(&self, keywords: &Option<Vec<String>>) -> Result<String, SshdConfigError> {
        self.config_container.get(&keywords)
    }

    pub fn set(&self, other: &SshdManager, purge: bool) -> Result<bool, SshdConfigError> {
        self.config_container.set(&other.config_container, purge)
    }

    pub fn test(&self, other: &SshdManager) -> Result<(String, bool), SshdConfigError> {
        self.config_container.test(&other.config_container)
    }

    pub fn get_keywords_from_file(&self, filepath: &PathBuf) -> Result<Vec<String>, SshdConfigError> {
        get_keywords_from_file(filepath)
    }

    pub fn get_keywords_from_json(&self, data: &String) -> Result<Vec<String>, SshdConfigError> {
        get_keywords_from_json(data)
    }
}

impl Default for SshdManager {
    fn default() -> Self {
        Self::new()
    }
}
