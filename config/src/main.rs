use args::*;
use atty::Stream;
use clap::Parser;
use dsc_lib::dscresources::dscresource::Invoke;
use std::io::{self, Read};
use std::process::exit;
use dsc_lib::{DscManager, dscresources::dscresource::DscResource};

#[cfg(debug_assertions)]
use crossterm::event;
#[cfg(debug_assertions)]
use std::env;

pub mod args;

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_ARGS: i32 = 1;
const EXIT_DSC_ERROR: i32 = 2;
const EXIT_JSON_ERROR: i32 = 3;

fn main() {
    #[cfg(debug_assertions)]
    check_debug();

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

    let mut dsc = match DscManager::new() {
        Ok(dsc) => dsc,
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(EXIT_DSC_ERROR);
        }
    };

    match args.subcommand {
        SubCommand::List { resource_name } => {
            match dsc.initialize_discovery() {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("Error: {}", err);
                    exit(EXIT_DSC_ERROR);
                }
            };
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
            // TODO: support streaming stdin which includes resource and input

            let input = check_stdin_and_input(&input, &stdin);
            let resource = get_resource(&mut dsc, resource.as_str());
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
            let input = check_stdin_and_input(&None, &stdin);
            let resource = get_resource(&mut dsc, resource.as_str());
            match resource.set(input.as_str()) {
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
        SubCommand::Test { resource, input: _ } => {
            let input = check_stdin_and_input(&None, &stdin);
            let resource = get_resource(&mut dsc, resource.as_str());
            match resource.test(input.as_str()) {
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
    }

    exit(EXIT_SUCCESS);
}

fn get_resource(dsc: &mut DscManager, resource: &str) -> DscResource {
    // check if resource is JSON or just a name
    match serde_json::from_str(resource) {
        Ok(resource) => resource,
        Err(_err) => {
            match dsc.initialize_discovery() {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("Error: {}", err);
                    exit(EXIT_DSC_ERROR);
                }
            };
            let resources: Vec<DscResource> = dsc.find_resource(resource).collect();
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
    }
}

fn check_stdin_and_input(input: &Option<String>, stdin: &Option<String>) -> String {
    let input = match (input, stdin) {
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
    };

    match serde_json::from_str::<serde_json::Value>(input.as_str()) {
        Ok(_) => input,
        Err(err) => {
            eprintln!("Input JSON Error: {}", err);
            exit(EXIT_INVALID_ARGS);
        }
    }
}

#[cfg(debug_assertions)]
fn check_debug() {
    if env::var("DEBUG_CONFIG").is_ok() {
        eprintln!("attach debugger to pid {} and press any key to continue", std::process::id());
        loop {
            let event = event::read().unwrap();
            match event {
                event::Event::Key(_key) => {
                    break;
                }
                _ => {
                    eprintln!("Unexpected event: {:?}", event);
                    continue;
                }
            }
        }
    }
}
