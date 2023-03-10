use args::*;
use atty::Stream;
use clap::Parser;
use dsc_lib::dscresources::dscresource::Invoke;
use std::io::{self, Read};
use std::process::exit;
use dsc_lib::{DscManager, dscresources::dscresource::DscResource};

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
        SubCommand::Get { resource, input } => {
            // check if resource is a DscResource we use directly
            // or just a name and we have to search for it
            let input = check_stdin_and_input(&input, &stdin);
            let resource = match serde_json::from_str(input.as_str()) {
                Ok(resource) => resource,
                Err(_err) => {
                    let resources: Vec<DscResource> = dsc.find_resource(&resource).collect();
                    match resources.len() {
                        0 => {
                            eprintln!("Error: Resource not found");
                            exit(EXIT_INVALID_ARGS);
                        }
                        1 => resources[0].clone(),
                        _ => {
                            eprintln!("Error: Multiple resources found");
                            exit(EXIT_INVALID_ARGS);
                        }
                    }
                }
            };
            match resource.get(input.as_str()) {
                Ok(result) => {
                    // convert to json
                    let json = match serde_json::to_string(&result) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("JSON Error: {}", err);
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    println!("{}", json);
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    exit(EXIT_DSC_ERROR);
                }
            }
        }
        SubCommand::Set { resource, input: _ } => {
            println!("Set {}: {}", resource, stdin.unwrap_or_default());
        }
        SubCommand::Test { resource, input: _ } => {
            println!("Test {}: {}", resource, stdin.unwrap_or_default());
        }
    }

    exit(EXIT_SUCCESS);
}

fn check_stdin_and_input(input: &Option<String>, stdin: &Option<String>) -> String {
    match (input, stdin) {
        (Some(_input), Some(_stdin)) => {
            eprintln!("Error: Cannot specify both --input and stdin");
            exit(EXIT_INVALID_ARGS);
        }
        (Some(input), None) => input.clone(),
        (None, Some(stdin)) => stdin.clone(),
        (None, None) => {
            eprintln!("Error: No input specified");
            exit(EXIT_INVALID_ARGS);
        },
    }
}
