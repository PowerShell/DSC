
pub enum InputData {
    Text(String),
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
pub fn get_input_filepath(filepath: &Option<String>) -> String {
    "not implemented yet".to_string()
}
