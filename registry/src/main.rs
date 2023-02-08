use args::Arguments;
use atty::Stream;
use clap::Parser;
use ntreg::{registry_key::RegistryKey, registry_value::RegistryValueData};
use ntstatuserror::NtStatusErrorKind;
use std::{io::{self, Read}, process::exit};

use crate::config::RegistryConfig;

mod args;
mod bcrypt;
mod config;

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_PARAMETER: i32 = 1;
const EXIT_INVALID_INPUT: i32 = 2;
const EXIT_REGISTRY_ERROR: i32 = 3;
const EXIT_NOT_IN_DESIRED_STATE: i32 = 4;

fn main() {
    let args = Arguments::parse();
    let input: Option<String> = if atty::is(Stream::Stdin) {
        None
    } else {
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        let input = match String::from_utf8(buffer) {
            Ok(input) => input,
            Err(e) => {
                eprintln!("Invalid UTF-8 sequence: {}", e);
                exit(EXIT_INVALID_INPUT);
            }
        };
        Some(input)
    };
    
    let mut config: RegistryConfig = Default::default();
    // check if input is valid for subcommand
    match args.subcommand {
        args::SubCommand::Config { subcommand: _ } => {
            match input {
                Some(input) => {
                    config = match serde_json::from_str(&input) {
                        Ok(config) => config,
                        Err(err) => {
                            eprintln!("Error JSON does not match schema: {}", err);
                            exit(EXIT_INVALID_INPUT);
                        }
                    };
                },
                None => {
                    eprintln!("Error: Input JSON via STDIN is required for config subcommand.");
                    exit(EXIT_INVALID_PARAMETER);
                }
            }
        }
        _ => {
            if input.is_some() {
                eprintln!("Error: Input JSON via STDIN is only valid for config subcommand.");
                exit(EXIT_INVALID_INPUT);
            }
        }
    }

    match args.subcommand {
        args::SubCommand::Query { key_path, value_name, recurse } => {
            eprintln!("Get key_path: {}, value_name: {:?}, recurse: {}", key_path, value_name, recurse);
        },
        args::SubCommand::Set { key_path, value } => {
            eprintln!("Set key_path: {}, value: {}", key_path, value);
        },
        args::SubCommand::Test => {
            eprintln!("Test");
        },
        args::SubCommand::Remove { key_path, value_name, recurse } => {
            eprintln!("Remove key_path: {}, value_name: {:?}, recurse: {}", key_path, value_name, recurse);
        },
        args::SubCommand::Find { key_path, find, recurse, keys_only, values_only } => {
            eprintln!("Find key_path: {}, find: {}, recurse: {:?}, keys_only: {:?}, values_only: {:?}", key_path, find, recurse, keys_only, values_only);
        },
        args::SubCommand::Config { subcommand } => {
            match subcommand {
                args::ConfigSubCommand::Get => {
                    config_get(config);
                },
                args::ConfigSubCommand::Set => {
                    println!("Set config");
                },
                args::ConfigSubCommand::Test => {
                    config_test(config);
                },
            }
        }
    }

    exit(EXIT_SUCCESS);
}

fn config_get(config: RegistryConfig) {
    let reg_key = match RegistryKey::new(config.key_path.as_str()) {
        Ok(reg_key) => reg_key,
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(EXIT_REGISTRY_ERROR);
        }
    };

    let mut reg_result = RegistryConfig {
        key_path: config.key_path,
        value_name: None,
        value_data: None,
        ensure: None,
        clobber: None,
    };

    if config.value_name.is_some() {
        let reg_value = match reg_key.get_value(config.value_name.unwrap().as_str()) {
            Ok(reg_value) => reg_value,
            Err(err) => {
                eprintln!("Error: {}", err);
                exit(EXIT_REGISTRY_ERROR);
            }
        };

        reg_result.value_name = Some(reg_value.name);
        reg_result.value_data = Some(convert_reg_data(reg_value.data));
    }

    let reg_json = match serde_json::to_string(&reg_result) {
        Ok(reg_json) => reg_json,
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(EXIT_REGISTRY_ERROR);
        }
    };

    println!("{}", reg_json);
}

fn config_test(config: RegistryConfig) {
    let mut reg_result = RegistryConfig {
        key_path: config.key_path.clone(),
        value_name: None,
        value_data: None,
        ensure: None,
        clobber: None,
    };

    if config.value_name.is_none() {
        let key_exists;
        match RegistryKey::new(config.key_path.as_str()) {
            Ok( _ ) => {
                key_exists = true;
            },
            Err(err) => {
                match err.status {
                    NtStatusErrorKind::ObjectNameNotFound => {
                        key_exists = false;
                    },
                    _ => {
                        eprintln!("Error: {}", err);
                        exit(EXIT_REGISTRY_ERROR);
                    }
                }
            }
        };

        match config.ensure {
            Some(ensure) => {
                reg_result.ensure = Some(ensure.clone());
                let mut in_desired_state = true;
                match ensure {
                    config::EnsureKind::Present => {
                        if !key_exists {
                            reg_result.key_path = String::new();
                            in_desired_state = false;
                        }
                    },
                    config::EnsureKind::Absent => {
                        if key_exists {
                            in_desired_state = false;
                        }
                    }
                }
                
                let reg_json = match serde_json::to_string(&reg_result) {
                    Ok(reg_json) => reg_json,
                    Err(err) => {
                        eprintln!("Error: {}", err);
                        exit(EXIT_REGISTRY_ERROR);
                    }
                };

                println!("{}", reg_json);
                match in_desired_state {
                    true => exit(EXIT_SUCCESS),
                    false => exit(EXIT_NOT_IN_DESIRED_STATE),
                }
            },
            None => {
                eprintln!("Error: `_ensure` is required if `value_name` is not specified.");
                exit(EXIT_INVALID_INPUT);
            }
        }
    }
}

fn convert_reg_data (reg_data: ntreg::registry_value::RegistryValueData) -> config::RegistryValueData {
    match reg_data {
        RegistryValueData::String(data) => config::RegistryValueData::String(data),
        RegistryValueData::MultiString(data) => config::RegistryValueData::MultiString(data),
        RegistryValueData::Binary(data) => config::RegistryValueData::Binary(data),
        RegistryValueData::DWord(data) => config::RegistryValueData::DWord(data),
        RegistryValueData::QWord(data) => config::RegistryValueData::QWord(data),
        RegistryValueData::ExpandString(data) => config::RegistryValueData::ExpandString(data),
        _ => {
            eprintln!("Error: Unsupported registry value type.");
            exit(EXIT_REGISTRY_ERROR);
        }
    }
}
