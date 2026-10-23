// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! `dsclint` checks DSC configuration documents for problems without applying them.

use args::Args;
use clap::Parser;
use dsc_lib::util::DSC_IGNORE_SETTINGS_FILE;
use rust_i18n::{i18n, t};
use std::{env::set_var, process::ExitCode};
use tracing::debug;

mod args;
mod lint;
mod report;
mod tablewriter;
mod util;

i18n!("locales", fallback = "en-us");

fn main() -> ExitCode {
    #[cfg(windows)]
    {
        let handle = match std::thread::Builder::new()
            .name("dsclint-main".to_string())
            .stack_size(2 * 1024 * 1024) // Default stack is too small on Windows causing overflow
            .spawn(lint_main)
        {
            Ok(handle) => handle,
            Err(err) => {
                eprintln!("{}", t!("main.failedToSpawnMain", error = err));
                return ExitCode::from(util::EXIT_DSC_ERROR);
            }
        };

        match handle.join() {
            Ok(Ok(())) => ExitCode::from(util::EXIT_SUCCESS),
            Ok(Err(code)) => code,
            Err(err) => {
                eprintln!("{}", t!("main.failedToJoinMain", error = err : {:?}));
                ExitCode::from(util::EXIT_DSC_ERROR)
            }
        }
    }
    #[cfg(not(windows))]
    {
        match lint_main() {
            Ok(()) => ExitCode::from(util::EXIT_SUCCESS),
            Err(code) => code,
        }
    }
}

fn lint_main() -> Result<(), ExitCode> {
    let args = Args::parse();

    // the settings file is read by `dsc-lib`, which honors this environment variable
    if args.ignore_settings_file {
        unsafe {
            set_var(DSC_IGNORE_SETTINGS_FILE, "1");
        }
    }

    util::enable_tracing(args.trace_level.as_ref(), args.trace_format.as_ref());

    debug!("{}: {}", t!("main.usingVersion"), env!("CARGO_PKG_VERSION"));

    lint::run(&args)
}
