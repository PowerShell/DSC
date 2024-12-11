// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use std::{io::Read, process::{Command, exit, Stdio}};
use tracing::{Level, error, debug, trace};
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, Layer};

use crate::args::{TraceFormat, TraceLevel};
use crate::runcommand;

pub const EXIT_INVALID_ARGS: i32 = 1;
pub const EXIT_DSC_ERROR: i32 = 2;
pub const EXIT_CODE_MISMATCH: i32 = 3;
pub const EXIT_INVALID_INPUT: i32 = 4;
pub const EXIT_PROCESS_TERMINATED: i32 = 5;

/// Initialize `RunCommand` struct from input provided via stdin or via CLI arguments.
///
/// # Arguments
///
/// * `arguments` - Optional arguments to pass to the command
/// * `executable` - The command to execute
/// * `exit_code` - The expected exit code upon success, if non-zero
/// * `stdin` - Optional JSON or YAML input provided via stdin
///
/// # Errors
///
/// Error message then exit if the `RunCommand` struct cannot be initialized from the provided inputs.
pub fn parse_input(arguments: Option<Vec<String>>, executable: Option<String>, exit_code: i32, stdin: Option<String>) -> runcommand::RunCommand {
    let command: runcommand::RunCommand;
    if let Some(input) = stdin {
        debug!("Input: {}", input);
        command = match serde_json::from_str(&input) {
            Ok(json) => json,
            Err(err) => {
                error!("{}: {err}", t!("utils.invalidInput"));
                exit(EXIT_INVALID_INPUT);
            }
        }
    } else if let Some(executable) = executable {
        command = runcommand::RunCommand {
            executable,
            arguments,
            exit_code,
        };
    }
    else {
        error!("{}", t!("utils.executableRequired"));
        exit(EXIT_INVALID_INPUT);
    }
    command
}

/// Setup tracing subscriber based on the provided trace level and format.
///
/// # Arguments
///
/// * `trace_level` - The level of information of to output
/// * `trace_format` - The format of the output
///
/// # Errors
///
/// If unable to initialize the tracing subscriber, an error message is printed and tracing is disabled.
pub fn enable_tracing(trace_level: &TraceLevel, trace_format: &TraceFormat) {
    // originally implemented in dsc/src/util.rs
    let tracing_level = match trace_level {
        TraceLevel::Error => Level::ERROR,
        TraceLevel::Warn => Level::WARN,
        TraceLevel::Info => Level::INFO,
        TraceLevel::Debug => Level::DEBUG,
        TraceLevel::Trace => Level::TRACE,
    };

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("warn"))
        .unwrap_or_default()
        .add_directive(tracing_level.into());
    let layer = tracing_subscriber::fmt::Layer::default().with_writer(std::io::stderr);
    let fmt = match trace_format {
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
                .with_level(true)
                .with_line_number(false)
                .boxed()
        },
        TraceFormat::Json => {
            layer
                .with_ansi(false)
                .with_level(true)
                .with_line_number(true)
                .json()
                .boxed()
        }
    };

    let subscriber = tracing_subscriber::Registry::default().with(fmt).with(filter);

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        eprintln!("{}", t!("utils.unableToTrace"));
    }
}

/// Invoke a command and return the exit code, stdout, and stderr.
///
/// # Arguments
///
/// * `executable` - The command to execute
/// * `args` - Optional arguments to pass to the command
///
/// # Errors
///
/// Error message then exit if the command fails to execute or stdin/stdout/stderr cannot be opened.
pub fn invoke_command(executable: &str, args: Option<Vec<String>>) -> (i32, String, String) {
    // originally implemented in dsc_lib/src/dscresources/command_resource.rs
    trace!("Invoking command {} with args {:?}", executable, args);
    let mut command = Command::new(executable);

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    if let Some(args) = args {
        command.args(args);
    }

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(e) => {
            error!("{} '{executable}': {e}", t!("utils.failedToExecute"));
            exit(EXIT_DSC_ERROR);
        }
    };

    let Some(mut child_stdout) = child.stdout.take() else {
        error!("{} {executable}", t!("utils.failedOpenStdout"));
        exit(EXIT_DSC_ERROR);
    };
    let mut stdout_buf = Vec::new();
    match child_stdout.read_to_end(&mut stdout_buf) {
        Ok(_) => (),
        Err(e) => {
            error!("{} '{executable}': {e}", t!("utils.failedReadStdout"));
            exit(EXIT_DSC_ERROR);
        }
    }

    let Some(mut child_stderr) = child.stderr.take() else {
        error!("{} {executable}", t!("utils.failedOpenStderr"));
        exit(EXIT_DSC_ERROR);
    };
    let mut stderr_buf = Vec::new();
    match child_stderr.read_to_end(&mut stderr_buf) {
        Ok(_) => (),
        Err(e) => {
            error!("{} '{executable}': {e}", t!("utils.failedReadStderr"));
            exit(EXIT_DSC_ERROR);
        }
    }

    let exit_status = match child.wait() {
        Ok(exit_status) => exit_status,
        Err(e) => {
            error!("{} '{executable}': {e}", t!("utils.failedWait"));
            exit(EXIT_DSC_ERROR);
        }
    };

    let exit_code = exit_status.code().unwrap_or(EXIT_PROCESS_TERMINATED);
    let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
    let stderr = String::from_utf8_lossy(&stderr_buf).to_string();
    (exit_code, stdout, stderr)
}
