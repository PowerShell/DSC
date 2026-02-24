// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum Schemas {
    Adapter,
    CopyResource,
    Delete,
    Exist,
    ExitCode,
    Export,
    Exporter,
    Get,
    InDesiredState,
    Metadata,
    Operation,
    Sleep,
    Trace,
    Version,
    WhatIf,
    WhatIfDelete
}

#[derive(Debug, Parser)]
#[clap(name = "dscrtest", version = "0.1.0", about = "Test resource", long_about = None)]
pub struct Args {
    /// The subcommand to run
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum AdapterOperation {
    Get,
    Set,
    Test,
    List,
    Export,
    Validate,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(name = "adapter", about = "Resource adapter")]
    Adapter {
        #[clap(name = "input", short, long, help = "The input to the adapter command as JSON")]
        input: String,
        #[clap(name = "resource-type", long, help = "The resource type to adapt to")]
        resource_type: String,
        #[clap(name = "resource-path", long, help = "The path to the adapted resource")]
        resource_path: Option<String>,
        #[clap(name = "operation", short, long, help = "The operation to perform")]
        operation: AdapterOperation,
    },

    #[clap(name = "copy-resource", about = "Copy a resource")]
    CopyResource {
        #[clap(name = "input", short, long, help = "The input to the copy resource command as JSON")]
        input: String,
    },

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

    #[clap(name = "export", about = "Export instances")]
    Export {
        #[clap(name = "input", short, long, help = "The input to the export command as JSON")]
        input: String,
    },

    #[clap(name = "exporter", about = "Exports different types of resources")]
    Exporter {
        #[clap(name = "input", short, long, help = "The input to the exporter command as JSON")]
        input: String,
    },

    #[clap(name = "get", about = "Get a resource")]
    Get {
        #[clap(name = "input", short, long, help = "The input to the get command as JSON")]
        input: String,
    },

    #[clap(name = "in-desired-state", about = "Specify if the resource is in the desired state")]
    InDesiredState {
        #[clap(name = "input", short, long, help = "The input to the in desired state command as JSON")]
        input: String,
    },

    #[clap(name = "metadata", about = "Return the metadata")]
    Metadata {
        #[clap(name = "input", short, long, help = "The input to the metadata command as JSON")]
        input: String,
        #[clap(name = "export", short, long, help = "Use export operation")]
        export: bool,
    },

    #[clap(name = "no-op", about = "Perform no operation, just return success")]
    NoOp,

    #[clap(name = "operation", about = "Perform an operation")]
    Operation {
        #[clap(name = "operation", short, long, help = "The name of the operation to perform")]
        operation: String,
        #[clap(name = "input", short, long, help = "The input to the operation command as JSON")]
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

    #[clap(name = "version", about = "Test multiple versions of same resource")]
    Version {
        version: String,
    },

    #[clap(name = "whatif", about = "Check if it is a whatif operation")]
    WhatIf {
        #[clap(name = "whatif", short, long, help = "Run as a whatif executionType instead of actual executionType")]
        what_if: bool,
    },

    #[clap(name = "whatif-delete", about = "Check if it is a whatif delete operation")]
    WhatIfDelete {
        #[clap(name = "whatif", short, long, help = "Run as a whatif executionType instead of actual executionType")]
        what_if: bool,
    }
}
