// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use args::{Args, SubCommand};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use dsc_lib::progress::ProgressFormat;
use mcp::start_mcp_server;
use rust_i18n::{i18n, t};
use std::{io, io::Read, process::exit};
use sysinfo::{get_current_pid, Process, ProcessRefreshKind, RefreshKind, System};
use tracing::{debug, error, info, warn};

use crate::util::EXIT_INVALID_INPUT;

#[cfg(debug_assertions)]
use crossterm::event;
#[cfg(debug_assertions)]
use std::env;

pub mod args;
pub mod mcp;
pub mod resolve;
pub mod resource_command;
pub mod subcommand;
pub mod tablewriter;
pub mod util;

i18n!("locales", fallback = "en-us");

fn main() {
    #[cfg(debug_assertions)]
    check_debug();

    #[cfg(windows)]
    check_store();

    if ctrlc::set_handler(ctrlc_handler).is_err() {
        error!("{}", t!("main.failedCtrlCHandler"));
    }

    let args = Args::parse();

    util::enable_tracing(args.trace_level.as_ref(), args.trace_format.as_ref());

    debug!("{}: {}", t!("main.usingDscVersion"), env!("CARGO_PKG_VERSION"));

    let progress_format = args.progress_format.unwrap_or(ProgressFormat::Default);

    match args.subcommand {
        SubCommand::Completer { shell } => {
            info!("{} {:?}", t!("main.generatingCompleter"), shell);
            let mut cmd = Args::command();
            generate(shell, &mut cmd, "dsc", &mut io::stdout());
        }
        SubCommand::Config {
            subcommand,
            parameters,
            parameters_file,
            system_root,
            as_group,
            as_assert,
            as_include,
        } => {
            if let Some(file_name) = parameters_file {
                if file_name == "-" {
                    info!("{}", t!("main.readingParametersFromStdin"));
                    let mut stdin = Vec::<u8>::new();
                    let parameters = match io::stdin().read_to_end(&mut stdin) {
                        Ok(_) => match String::from_utf8(stdin) {
                            Ok(input) => input,
                            Err(err) => {
                                error!("{}: {err}", t!("util.invalidUtf8"));
                                exit(EXIT_INVALID_INPUT);
                            }
                        },
                        Err(err) => {
                            error!("{}: {err}", t!("util.failedToReadStdin"));
                            exit(EXIT_INVALID_INPUT);
                        }
                    };
                    subcommand::config(
                        &subcommand,
                        &Some(parameters),
                        true,
                        system_root.as_ref(),
                        &as_group,
                        &as_assert,
                        &as_include,
                        progress_format,
                    );
                    return;
                }
                info!("{}: {file_name}", t!("main.readingParametersFile"));
                match std::fs::read_to_string(&file_name) {
                    Ok(parameters) => subcommand::config(
                        &subcommand,
                        &Some(parameters),
                        false,
                        system_root.as_ref(),
                        &as_group,
                        &as_assert,
                        &as_include,
                        progress_format,
                    ),
                    Err(err) => {
                        error!("{} '{file_name}': {err}", t!("main.failedReadingParametersFile"));
                        exit(util::EXIT_INVALID_INPUT);
                    }
                }
            } else {
                subcommand::config(
                    &subcommand,
                    &parameters,
                    false,
                    system_root.as_ref(),
                    &as_group,
                    &as_assert,
                    &as_include,
                    progress_format,
                );
            }
        }
        SubCommand::Extension { subcommand } => {
            subcommand::extension(&subcommand, progress_format);
        }
        SubCommand::Function { subcommand } => {
            subcommand::function(&subcommand);
        }
        SubCommand::Mcp => {
            if let Err(err) = start_mcp_server() {
                error!("{}", t!("main.failedToStartMcpServer", error = err));
                exit(util::EXIT_MCP_FAILED);
            }
            exit(util::EXIT_SUCCESS);
        }
        SubCommand::Resource { subcommand } => {
            subcommand::resource(&subcommand, progress_format);
        }
        SubCommand::Schema {
            dsc_type,
            output_format,
        } => {
            let schema = util::get_schema(dsc_type);
            let json = match serde_json::to_string(&schema) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON: {err}");
                    exit(util::EXIT_JSON_ERROR);
                }
            };
            util::write_object(&json, output_format.as_ref(), false);
        }
    }

    exit(util::EXIT_SUCCESS);
}

fn ctrlc_handler() {
    warn!("{}", t!("main.ctrlCReceived"));

    // get process tree for current process and terminate all processes
    let sys = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));
    info!("{}: {}", t!("main.foundProcesses"), sys.processes().len());
    let Ok(current_pid) = get_current_pid() else {
        error!("{}", t!("main.failedToGetPid"));
        exit(util::EXIT_CTRL_C);
    };
    info!("{}: {}", t!("main.currentPid"), current_pid);
    let Some(current_process) = sys.process(current_pid) else {
        error!("{}", t!("main.failedToGetProcess"));
        exit(util::EXIT_CTRL_C);
    };

    terminate_subprocesses(&sys, current_process);
    exit(util::EXIT_CTRL_C);
}

fn terminate_subprocesses(sys: &System, process: &Process) {
    info!(
        "{}: {:?} {}",
        t!("main.terminatingSubprocess"),
        process.name(),
        process.pid()
    );
    for subprocess in sys
        .processes()
        .values()
        .filter(|p| p.parent().is_some_and(|parent| parent == process.pid()))
    {
        terminate_subprocesses(sys, subprocess);
    }

    info!(
        "{}: {:?} {}",
        t!("main.terminatingProcess"),
        process.name(),
        process.pid()
    );
    if !process.kill() {
        error!(
            "{}: {:?} {}",
            t!("main.failedTerminatingProcess"),
            process.name(),
            process.pid()
        );
    }
}

#[cfg(debug_assertions)]
fn check_debug() {
    if env::var("DEBUG_DSC").is_ok() {
        eprintln!(
            "attach debugger to pid {} and press a key to continue",
            std::process::id()
        );
        loop {
            let event = match event::read() {
                Ok(event) => event,
                Err(err) => {
                    eprintln!("Failed to read event: {err}");
                    break;
                }
            };
            if let event::Event::Key(key) = event {
                // workaround bug in 0.26+ https://github.com/crossterm-rs/crossterm/issues/752#issuecomment-1414909095
                if key.kind == event::KeyEventKind::Press {
                    break;
                }
            }
        }
    }
}

// Check if the dsc binary parent process is WinStore.App or Explorer.exe
#[cfg(windows)]
fn check_store() {
    use std::io::Read;

    let sys = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));
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
    if parent_process.name().eq_ignore_ascii_case("sihost.exe")
        || parent_process.name().eq_ignore_ascii_case("explorer.exe")
    {
        eprintln!("{}", t!("main.storeMessage"));
        // wait for keypress
        let _ = io::stdin().read(&mut [0u8]).unwrap();
        exit(util::EXIT_INVALID_ARGS);
    }
}
