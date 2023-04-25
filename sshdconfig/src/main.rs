use args::*;
use atty::Stream;
use clap::Parser;
use config::config::SshdConfig;
use input_helper::*;
use sshdconfig_error::*;
use std::{io::{self, Read}, process::exit};

pub mod args;
pub mod config;
pub mod input_helper;
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

    let input_data;
    let curr_sshdconfig;
    match &args.command {
        Commands::Get {config_path, 
            config_json, curr_config_path, ..} |
        Commands::Set {config_path, 
            config_json, curr_config_path} |
        Commands::Test {config_path, 
            config_json, curr_config_path} => {
            match parse_input(&config_path, &config_json, 
                &stdin, &curr_config_path) {
                Ok(result) => {
                    input_data = result.0;
                    curr_sshdconfig = result.1;
                }
                Err(e) => {
                    eprintln!("Error getting input: {}", e);
                    exit(EXIT_INPUT_INVALID);
                }
            }
        }
    }

    match args.command {
        Commands::Get {include_defaults, ..} => {
            let keywords = match parse_keywords(&input_data, &curr_sshdconfig) {
                Ok(keywords) => {
                    keywords
                }
                Err(e) => {
                    eprintln!("Error getting input keywords: {}", e);
                    exit(EXIT_INPUT_INVALID)
                }
            };
            match curr_sshdconfig.get(&keywords, include_defaults) {
                Ok(result) => {
                    println!("{}", result);
                },
                Err(e) => {
                    eprintln!("Error getting sshd config: {}", e);
                    exit(EXIT_UNSPECIFIED_ERR);
                }
            }
        }
        Commands::Set {..} | Commands::Test {..} => {
            let new_sshdconfig = match initialize_new_config(&input_data) {
                Ok(new_sshdconfig) => {
                    new_sshdconfig
                }
                Err(e) => {
                    eprintln!("Error initializing sshdconfig: {}", e);
                    exit(EXIT_INPUT_INVALID)
                }
            };
            match &args.command {
                Commands::Set {..} => {
                    match curr_sshdconfig.set(&new_sshdconfig) {
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
                Commands::Test {..} => {
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
                _ => {}
            }
        }
    }
    exit(EXIT_SUCCESS);
}

// mainly an example at this point
#[test]
fn test_config() {
    let input_json: &str = r#"
    {
        "passwordauthentication": "yes",
        "syslogfacility": "INFO",
        "subsystem": [
            {
                "name": "powershell",
                "value": "pwsh.exe"
            }
        ],
        "port": [
            { "value": "24" },
            { "value": "23" }
        ],
        "match": [
            {
                "conditionalkey": "group",
                "conditionalvalue": "administrator",
                "data": {
                    "passwordauthentication": {
                        "value": "yes"
                    },
                    "authorizedkeysfile": {
                        "value": "test.txt",
                        "_ensure": "Absent"
                    }
                }
            },
            {
                "conditionalKey": "user",
                "conditionalValue": "anoncvs",
                "data": {
                    "passwordauthentication": {
                        "value": "no"
                    }
                }
            }
        ]
    }
    "#;
    let config: SshdConfig = serde_json::from_str(input_json).unwrap();
    //println!("{:?}", &config);
    let json = config.to_json();
    println!("{}", &json);
}
