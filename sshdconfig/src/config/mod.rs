use std::path::PathBuf;

pub mod match_container;
pub mod shared;
pub mod sshd;

use sshd::SshdConfig;

use crate::sshdconfig_error::SshdConfigError;

struct ConfigValidation {
    exit_code: i32,
    stderr: Option<String>,
    stdout: Option<String>,
}

pub struct SshdManager {
    config: SshdConfig,
    defaults: Option<SshdConfig>,
    filepath: PathBuf,
}

impl SshdManager {
    pub fn new() -> Self {
        Self {
            config: serde_json::from_str("{}").unwrap(),
            defaults: None,
            filepath: PathBuf::from("not implemented yet"),
        }
    }

    /// Retrieve existing sshd config settings with options to
    /// only get certain keywords and include defaults
    /// that are internally defined in SSHD program 
    /// 
    /// # Arguments
    /// 
    /// * `keywords` - If provided, a list of keywords to return from existing sshd config; ignoring any others
    /// * `defaults` - If true, return will include default values that are set in SSHD program (when not explicitly set in sshd_config)
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the configuration cannot be retrieved or formatted based on the specified inputs
    /// 
    pub fn get(&self, keywords: &Option<Vec<String>>, defaults: bool) -> Result<String, SshdConfigError> {
        // if keywords are provided, filter current config string to those values
        // if a keyword is provided but not set in the current config, set value to NULL
        // return string of current config as json
        Ok("".to_string())
    }

    /// Modify existing sshd config settings based on provided input
    /// 
    /// # Arguments
    /// 
    /// * `other` - The sshd config struct to pull the new settings from
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the configuration cannot be updated
    /// 
    pub fn set(&self, other: &SshdConfig) -> Result<bool, SshdConfigError> {
        // compare other to self
        // apply any differences from other to self
        // write contents to sshd_config file
        // restart sshd
        // return in_desired_state boolean
        Ok(false)
    }

    /// Compare existing sshd config settings to provided input
    /// 
    /// # Arguments
    /// 
    /// * `other` - The sshd config struct to be used for comparison
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the configurations cannot be compared
    /// 
    pub fn test(&self, other: &SshdConfig) -> Result<(String, bool), SshdConfigError> {
        // compare other to self, return differences and in_desired_state boolean
        Ok(("".to_string(), false))
    }

    /// Retrieve a list of keywords from a sshd config file
    /// 
    /// # Arguments
    /// 
    /// * `filepath` - The path to the sshd config file 
    /// 
    /// # Errors
    /// 
    /// This function will return an error if no valid keywords are found
    /// 
    pub fn get_keywords_from_file(&self, filepath: &PathBuf) -> Result<Option<Vec<String>>, SshdConfigError> {
        // read sshdconfig from filepath, parse sshdconfig, and return a list of the keywords
        Ok(Some(Vec::new()))
    }

    /// Retrieve a list of keywords from a sshd config json
    /// 
    /// # Arguments
    /// 
    /// * `data` - A json representation of sshd config
    /// 
    /// # Errors
    /// 
    /// This function will return an error if no valid keywords are found
    /// 
    pub fn get_keywords_from_json(&self, data: &String) -> Result<Option<Vec<String>>, SshdConfigError> {
        // read sshdconfig from json, parse sshdconfig, and return a list of the keywords
        Ok(Some(Vec::new()))
    }

    /// Import sshd config from text file
    /// 
    /// # Arguments
    /// 
    /// * `filepath` - The path to the sshd config file to be imported
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the file contents are invalid
    /// 
    pub fn import_sshd_config(&self, filepath: &PathBuf) -> Result<(), SshdConfigError> {
        // populate self.config from filepath to sshd_config
        // can use convert_text_to_json
        Ok(())
    }

    fn convert_text_to_json(data: &String) -> Result<(), SshdConfigError> {
        // parse sshd_config text into self.config_struct
        // can be used by both import_sshdconfig & after calling validate_config
        Ok(())
    }


    fn convert_json_to_text() -> Result<String, SshdConfigError> {
        // parse self.config_struct to sshd_config text,
        // can be used to generate sshd_config text string to export to file
        Ok(String::new())
    }

    fn export_text_to_file(data: &String) -> Result<(), SshdConfigError> {
        // write data to self.config_filepath
        Ok(())
    }

    fn validate_config(config: &SshdConfig) -> Result<ConfigValidation, SshdConfigError> {
        // export self.config to temp file
        // run sshd -T with temp file
        // delete temp file
        // return STDIN, STDERR, and exit code
        Ok(ConfigValidation {exit_code: 0, stderr: None, stdout: None, })
    }
}

impl Default for SshdManager {
    fn default() -> Self {
        Self::new()
    }
}
