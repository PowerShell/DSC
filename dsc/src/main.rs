// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use args::{Args, SubCommand};
use atty::Stream;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use std::io::{self, Read};
use std::process::exit;
use tracing::{error, info};
use sysinfo::{Process, ProcessExt, RefreshKind, System, SystemExt, get_current_pid, ProcessRefreshKind};

#[cfg(debug_assertions)]
use crossterm::event;
#[cfg(debug_assertions)]
use std::env;

pub mod args;
pub mod resource_command;
pub mod subcommand;
pub mod tablewriter;
pub mod util;

fn main() {
    #[cfg(debug_assertions)]
    check_debug();

    // create subscriber that writes all events to stderr
    let subscriber = tracing_subscriber::fmt().pretty().with_writer(std::io::stderr).finish();
    let _ = tracing::subscriber::set_global_default(subscriber).map_err(|_err| eprintln!("Unable to set global default subscriber"));

    if ctrlc::set_handler(ctrlc_handler).is_err() {
        error!("Error: Failed to set Ctrl-C handler");
    }

    let args = Args::parse();

    let stdin: Option<String> = if atty::is(Stream::Stdin) {
        None
    } else {
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        let input = match String::from_utf8(buffer) {
            Ok(input) => input,
            Err(e) => {
                error!("Invalid UTF-8 sequence: {e}");
                exit(util::EXIT_INVALID_ARGS);
            },
        };
        Some(input)
    };

    match args.subcommand {
        SubCommand::Completer { shell } => {
            info!("Generating completion script for {:?}", shell);
            let mut cmd = Args::command();
            generate(shell, &mut cmd, "dsc", &mut io::stdout());
        },
        SubCommand::Config { subcommand } => {
            subcommand::config(&subcommand, &args.format, &stdin);
        },
        SubCommand::Resource { subcommand } => {
            subcommand::resource(&subcommand, &args.format, &stdin);
        },
        SubCommand::Schema { dsc_type } => {
            let schema = util::get_schema(dsc_type);
            let json = match serde_json::to_string(&schema) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON Error: {err}");
                    exit(util::EXIT_JSON_ERROR);
                }
            };
            util::write_output(&json, &args.format);
        },
    }

    exit(util::EXIT_SUCCESS);
}

fn ctrlc_handler() {
    error!("Ctrl-C received");

    // get process tree for current process and terminate all processes
    let sys = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
    let Ok(current_pid) = get_current_pid() else {
        eprintln!("Could not get current process id");
        exit(util::EXIT_CTRL_C);
    };
    let Some(current_process) = sys.process(current_pid) else {
        eprintln!("Could not get current process");
        exit(util::EXIT_CTRL_C);
    };

    terminate_subprocesses(&sys, current_process);
    exit(util::EXIT_CTRL_C);
}

fn terminate_subprocesses(sys: &System, process: &Process) {
    for subprocess in sys.processes().values().filter(|p| p.parent().map_or(false, |parent| parent == process.pid())) {
        terminate_subprocesses(sys, subprocess);
    }

    process.kill();
}

#[cfg(debug_assertions)]
fn check_debug() {
    if env::var("DEBUG_DSC").is_ok() {
        error!("attach debugger to pid {} and press a key to continue", std::process::id());
        loop {
            let event = event::read().unwrap();
            if let event::Event::Key(key) = event {
                // workaround bug in 0.26+ https://github.com/crossterm-rs/crossterm/issues/752#issuecomment-1414909095
                if key.kind == event::KeyEventKind::Press {
                    break;
                }
            } else {
                error!("Unexpected event: {event:?}");
                continue;
            }
        }
    }
}
