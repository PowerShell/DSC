
pub enum InputFormat {
    Text(String),
    Json(String),
    None,
}

pub struct InputData {
    pub input_config: InputFormat,
    pub curr_config_text: String,
}

pub fn parse_input_data(input_config_text: &Option<String>, input_config_json: &Option<String>, 
    input_config_stdin: &Option<String>, curr_config_text: &Option<String>) -> InputData {
        InputData {
            input_config: InputFormat::None,
            curr_config_text: "".to_string(),
        }
}