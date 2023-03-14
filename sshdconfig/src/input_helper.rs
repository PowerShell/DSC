use std::{path::PathBuf, process::exit};
use crate::{sshdconfig_error::*, config::{config::SshdConfig, SshdManager}};

pub enum InputData {
    Text(PathBuf),
    Json(String),
    None,
}

// parse_input_data will unwrap inputs from command line
// and ensure that, at most, one input is provided
// return InputData
pub fn parse_input_data(input_config_text: &Option<String>, input_config_json: &Option<String>, 
    input_config_stdin: &Option<String>) -> InputData {
    InputData::None
}

// parse optional input for path to current config file
// if none is provided, find the location of the default
// config file (OS dependent)
// if no valid filepath is found, throw an error
pub fn get_input_filepath(filepath: &Option<String>) -> PathBuf {
    PathBuf::from("not implemented yet")
}

// initial_setup calls out to parse_input_data and 
// get_input_filepath since this is shared between
// the get, set, and test commands
pub fn initial_setup(input_config_text: &Option<String>, input_config_json: &Option<String>, 
    input_config_stdin: &Option<String>, curr_filepath: &Option<String>) -> (InputData, SshdManager) {
    let input_data = parse_input_data(input_config_text, input_config_json, input_config_stdin);
    let curr_filepath = get_input_filepath(curr_filepath);
    let sshdconfig = SshdManager::new();
    match sshdconfig.import_sshd_config(&curr_filepath) {
        Ok(_) => {},
        Err(e) => { 
            eprintln!("Invalid input error: {}", e);
            exit(EXIT_INPUT_INVALID);
        }
    }
    (input_data, sshdconfig)
}

// initialize_new_config provides a sshdconfig json from 
// the provided input to be passed to the set, and test commands
pub fn initialize_new_config(input_data: &InputData) -> SshdConfig
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
            exit(EXIT_INPUT_UNAVAILABLE);
        }
    };
    match serde_json::from_str(&input_json) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error importing new sshd config from json: {}", e);
            exit(EXIT_INPUT_INVALID);
        }
    }
}
