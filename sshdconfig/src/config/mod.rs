use std::{collections::HashMap,path::PathBuf};

pub mod config;
pub mod config_helper;
pub mod match_config;

use config::*;

use crate::sshdconfig_error::SshdConfigError;

pub struct SshdManager {
    config: SshdConfig,
    config_filepath: PathBuf,
    is_custom: HashMap<String, bool>, // track whether keyword is default or custom
    linux_only: Vec<String>, // list keywords only applicable to Linux (fairly static)
    format_output: Vec<String>, // something to hold special keywords that can have multiple inputs, but need to be output a single line like permit listen
}

impl SshdManager {
    pub fn new() -> Self {
        Self {
            config: serde_json::from_str("").unwrap(),
            config_filepath: PathBuf::from("not implemented yet"),
            is_custom: HashMap::new(),
            linux_only: Vec::new(),
            format_output: Vec::new(),
        }
    }

    pub fn import_sshd_config(&self, filepath: &PathBuf) -> Result<(), SshdConfigError> {
        Ok(())
    }

    pub fn import_json(&self, data: &String) -> Result<(), SshdConfigError> {
        Ok(())
    }

    pub fn get(&self, keywords: &Option<Vec<String>>, defaults: bool) -> Result<String, SshdConfigError> {
        Ok("".to_string())
    }

    pub fn set(&self, other: &SshdConfig) -> Result<bool, SshdConfigError> {
        Ok(false)
    }

    pub fn test(&self, other: &SshdConfig) -> Result<(String, bool), SshdConfigError> {
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