use atty::Stream;
use clap::{Parser};
use std::{io::{self, Read}, process::{Command,exit}};
use tracing::{error, info, warn, debug};

use args::{Arguments, SubCommand};
use runcommand::{RunCommand};

pub mod args;
pub mod runcommand;

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_INVALID_ARGS: i32 = 1;
pub const EXIT_DSC_ERROR: i32 = 2;
pub const EXIT_JSON_ERROR: i32 = 3;
pub const EXIT_INVALID_INPUT: i32 = 4;
pub const EXIT_VALIDATION_FAILED: i32 = 5;
pub const EXIT_CTRL_C: i32 = 6;

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
                exit(EXIT_INVALID_ARGS);
            },
        };
        // parse_input expects at most 1 input, so wrapping Some(empty input) would throw it off
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
        SubCommand::Get { arguments, executable, exit_code } => {
            let command: RunCommand = parse_input(arguments, executable, exit_code, stdin);
            println!(
                "Command is: {} {} with expected exit code: {}",
                command.executable, command.arguments.unwrap_or_default().join(" "), command.exit_code
            );
        }
        SubCommand::Set { arguments, executable, exit_code } => {
            let command: RunCommand = parse_input(arguments, executable, exit_code, stdin);
            let output = match Command::new(command.executable).args(command.arguments.unwrap_or_default()).output() {
                Ok(output) => output,
                Err(e) => {
                    // TODO should print as error
                    println!("Error: {e}");
                    exit(EXIT_INVALID_INPUT);
                }
            };

            let stdout = String::from_utf8_lossy(&output.stdout);
            // TODO should print as info
            println!("STDOUT: {}", stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            // TODO should print as warning
            println!("STDERR: {}", stderr);

            if let Some(actual_exit_code) = output.status.code() {
                if actual_exit_code != command.exit_code {
                    // TODO should print as error
                    println!("Error: expected exit code: {}, actual: {}", command.exit_code, actual_exit_code);
                    exit(EXIT_INVALID_INPUT);
                }
            }
        }
    }
}

fn parse_input(arguments: Option<Vec<String>>, executable: String, exit_code: i32, stdin: Option<String>) -> RunCommand {
    let command: RunCommand;
    if let Some(input) = stdin {
        debug!("Input: {}", input);
        command = match serde_json::from_str(&input) {
            Ok(json) => json,
            Err(_) => {
                match serde_yaml::from_str::<runcommand::RunCommand>(&input) {
                    Ok(yaml) => yaml,
                    Err(err) => {
                        error!("Error: Input is not valid JSON or YAML: {err}");
                        exit(EXIT_INVALID_INPUT);
                    }
                }
            }
        }
    } else {
        command = RunCommand {
            arguments,
            executable,
            exit_code,
        };
    }
    command
}
