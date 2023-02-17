use args::*;
use atty::Stream;
use clap::Parser;
use config::*;
use input_parser::*;
use std::io::{self, Read};

pub mod args;
pub mod config;
pub mod input_parser;
pub mod sshdconfig_error;

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
            let curr_sshdconfig = SshdManager::new();
            let new_sshdconfig = SshdManager::new();
            curr_sshdconfig.import_sshd_config(&input_data.curr_config_text);
            match input_data.input_config {
                InputFormat::Text(data) => {
                    new_sshdconfig.import_sshd_config(&data);
                }
                InputFormat::Json(data) => {
                    new_sshdconfig.import_json(&data);
                }
                InputFormat::None => {
                    // invalid state, TODO: catch this error appropriately
                    println!("new config, via json or text file, must be provided with set");
                }
            };
            curr_sshdconfig.set(&new_sshdconfig);
        }
        Commands::Test { input_config_path, input_config_json, curr_config_path } => {
            let input_data = parse_input_data(&input_config_path, &input_config_json, &stdin, &curr_config_path);
            let curr_sshdconfig = SshdManager::new();
            let new_sshdconfig = SshdManager::new();
            curr_sshdconfig.import_sshd_config(&input_data.curr_config_text);
            match input_data.input_config {
                InputFormat::Text(data) => {
                    new_sshdconfig.import_sshd_config(&data);
                }
                InputFormat::Json(data) => {
                    new_sshdconfig.import_json(&data);
                }
                InputFormat::None => {
                    // invalid state, TODO: catch this error appropriately
                    println!("new config, via json or text file, must be provided with test");
                }
            };
            curr_sshdconfig.test(&new_sshdconfig);
        }
    }
}
