// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum Schemas {
    Sleep,
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
    #[clap(name = "schema", about = "Get the JSON schema for a subcommand")]
    Schema {
        #[clap(name = "subcommand", short, long, help = "The subcommand to get the schema for")]
        subcommand: Schemas,
    },

    #[clap(name = "sleep", about = "Sleep for a specified number of seconds")]
    Sleep {
        #[clap(name = "input", short, long, help = "The input to the sleep command")]
        input: String,
    },
}
