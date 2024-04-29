// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use args::{Args, SubCommand};
use atty::Stream;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use dsc_lib::configure::config_doc::Configuration;
use crate::include::Include;
use std::io::{self, Read};
use std::fs::File;
use std::path::Path;
use std::process::exit;
use sysinfo::{Process, ProcessExt, RefreshKind, System, SystemExt, get_current_pid, ProcessRefreshKind};
use tracing::{error, info, warn, debug};

#[cfg(debug_assertions)]
use crossterm::event;
#[cfg(debug_assertions)]
use std::env;

pub mod args;
pub mod include;
pub mod resource_command;
pub mod subcommand;
pub mod tablewriter;
pub mod util;

fn main() {
    #[cfg(debug_assertions)]
    check_debug();

    if ctrlc::set_handler(ctrlc_handler).is_err() {
        error!("Error: Failed to set Ctrl-C handler");
    }

    let args = Args::parse();

    util::enable_tracing(&args.trace_level, &args.trace_format);

    debug!("Running dsc {}", env!("CARGO_PKG_VERSION"));

    let mut input = if atty::is(Stream::Stdin) {
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
        SubCommand::Config { subcommand, parameters, parameters_file, as_group, as_include} => {
            if as_include {
                input = Some(read_include_file(&input));
            }

            if let Some(file_name) = parameters_file {
                info!("Reading parameters from file {}", file_name);
                match std::fs::read_to_string(file_name) {
                    Ok(parameters) => subcommand::config(&subcommand, &Some(parameters), &input, &as_group, &as_include),
                    Err(err) => {
                        error!("Error: Failed to read parameters file: {err}");
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

fn read_include_file(input: &Option<String>) -> String {
    let Some(include) = input else {
        error!("Error: Include requires input from STDIN");
        exit(util::EXIT_INVALID_INPUT);
    };

    // deserialize the Include input
    let include: Include = match serde_json::from_str(include) {
        Ok(include) => include,
        Err(err) => {
            error!("Error: Failed to deserialize Include input: {err}");
            exit(util::EXIT_INVALID_INPUT);
        }
    };

    let path = Path::new(&include.configuration_file);
    if path.is_absolute() {
        error!("Error: Include path must be relative: {}", include.configuration_file);
        exit(util::EXIT_INVALID_INPUT);
    }

    // check that no components of the path are '..'
    if path.components().any(|c| c == std::path::Component::ParentDir) {
        error!("Error: Include path must not contain '..': {}", include.configuration_file);
        exit(util::EXIT_INVALID_INPUT);
    }

    // use DSC_CONFIG_ROOT env var as current directory
    let current_directory = match std::env::var("DSC_CONFIG_ROOT") {
        Ok(current_directory) => current_directory,
        Err(err) => {
            error!("Error: Could not read DSC_CONFIG_ROOT env var: {err}");
            exit(util::EXIT_INVALID_INPUT);
        }
    };

    // combine the current directory with the Include path
    let include_path = Path::new(&current_directory).join(&include.configuration_file);

    // read the file specified in the Include input
    let mut buffer: Vec<u8> = Vec::new();
    match File::open(&include_path) {
        Ok(mut file) => {
            match file.read_to_end(&mut buffer) {
                Ok(_) => (),
                Err(err) => {
                    error!("Error: Failed to read file '{include_path:?}': {err}");
                    exit(util::EXIT_INVALID_INPUT);
                }
            }
        },
        Err(err) => {
            error!("Error: Failed to included file '{include_path:?}': {err}");
            exit(util::EXIT_INVALID_INPUT);
        }
    }
    // convert the buffer to a string
    let include_content = match String::from_utf8(buffer) {
        Ok(input) => input,
        Err(err) => {
            error!("Error: Invalid UTF-8 sequence in included file '{include_path:?}': {err}");
            exit(util::EXIT_INVALID_INPUT);
        }
    };

    // try to deserialize the Include content as YAML first
    let configuration: Configuration = match serde_yaml::from_str(&include_content) {
        Ok(configuration) => configuration,
        Err(_err) => {
            // if that fails, try to deserialize it as JSON
            match serde_json::from_str(&include_content) {
                Ok(configuration) => configuration,
                Err(err) => {
                    error!("Error: Failed to read the configuration file '{include_path:?}' as YAML or JSON: {err}");
                    exit(util::EXIT_INVALID_INPUT);
                }
            }
        }
    };

    // serialize the Configuration as JSON
    match serde_json::to_string(&configuration) {
        Ok(json) => json,
        Err(err) => {
            error!("Error: JSON Error: {err}");
            exit(util::EXIT_JSON_ERROR);
        }
    }
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
    info!("Terminating subprocesses of process {} {}", process.name(), process.pid());
    for subprocess in sys.processes().values().filter(|p| p.parent().map_or(false, |parent| parent == process.pid())) {
        terminate_subprocesses(sys, subprocess);
    }

    info!("Terminating process {} {}", process.name(), process.pid());
    if !process.kill() {
        error!("Failed to terminate process {} {}", process.name(), process.pid());
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
