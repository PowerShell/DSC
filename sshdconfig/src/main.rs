use args::*;
use atty::Stream;
use clap::Parser;
use input_parser::*;
use sshdconfig_lib::*;
use std::io::{self, Read};

pub mod args;
pub mod input_parser;

fn main() {
    let args = Cli::parse();

    let stdin: Option<String> = if atty::is(Stream::Stdin) {
        None
    } else {
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        let input = match String::from_utf8(buffer) {
            Ok(input) => input,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
        Some(input)
    };

    match args.command {
        Commands::Get { input_config_path, input_config_json, curr_config_path } => {
            let input_data = parse_input_data(&input_config_path, &input_config_json, &stdin, &curr_config_path);
            let sshdconfig = SshdManager::new();
            sshdconfig.import_sshd_config(&input_data.curr_config_text);
            let keywords = match input_data.input_config {
                InputFormat::Text(data) => {
                    Some(sshdconfig.get_keywords_from_text(&data))
                }
                InputFormat::Json(data) => {
                    Some(sshdconfig.get_keywords_from_json(&data))
                }
                InputFormat::None => {
                    None
                }
            };
            sshdconfig.get(&keywords);
        }
        Commands::Set { input_config_path, input_config_json, curr_config_path } => {
            let input_data = parse_input_data(&input_config_path, &input_config_json, &stdin, &curr_config_path);
            let sshdconfig = SshdManager::new();
            sshdconfig.import_sshd_config(&input_data.curr_config_text);
            // need to parse input file/json/stdin into a usable object to call set with
            sshdconfig.set();
        }
        Commands::Test { input_config_path, input_config_json, curr_config_path } => {
            let input_data = parse_input_data(&input_config_path, &input_config_json, &stdin, &curr_config_path);
            let sshdconfig = SshdManager::new();
            sshdconfig.import_sshd_config(&input_data.curr_config_text);
            // need to parse input file/json/stdin into a usable object to call test with
            sshdconfig.test();
        }
    }
}
