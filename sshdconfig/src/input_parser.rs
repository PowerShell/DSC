
pub enum InputData {
    Text(String),
    Json(String),
    None,
}

pub fn parse_input_data(input_config_text: &Option<String>, input_config_json: &Option<String>, 
    input_config_stdin: &Option<String>, curr_config_text: &Option<String>) -> InputData {
    InputData::None
}