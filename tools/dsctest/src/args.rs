// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum Schemas {
    Delete,
    Exist,
    ExitCode,
    InDesiredState,
    Export,
    ExportBubble,
    Exporter,
    Sleep,
    Trace,
    WhatIf,
}

#[derive(Debug, Parser)]
#[clap(name = "dscrtest", version = "0.1.0", about = "Test resource", long_about = None)]
pub struct Args {
    /// The subcommand to run
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(name = "delete", about = "delete operation")]
    Delete {
        #[clap(name = "input", short, long, help = "The input to the delete command as JSON")]
        input: String,
    },

    #[clap(name = "exist", about = "Check if a resource exists")]
    Exist {
        #[clap(name = "input", short, long, help = "The input to the exist command as JSON")]
        input: String,
    },

    #[clap(name = "exit-code", about = "Return the exit code specified in the input")]
    ExitCode {
        #[clap(name = "input", short, long, help = "The input to the exit code command as JSON")]
        input: String,
    },

    #[clap(name = "in-desired-state", about = "Specify if the resource is in the desired state")]
    InDesiredState {
        #[clap(name = "input", short, long, help = "The input to the in desired state command as JSON")]
        input: String,
    },
    #[clap(name = "export", about = "Export instances")]
    Export {
        #[clap(name = "input", short, long, help = "The input to the export command as JSON")]
        input: String,
    },

    #[clap(name = "export-bubble", about = "Export properties that DSC will bubble up")]
    ExportBubble,

    #[clap(name = "exporter", about = "Exports different types of resources")]
    Exporter {
        #[clap(name = "input", short, long, help = "The input to the exporter command as JSON")]
        input: String,
    },

    #[clap(name = "schema", about = "Get the JSON schema for a subcommand")]
    Schema {
        #[clap(name = "subcommand", short, long, help = "The subcommand to get the schema for")]
        subcommand: Schemas,
    },

    #[clap(name = "sleep", about = "Sleep for a specified number of seconds")]
    Sleep {
        #[clap(name = "input", short, long, help = "The input to the sleep command as JSON")]
        input: String,
    },

    #[clap(name = "trace", about = "The trace level")]
    Trace,

    #[clap(name = "whatif", about = "Check if it is a whatif operation")]
    WhatIf {
        #[clap(name = "whatif", short, long, help = "Run as a whatif executionType instead of actual executionType")]
        what_if: bool,
    }
}
