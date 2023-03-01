use args::*;
use atty::Stream;
use clap::Parser;
use config::*;
use input_parser::*;
use std::{io::{self, Read}, process::exit};

pub mod args;
pub mod config;
pub mod input_parser;
pub mod sshdconfig_error;

const EXIT_SUCCESS: i32 = 0;
const EXIT_UNSPECIFIED_ERR: i32 = 1;
const EXIT_INPUT_INVALID: i32 = 2;
const EXIT_INPUT_UNAVAILABLE: i32 = 3;
const EXIT_CONFIG_NOT_FOUND: i32 = 4;
const EXIT_NOT_IN_DESIRED_STATE: i32 = 5;

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
            let input_data;
            let sshdconfig;
            match initial_setup(&input_config_path, &input_config_json, &stdin, &curr_config_path) {
                Ok(result) => {
                    input_data = result.0;
                    sshdconfig = result.1;
                }
                Err(e) => {
                    eprintln!("Invalid input error: {}", e);
                    exit(EXIT_INPUT_INVALID);
                }
            }
            let keywords = match input_data {
                InputData::Text(data) => {
                    match sshdconfig.get_keywords_from_file(&data) {
                        Ok(result) => Some(result),
                        Err(_) => None
                    }
                }
                InputData::Json(data) => {
                    match sshdconfig.get_keywords_from_json(&data) {
                        Ok(result) => Some(result),
                        Err(_) => None
                    }
                }
                InputData::None => {
                    None
                }
            };
            match sshdconfig.get(&keywords) {
                Ok(result) => {
                    println!("{}", result);
                },
                Err(e) => {
                    eprintln!("Error getting sshd config: {}", e);
                    exit(EXIT_UNSPECIFIED_ERR);
                }
            }
        }
        Commands::Set { input_config_path, input_config_json, curr_config_path } => {
            let input_data;
            let curr_sshdconfig;
            match initial_setup(&input_config_path, &input_config_json, &stdin, &curr_config_path) {
                Ok(result) => {
                    input_data = result.0;
                    curr_sshdconfig = result.1;
                }
                Err(e) => {
                    eprintln!("Invalid input error: {}", e);
                    exit(EXIT_INPUT_INVALID);
                }
            }
            let new_sshdconfig = SshdManager::new();
            let should_purge = false;
            match input_data {
                InputData::Text(data) => { 
                    match new_sshdconfig.import_sshd_config(&data) {
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("Error importing new sshd config: {}", e);
                            exit(EXIT_INPUT_INVALID);
                        }
                    }
                }
                InputData::Json(data) => {
                    match new_sshdconfig.import_json(&data) {
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("Error importing new sshd config: {}", e);
                            exit(EXIT_INPUT_INVALID);
                        }
                    }
                    // look for optional _purge key in json
                }
                InputData::None => {
                    // invalid state, TODO: catch this error appropriately
                    println!("new config, via json, stdin, or text file, must be provided with set");
                }
            };
            match curr_sshdconfig.set(&new_sshdconfig, should_purge) {
                Ok(result) => {
                    if !result {
                        exit(EXIT_NOT_IN_DESIRED_STATE);
                    }
                },
                Err(e) => {
                    eprintln!("Error setting sshd config: {}", e);
                    exit(EXIT_UNSPECIFIED_ERR);
                }
            }
        }
        Commands::Test { input_config_path, input_config_json, curr_config_path } => {
            let input_data;
            let curr_sshdconfig;
            match initial_setup(&input_config_path, &input_config_json, &stdin, &curr_config_path) {
                Ok(result) => {
                    input_data = result.0;
                    curr_sshdconfig = result.1;
                }
                Err(e) => {
                    eprintln!("Invalid input error: {}", e);
                    exit(EXIT_INPUT_INVALID);
                }
            }
            let new_sshdconfig = SshdManager::new();
            match input_data {
                InputData::Text(data) => {
                    match new_sshdconfig.import_sshd_config(&data) {
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("Error importing new sshd config: {}", e);
                            exit(EXIT_INPUT_INVALID);
                        }
                    }
                }
                InputData::Json(data) => {
                    match new_sshdconfig.import_json(&data) {
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("Error importing new sshd config: {}", e);
                            exit(EXIT_INPUT_INVALID);
                        }
                    }
                }
                InputData::None => {
                    // invalid state, TODO: catch this error appropriately
                    println!("new config, via json, stdin, or text file, must be provided with test");
                }
            };
            match curr_sshdconfig.test(&new_sshdconfig) {
                Ok(result) => {
                    println!("{}", result.0);
                    if !result.1 {
                        exit(EXIT_NOT_IN_DESIRED_STATE);
                    }
                },
                Err(e) => {
                    eprintln!("Error testing sshd config: {}", e);
                    exit(EXIT_UNSPECIFIED_ERR);
                }
            }
        }
    }
    exit(EXIT_SUCCESS);
}
