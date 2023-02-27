
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
