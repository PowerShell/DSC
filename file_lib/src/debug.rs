// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crossterm::event;
use std::env;

pub const DEBUG_ENV_VAR: &str = "DEBUG_DSC";

/// If the `DEBUG_DSC` environment variable is set, and it contains the given command,
/// then prompt the user to attach a debugger.
/// 
/// The environment variable is a comma-separated list of case-insensitive commands to enable for
/// prompting to attach the debugger.
/// 
/// # Arguments
/// 
/// * `command` - The command to check for in the `DEBUG_DSC` environment variable.
/// 
pub fn check_debug(command: &String) {
    if env::var("DEBUG_DSC").is_ok() {
        let debug_args: Vec<String> = env::var(DEBUG_ENV_VAR)
            .unwrap()
            .split(',')
            .map(|s| s.to_lowercase())
            .collect();

        if debug_args.contains(command) {
            eprintln!(
                "attach debugger to pid {} and press any key to continue",
                std::process::id()
            );
            loop {
                let event = event::read().unwrap();
                if let event::Event::Key(_key) = event {
                    break;
                }
                eprintln!("Unexpected event: {event:?}");
            }
        }
    }
}
