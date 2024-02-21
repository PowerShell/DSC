// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use atty::Stream;
use clap::{Parser};
use std::{io::{self, Read}, process::exit};
use tracing::{error, warn, debug};

use args::{Arguments, SubCommand};
use runcommand::{RunCommand};
use utils::{enable_tracing, invoke_command, parse_input, EXIT_INVALID_ARGS};

pub mod args;
pub mod runcommand;
pub mod utils;

fn main() {
    let args = Arguments::parse();
    enable_tracing(&args.trace_level, &args.trace_format);
    warn!("This resource is not idempotent");

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

    let mut command: RunCommand;

    match args.subcommand {
        SubCommand::Get { arguments, executable, exit_code } => {
            command = parse_input(arguments, executable, exit_code, stdin);
        }
        SubCommand::Set { arguments, executable, exit_code } => {
            command = parse_input(arguments, executable, exit_code, stdin);
            let (exit_code, stdout, stderr) = invoke_command(command.executable.as_ref(), command.arguments.clone());
            // TODO: convert this to tracing json once other PR is merged to handle tracing from resources
            eprintln!("Stdout: {}", stdout);
            eprintln!("Stderr: {}", stderr);
            command.exit_code = exit_code;
        }
    }

    println!("{}", command.to_json());
}
