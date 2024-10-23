// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use args::{Args, SubCommand};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use std::io::{self, IsTerminal, Read};
use std::process::exit;
use sysinfo::{Process, RefreshKind, System, get_current_pid, ProcessRefreshKind};
use tracing::{error, info, warn, debug};

#[cfg(debug_assertions)]
use crossterm::event;
#[cfg(debug_assertions)]
use std::env;

pub mod args;
pub mod resolve;
pub mod resource_command;
pub mod subcommand;
pub mod tablewriter;
pub mod util;

fn main() {
    #[cfg(debug_assertions)]
    check_debug();

    #[cfg(windows)]
    check_store();

    if ctrlc::set_handler(ctrlc_handler).is_err() {
        error!("Error: Failed to set Ctrl-C handler");
    }

    let args = Args::parse();

    util::enable_tracing(&args.trace_level, &args.trace_format);

    debug!("Running dsc {}", env!("CARGO_PKG_VERSION"));

    let input = if io::stdin().is_terminal() {
        None
    } else {
        info!("Reading input from STDIN");
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        let input = match String::from_utf8(buffer) {
            Ok(input) => input,
            Err(e) => {
                error!("Invalid UTF-8 sequence: {e}");
                exit(util::EXIT_INVALID_ARGS);
            },
        };
        // get_input call expects at most 1 input, so wrapping Some(empty input) would throw it off
        // have only seen this happen with dsc_args.test.ps1 running on the CI pipeline
        if input.is_empty() {
            debug!("Input from STDIN is empty");
            None
        }
        else {
            Some(input)
        }
    };

    match args.subcommand {
        SubCommand::Completer { shell } => {
            info!("Generating completion script for {:?}", shell);
            let mut cmd = Args::command();
            generate(shell, &mut cmd, "dsc", &mut io::stdout());
        },
        SubCommand::Config { subcommand, parameters, parameters_file, as_group, as_include } => {
            if let Some(file_name) = parameters_file {
                info!("Reading parameters from file {file_name}");
                match std::fs::read_to_string(&file_name) {
                    Ok(parameters) => subcommand::config(&subcommand, &Some(parameters), &input, &as_group, &as_include),
                    Err(err) => {
                        error!("Error: Failed to read parameters file '{file_name}': {err}");
                        exit(util::EXIT_INVALID_INPUT);
                    }
                }
            }
            else {
                subcommand::config(&subcommand, &parameters, &input, &as_group, &as_include);
            }
        },
        SubCommand::Resource { subcommand } => {
            subcommand::resource(&subcommand, &input);
        },
        SubCommand::Schema { dsc_type , format } => {
            let schema = util::get_schema(dsc_type);
            let json = match serde_json::to_string(&schema) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON Error: {err}");
                    exit(util::EXIT_JSON_ERROR);
                }
            };
            util::write_output(&json, &format);
        },
    }

    exit(util::EXIT_SUCCESS);
}

fn ctrlc_handler() {
    warn!("Ctrl-C received");

    // get process tree for current process and terminate all processes
    let sys = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
    info!("Found {} processes", sys.processes().len());
    let Ok(current_pid) = get_current_pid() else {
        error!("Could not get current process id");
        exit(util::EXIT_CTRL_C);
    };
    info!("Current process id: {}", current_pid);
    let Some(current_process) = sys.process(current_pid) else {
        error!("Could not get current process");
        exit(util::EXIT_CTRL_C);
    };

    terminate_subprocesses(&sys, current_process);
    exit(util::EXIT_CTRL_C);
}

fn terminate_subprocesses(sys: &System, process: &Process) {
    info!("Terminating subprocesses of process {:?} {}", process.name(), process.pid());
    for subprocess in sys.processes().values().filter(|p| p.parent().map_or(false, |parent| parent == process.pid())) {
        terminate_subprocesses(sys, subprocess);
    }

    info!("Terminating process {:?} {}", process.name(), process.pid());
    if !process.kill() {
        error!("Failed to terminate process {:?} {}", process.name(), process.pid());
    }
}

#[cfg(debug_assertions)]
fn check_debug() {
    if env::var("DEBUG_DSC").is_ok() {
        eprintln!("attach debugger to pid {} and press a key to continue", std::process::id());
        loop {
            let event = match event::read() {
                Ok(event) => event,
                Err(err) => {
                    eprintln!("Error: Failed to read event: {err}");
                    break;
                }
            };
            if let event::Event::Key(key) = event {
                // workaround bug in 0.26+ https://github.com/crossterm-rs/crossterm/issues/752#issuecomment-1414909095
                if key.kind == event::KeyEventKind::Press {
                    break;
                }
            } else {
                eprintln!("Unexpected event: {event:?}");
                continue;
            }
        }
    }
}

// Check if the dsc binary parent process is WinStore.App or Exploerer.exe
#[cfg(windows)]
fn check_store() {
    let message = r"
DSC.exe is a command-line tool and cannot be run directly from the Windows Store or Explorer.
Visit https://aka.ms/dscv3-docs for more information on how to use DSC.exe.

Press any key to close this window
";
    let sys = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
    // get current process
    let Ok(current_pid) = get_current_pid() else {
        return;
    };

    // get parent process
    let Some(current_process) = sys.process(current_pid) else {
        return;
    };
    let Some(parent_process_pid) = current_process.parent() else {
        return;
    };
    let Some(parent_process) = sys.process(parent_process_pid) else {
        return;
    };

    // MS Store runs app using `sihost.exe`
    if parent_process.name().to_ascii_lowercase() == "sihost.exe" || parent_process.name().to_ascii_lowercase() == "explorer.exe"{
        eprintln!("{message}");
        // wait for keypress
        let _ = io::stdin().read(&mut [0u8]).unwrap();
        exit(util::EXIT_INVALID_ARGS);
    }
}
