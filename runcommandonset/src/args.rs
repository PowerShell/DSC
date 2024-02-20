// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum TraceFormat {
    Default,
    Plaintext,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum TraceLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace
}

#[derive(Parser)]
#[clap(name = "runcommandonset", version = "0.0.1", about = "Run a command on set", long_about = None)]
pub struct Arguments {

    #[clap(subcommand)]
    pub subcommand: SubCommand,
    #[clap(short = 'l', long, help = "Trace level to use", value_enum, default_value = "info")]
    pub trace_level: TraceLevel,
    #[clap(short = 'f', long, help = "Trace format to use", value_enum, default_value = "default")]
    pub trace_format: TraceFormat,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(name = "get", about = "Get formatted command to run on set.")]
    Get {
        #[clap(short = 'a', long, help = "The arguments to pass to the executable.")]
        arguments: Option<Vec<String>>,
        #[clap(short = 'e', long, help = "The executable to run.", default_value = "")]
        executable: String,
        #[clap(short = 'c', long, help = "The expected exit code to indicate success, if non-zero.", default_value = "0")]
        exit_code: i32,
    },
    #[clap(name = "set", about = "Run formatted command.")]
    Set {
        #[clap(short = 'a', long, help = "The arguments to pass to the executable.")]
        arguments: Option<Vec<String>>,
        #[clap(short = 'e', long, help = "The executable to run.", default_value = "")]
        executable: String,
        #[clap(short = 'c', long, help = "The expected exit code to indicate success, if non-zero.", default_value = "0")]
        exit_code: i32,
    }
}
