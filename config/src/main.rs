use args::*;
use atty::Stream;
use clap::Parser;
use std::io::{self, Read};
use std::process::exit;
use dsc_lib::DscManager;

pub mod args;

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_ARGS: i32 = 1;
const EXIT_DSC_ERROR: i32 = 2;
const EXIT_JSON_ERROR: i32 = 3;

fn main() {
    let args = Args::parse();

    let stdin: Option<String> = if atty::is(Stream::Stdin) {
        None
    } else {
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        let input = match String::from_utf8(buffer) {
            Ok(input) => input,
            Err(e) => {
                eprintln!("Invalid UTF-8 sequence: {}", e);
                exit(EXIT_INVALID_ARGS);
            },
        };
        Some(input)
    };

    let dsc = match DscManager::new() {
        Ok(dsc) => dsc,
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(EXIT_DSC_ERROR);
        }
    };

    match args.subcommand {
        SubCommand::List { resource_name } => {
            for resource in dsc.find_resource(&resource_name.unwrap_or_default()) {
                // convert to json
                let json = match serde_json::to_string(&resource) {
                    Ok(json) => json,
                    Err(err) => {
                        eprintln!("JSON Error: {}", err);
                        exit(EXIT_JSON_ERROR);
                    }
                };
                println!("{}", json);
            }
        }
        SubCommand::Get { resource_name ,input } => {
            println!("Get {}: {}", resource_name, stdin.unwrap_or_default());
        }
        SubCommand::Set { resource_name, input } => {
            println!("Set {}: {}", resource_name, stdin.unwrap_or_default());
        }
        SubCommand::Test { resource_name, input } => {
            println!("Test {}: {}", resource_name, stdin.unwrap_or_default());
        }
        SubCommand::Flush => {
            println!("Flush");
        }
    }

    exit(EXIT_SUCCESS);
}

fn check_stdin_and_input(input: &Option<String>, stdin: &Option<String>) -> Option<String> {
    match (input, stdin) {
        (Some(input), Some(stdin)) => {
            eprintln!("Error: Cannot specify both --input and stdin");
            exit(EXIT_INVALID_ARGS);
        }
        (Some(input), None) => Some(input.clone()),
        (None, Some(stdin)) => Some(stdin.clone()),
        (None, None) => None,
    }
}
