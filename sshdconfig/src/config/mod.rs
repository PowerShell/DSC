use std::path::PathBuf;

pub mod config;
pub mod config_helper;
pub mod match_config;

use config::*;

use crate::sshdconfig_error::SshdConfigError;

pub struct SshdManager {
    config: SshdConfig,
    config_filepath: PathBuf
}

impl SshdManager {
    pub fn new() -> Self {
        Self {
            config: Default::default(),
            config_filepath: PathBuf::from("not implemented yet"),
        }
    }

    pub fn import_sshd_config(&self, filepath: &PathBuf) -> Result<(), SshdConfigError> {
        Ok(())
    }

    pub fn import_json(&self, data: &String) -> Result<(), SshdConfigError> {
        Ok(())
    }

    pub fn get(&self, keywords: &Option<Vec<String>>) -> Result<String, SshdConfigError> {
        Ok("".to_string())
    }

    pub fn set(&self, other: &SshdManager) -> Result<bool, SshdConfigError> {
        Ok(false)
    }

    pub fn test(&self, other: &SshdManager) -> Result<(String, bool), SshdConfigError> {
        Ok(("".to_string(), false))
    }

    pub fn get_keywords_from_file(&self, filepath: &PathBuf) -> Result<Vec<String>, SshdConfigError> {
        Ok(Vec::new())
    }

    pub fn get_keywords_from_json(&self, data: &String) -> Result<Vec<String>, SshdConfigError> {
        Ok(Vec::new())
    }
}

impl Default for SshdManager {
    fn default() -> Self {
        Self::new()
    }
}