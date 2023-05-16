use std::{collections::HashMap,path::PathBuf};

pub mod config;
pub mod config_helper;
pub mod match_config;

use config::SshdConfig;

use crate::sshdconfig_error::SshdConfigError;

pub struct SshdManager {
    config: SshdConfig,
    config_filepath: PathBuf,
    is_default: HashMap<String, bool>, // track whether keyword is default or custom
}

impl SshdManager {
    pub fn new() -> Self {
        Self {
            config: serde_json::from_str("{}").unwrap(),
            config_filepath: PathBuf::from("not implemented yet"),
            is_default: HashMap::new(),
        }
    }

    pub fn import_sshd_config(&self, filepath: &PathBuf) -> Result<(), SshdConfigError> {
        // populate self.config from filepath to sshd_config
        // can use convert_text_to_json
        // update self.is_default hashmap to false for each keyword from file
        Ok(())
    }

    pub fn import_json(&self, data: &String) -> Result<(), SshdConfigError> {
        // populate self.config from data
        // update self.is_default hashmap to false for each keyword from data
        Ok(())
    }

    pub fn get(&self, keywords: &Option<Vec<String>>, defaults: bool) -> Result<String, SshdConfigError> {
        // if keywords are provided, filter current config string to those values
        // if a keyword is provided but not set in the current config, set value to NULL
        // return string of current config as json
        Ok("".to_string())
    }

    pub fn set(&self, other: &SshdConfig) -> Result<bool, SshdConfigError> {
        // compare other to self
        // apply any differences from other to self
        // write contents to sshd_config file
        // restart sshd
        // return in_desired_state boolean
        Ok(false)
    }

    pub fn test(&self, other: &SshdConfig) -> Result<(String, bool), SshdConfigError> {
        // compare other to self, return differences and in_desired_state boolean
        Ok(("".to_string(), false))
    }

    pub fn get_keywords_from_file(&self, filepath: &PathBuf) -> Result<Option<Vec<String>>, SshdConfigError> {
        // read sshdconfig from filepath, parse sshdconfig, and return a list of the keywords
        Ok(Some(Vec::new()))
    }

    pub fn get_keywords_from_json(&self, data: &String) -> Result<Option<Vec<String>>, SshdConfigError> {
        // read sshdconfig from json, parse sshdconfig, and return a list of the keywords
        Ok(Some(Vec::new()))
    }

    fn validate_config(config: &SshdConfig) -> Result<(String, String, i32), SshdConfigError>{
        // export self.config to temp file
        // run sshd -T with temp file
        // delete temp file
        // return STDIN, STDERR, and exit code
        Ok(("".to_string(), "".to_string(), 0))
    }

    fn convert_text_to_json(data: &String) -> Result<SshdConfig, SshdConfigError> {
        // parse sshd_config text into sshdConfig struct
        // can be used by both import_sshdconfig & after calling validate_config
        Ok(serde_json::from_str("{}").unwrap())
    }
}

impl Default for SshdManager {
    fn default() -> Self {
        Self::new()
    }
}