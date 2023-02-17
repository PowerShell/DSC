use std::collections::HashMap;
use crate::config::subcontainer::{SubContainer};

/// this is a helper function to
/// pull the keywords from an sshd_config file
/// can be used with get, if only a partial
/// config is being requested by the user
pub fn get_keywords_from_text(data: &String) -> Vec<String> {
    Vec::new()
}

/// this is a helper function to
/// pull the keywords from an sshd_config json
/// can be used with get, if only a partial
/// config is being requested by the user
pub fn get_keywords_from_json(data: &String) -> Vec<String> {
    Vec::new()
}

/// export_json will be called from get & test
/// to format internal representation of sshd_config to the user (get)
/// or differences between internal sshd_config & input (test)
/// can optionally provide filter to only include certain keywords
/// # Example
/// cd = ConfigData::new();
/// cd.export_json(vec!["Subsystem"-to_string(), "Port".to_string()])
pub fn export_json(config: &HashMap<String, SubContainer> , filter: &Option<Vec<String>>) -> String {
    "".to_string()
}

/// export_sshd_config will be called from set
/// to format internal representation of sshd_config to the actual file
/// not sure yet if filter will be required, similar to export_json
/// needs to generate unique filehash for future file_check
/// # Example
/// cd = ConfigData::new();
/// cd.export_sshd_config()
pub fn export_sshd_config(config: &HashMap<String, SubContainer>, filepath: &String) -> String {
    "".to_string()
}

/// validate_config will call sshd -T
/// it will return a bool indicating validity 
/// if config is valid, the default values returned by SSHD
/// and would need to be parsed 
/// # Example
/// cd = ConfigData::new();
/// cd.import_sshd_config("Port abc")
/// cd.validate_config()
pub fn validate_config(filepath: &String) -> (bool, String) {
    (false, "".to_string())
}