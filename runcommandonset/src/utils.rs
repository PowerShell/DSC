// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{io::Read, process::{Command, exit, Stdio}};
use tracing::{Level, error, debug};
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
                error!("Error: Input is not valid: {err}");
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
        error!("Error: Executable is required when input is not provided via stdin");
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
        TraceLevel::Warning => Level::WARN,
        TraceLevel::Info => Level::INFO,
        TraceLevel::Debug => Level::DEBUG,
        TraceLevel::Trace => Level::TRACE,
    };

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("warning"))
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
        eprintln!("Unable to set global default tracing subscriber.  Tracing is diabled.");
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
    debug!("Invoking command {} with args {:?}", executable, args);
    let mut command = Command::new(executable);

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    if let Some(args) = args {
        command.args(args);
    }

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(e) => {
            error!("Failed to execute {}: {e}", executable);
            exit(EXIT_DSC_ERROR);
        }
    };

    let Some(mut child_stdout) = child.stdout.take() else {
        error!("Failed to open stdout for {}", executable);
        exit(EXIT_DSC_ERROR);
    };
    let mut stdout_buf = Vec::new();
    match child_stdout.read_to_end(&mut stdout_buf) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to read stdout for {}: {e}", executable);
            exit(EXIT_DSC_ERROR);
        }
    }

    let Some(mut child_stderr) = child.stderr.take() else {
        error!("Failed to open stderr for {}", executable);
        exit(EXIT_DSC_ERROR);
    };
    let mut stderr_buf = Vec::new();
    match child_stderr.read_to_end(&mut stderr_buf) {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to read stderr for {}: {e}", executable);
            exit(EXIT_DSC_ERROR);
        }
    }

    let exit_status = match child.wait() {
        Ok(exit_status) => exit_status,
        Err(e) => {
            error!("Failed to wait for {}: {e}", executable);
            exit(EXIT_DSC_ERROR);
        }
    };

    let exit_code = exit_status.code().unwrap_or(EXIT_PROCESS_TERMINATED);
    let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
    let stderr = String::from_utf8_lossy(&stderr_buf).to_string();
    (exit_code, stdout, stderr)
}
