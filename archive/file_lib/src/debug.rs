// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crossterm::event;
use std::{env, io};

pub const DEBUG_ENV_VAR: &str = "DSC_DEBUG";

/// If the `DSC_DEBUG` environment variable is set, and it contains the given command,
/// then prompt the user to attach a debugger.
///
/// The environment variable is a comma-separated list of case-insensitive commands to enable for
/// prompting to attach the debugger.
///
/// # Arguments
///
/// * `command` - The command to check for in the `DEBUG_DSC` environment variable.
///
/// # Errors
///
/// Will return `Err` if there is an error reading the keyboard.
pub fn check_debugger_prompt(command: &str) -> io::Result<()> {
    let dsc_debug = env::var(DEBUG_ENV_VAR).unwrap_or_default().to_lowercase();
    let debug_args: Vec<&str> = dsc_debug.split(',').collect();

    for arg in &debug_args {
        if arg.trim() == command.to_lowercase() {
            eprintln!(
                "attach debugger to pid {} and press any key to continue",
                std::process::id()
            );

            wait_for_keypress()?;
        }
    }

    Ok(())
}

fn wait_for_keypress() -> io::Result<()> {
    // there will be a latent `Release` on the `Enter` key sitting in the buffer; ignore it
    let mut ignore_first_release = true;
    loop {
        let event = event::read()?;
        if let event::Event::Key(key) = event {
            if key.code == event::KeyCode::Enter
                && key.kind == event::KeyEventKind::Release
                && ignore_first_release
            {
                ignore_first_release = false;
                continue;
            }

            break;
        }

        eprintln!("Unexpected event: {event:?}");
    }

    Ok(())
}
