use std::path::PathBuf;

use crate::{sshdconfig_error::*, config::{config::SshdConfig, SshdManager}};

pub enum InputData {
    Text(PathBuf),
    Json(String),
    None,
}

// parse_input_helper will unwrap inputs from command line
// and ensure that, at most, one input is provided
pub fn parse_input_helper(input_config_text: &Option<String>, input_config_json: &Option<String>, 
    input_config_stdin: &Option<String>) -> Result<InputData, SshdConfigError> {
    Ok(InputData::None)
}

// get_input_filepath will parse optional input 
// for path to current config file
// if none is provided, find the location of the default
// config file (OS dependent)
// if no valid filepath is found, throw an error
pub fn get_input_filepath(filepath: &Option<String>) -> Result<PathBuf, SshdConfigError> {
    Ok(PathBuf::from("not implemented yet"))
}

// parse_input calls out to parse_input_helper and 
// get_input_filepath since this is shared between
// the get, set, and test commands
pub fn parse_input(input_config_text: &Option<String>, 
    input_config_json: &Option<String>, input_config_stdin: &Option<String>, 
    curr_filepath: &Option<String>) -> Result<(InputData, SshdManager), SshdConfigError> {
    let input_data = parse_input_helper(input_config_text, input_config_json, input_config_stdin)?;
    let curr_filepath = get_input_filepath(curr_filepath)?;
    let sshdconfig = SshdManager::new();
    sshdconfig.import_sshd_config(&curr_filepath)?;
    Ok((input_data, sshdconfig))
}

// initialize_new_config provides a sshdconfig json from 
// the provided input to be passed to the set and test commands
pub fn initialize_new_config(input_data: &InputData) -> Result<SshdConfig, SshdConfigError>
{
    let input_json;
    match input_data {
        InputData::Text(data) => { 
            // read data from file
            // call convert_text_to_json
            input_json = "{}".to_string();
        }
        InputData::Json(data) => {
            input_json = data.to_string();
        }
        InputData::None => {
            eprintln!("new config, via json, stdin, or text file, must be provided with set/test");
            return Err(SshdConfigError::NotImplemented);
        }
    };
    match serde_json::from_str(&input_json) {
        Ok(result) => Ok(result),
        Err(e) => {
            eprintln!("Error importing new sshd config from json: {}", e);
            return Err(SshdConfigError::NotImplemented);
        }
    }
}

// parse_keywords returns a optional list of keywords 
// to be used with get command to limit output
// to a specific set of keywords
pub fn parse_keywords(data: &InputData, sshd_config: &SshdManager) -> Result<Option<Vec<String>>,SshdConfigError> {
    match data {
        InputData::Text(data) => sshd_config.get_keywords_from_file(&data),
        InputData::Json(data) => sshd_config.get_keywords_from_json(&data),
        InputData::None => Ok(None)
    }
}