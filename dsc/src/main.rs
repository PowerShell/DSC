// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use args::{Args, TraceLevel, TraceFormat, SubCommand};
use atty::Stream;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use std::io::{self, Read};
use std::process::exit;
use sysinfo::{Process, ProcessExt, RefreshKind, System, SystemExt, get_current_pid, ProcessRefreshKind};
use tracing::{Level, error, info, warn, debug};
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, Layer};

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

    if ctrlc::set_handler(ctrlc_handler).is_err() {
        error!("Error: Failed to set Ctrl-C handler");
    }

    let args = Args::parse();

    let tracing_level = match args.trace_level {
        TraceLevel::Error => Level::ERROR,
        TraceLevel::Warning => Level::WARN,
        TraceLevel::Info => Level::INFO,
        TraceLevel::Debug => Level::DEBUG,
        TraceLevel::Trace => Level::TRACE,
    };

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("warning"))
        .unwrap()
        .add_directive(tracing_level.into());
    let layer = tracing_subscriber::fmt::Layer::default().with_writer(std::io::stderr);
    let fmt = match args.trace_format {
        TraceFormat::Default => {
            layer
                .with_ansi(true)
                .with_level(true)
                .with_line_number(true)
                .boxed()
        },
        TraceFormat::Plaintext => {
            layer
                .with_ansi(false)
                .with_level(false)
                .with_line_number(false)
                .boxed()
        },
        TraceFormat::Json => {
            layer
                .with_ansi(true)
                .with_level(false)
                .with_line_number(false)
                .json()
                .boxed()
        }
    };

    let subscriber = tracing_subscriber::Registry::default().with(fmt).with(filter);

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        eprintln!("Unable to set global default logging subscriber");
    }

    debug!("Running dsc {}", env!("CARGO_PKG_VERSION"));

    let input = if args.input.is_some() {
        args.input
    } else if args.input_file.is_some() {
        info!("Reading input from file {}", args.input_file.as_ref().unwrap());
        let input_file = args.input_file.unwrap();
        match std::fs::read_to_string(input_file) {
            Ok(input) => Some(input),
            Err(err) => {
                error!("Error: Failed to read input file: {err}");
                exit(util::EXIT_INVALID_INPUT);
            }
        }
    } else if atty::is(Stream::Stdin) {
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
        Some(input)
    };

    match args.subcommand {
        SubCommand::Completer { shell } => {
            info!("Generating completion script for {:?}", shell);
            let mut cmd = Args::command();
            generate(shell, &mut cmd, "dsc", &mut io::stdout());
        },
        SubCommand::Config { subcommand, parameters, parameters_file } => {
            if let Some(file_name) = parameters_file {
                info!("Reading parameters from file {}", file_name);
                match std::fs::read_to_string(file_name) {
                    Ok(parameters) => subcommand::config(&subcommand, &Some(parameters), &args.format, &input),
                    Err(err) => {
                        error!("Error: Failed to read parameters file: {err}");
                        exit(util::EXIT_INVALID_INPUT);
                    }
                }
            }
            else {
                subcommand::config(&subcommand, &parameters, &args.format, &input);
            }
        },
        SubCommand::Resource { subcommand } => {
            subcommand::resource(&subcommand, &args.format, &input);
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
            let event = event::read().unwrap();
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
