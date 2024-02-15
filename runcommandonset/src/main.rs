use atty::Stream;
use std::{io::{self, Read}, process::exit};
use tracing::{error, info, warn, debug};

use args::{Arguments, SubCommand};
use clap::{Parser};

pub mod args;
pub mod command;
pub mod error;

fn main() {

    let stdin = if atty::is(Stream::Stdin) {
        None
    } else {
        debug!("Reading input from STDIN");
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        let stdin = match String::from_utf8(buffer) {
            Ok(stdin) => stdin,
            Err(e) => {
                error!("Invalid UTF-8 sequence: {e}");
                exit(error::EXIT_INVALID_ARGS);
            },
        };
        // get_input call expects at most 1 input, so wrapping Some(empty input) would throw it off
        // have only seen this happen with dsc_args.test.ps1 running on the CI pipeline
        if stdin.is_empty() {
            debug!("Input from STDIN is empty");
            None
        }
        else {
            Some(stdin)
        }
    };

    let args = Arguments::parse();

    match args.subcommand {
        SubCommand::Get { arguments, executable, exit_code } |
        SubCommand::Set { arguments, executable, exit_code } => {
            let command: command::Command;
            if let Some(input) = stdin {
                debug!("Input: {}", input);
                command = match serde_json::from_str(&input) {
                    Ok(json) => json,
                    Err(_) => {
                        match serde_yaml::from_str::<command::Command>(&input) {
                            Ok(yaml) => yaml,
                            Err(err) => {
                                error!("Error: Input is not valid JSON or YAML: {err}");
                                exit(error::EXIT_INVALID_INPUT);
                            }
                        }
                    }
                }
            } else {
                command = command::Command {
                    arguments,
                    executable,
                    exit_code,
                };
            }
            println!("{}", command.to_json());
        }
    }
}
