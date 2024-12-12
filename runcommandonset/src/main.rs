// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::Parser;
use rust_i18n::{i18n, t};
use std::{io::{self, Read, IsTerminal}, process::exit};
use tracing::{error, warn, debug, trace};

use args::{Arguments, SubCommand, TraceLevel};
use runcommand::RunCommand;
use utils::{enable_tracing, invoke_command, parse_input, EXIT_INVALID_ARGS};

pub mod args;
pub mod runcommand;
pub mod utils;

i18n!("locales", fallback = "en-us");

fn main() {
    let args = Arguments::parse();
    let trace_level = match args.trace_level {
        Some(trace_level) => trace_level,
        None => {
            // get from DSC_TRACE_LEVEL env var
            if let Ok(trace_level) = std::env::var("DSC_TRACE_LEVEL") {
                match trace_level.to_lowercase().as_str() {
                    "error" => TraceLevel::Error,
                    "warn" => TraceLevel::Warn,
                    "info" => TraceLevel::Info,
                    "debug" => TraceLevel::Debug,
                    "trace" => TraceLevel::Trace,
                    _ => {
                        warn!("{}: {trace_level}", t!("main.invalidTraceLevel"));
                        TraceLevel::Info
                    }
                }
            } else {
                // default to info
                TraceLevel::Info
            }
        }
    };
    enable_tracing(&trace_level, &args.trace_format);
    warn!("{}", t!("main.notIdempotent"));

    let stdin = if std::io::stdin().is_terminal() {
        None
    } else {
        debug!("{}", t!("main.readStdin"));
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        let stdin = match String::from_utf8(buffer) {
            Ok(stdin) => stdin,
            Err(e) => {
                error!("{}: {e}", t!("main.invalidUtf8"));
                exit(EXIT_INVALID_ARGS);
            },
        };
        // parse_input expects at most 1 input, so wrapping Some(empty input) would throw it off
        if stdin.is_empty() {
            debug!("{}", t!("main.emptyStdin"));
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
            trace!("Stdout: {stdout}");
            trace!("Stderr: {stderr}");
            command.exit_code = exit_code;
        }
    }

    println!("{}", command.to_json());
}
