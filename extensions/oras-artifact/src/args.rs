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
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Parser)]
#[clap(
    name = "oras-artifact-discover",
    version = "0.1.0",
    about = "Discovers DSC resources from OCI-compliant registries using ORAS conventions.",
    long_about = None
)]
pub struct Arguments {
    #[clap(subcommand)]
    pub subcommand: SubCommand,

    #[clap(
        short = 'l',
        long,
        help = "Trace level to use",
        value_enum,
        default_value = "warn"
    )]
    pub trace_level: Option<TraceLevel>,

    #[clap(
        short = 'f',
        long,
        help = "Trace format to use",
        value_enum,
        default_value = "json"
    )]
    pub trace_format: TraceFormat,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(
        name = "discover",
        about = "Discover DSC artifacts in configured OCI registries. Reads JSON configuration from STDIN."
    )]
    Discover,
}
