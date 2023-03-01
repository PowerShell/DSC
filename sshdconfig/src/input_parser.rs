use std::path::PathBuf;

use crate::{sshdconfig_error::SshdConfigError, config::SshdManager};

pub enum InputData {
    Text(PathBuf),
    Json(String),
    None,
}

// parse_input_data will unwrap inputs from command line
// and ensure that, at most, one input is provided
// return InputData
pub fn parse_input_data(input_config_text: &Option<String>, input_config_json: &Option<String>, 
    input_config_stdin: &Option<String>) -> Result<InputData, SshdConfigError> {
    Ok(InputData::None)
}

// parse optional input for path to current config file
// if none is provided, find the location of the default
// config file (OS dependent)
// if no valid filepath is found, throw an error
pub fn get_input_filepath(filepath: &Option<String>) -> Result<PathBuf, SshdConfigError> {
    Ok(PathBuf::from("not implemented yet"))
}

// initial_setup calls out to parse_input_data and 
// get_input_filepath since this is shared between
// the get, set, and test commands
pub fn initial_setup(input_config_text: &Option<String>, input_config_json: &Option<String>, 
    input_config_stdin: &Option<String>, curr_filepath: &Option<String>) -> Result<(InputData, SshdManager), SshdConfigError> {
    let input_data = match parse_input_data(input_config_text, input_config_json, input_config_stdin) {
        Ok(result) => result,
        Err(_) => { return Err(SshdConfigError::NotImplemented)}
    };
    let curr_filepath = match get_input_filepath(curr_filepath) {
        Ok(result) => result,
        Err(_) => {return Err(SshdConfigError::NotImplemented)}
    };
    let sshdconfig = SshdManager::new();
    match sshdconfig.import_sshd_config(&curr_filepath) {
        Ok(_) => {},
        Err(_) => { return Err(SshdConfigError::NotImplemented) }
    }
    Ok((input_data, sshdconfig))
}
