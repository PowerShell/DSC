// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(name = "file", version = "1.0", about = "Manage state of a file on disk.", long_about = None)]

pub struct Args {
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(name = "get", about = "Get the current state of the file.", arg_required_else_help = true)]
    Get {
        #[clap(short, long, required = true, help = "The path to the file.")]
        input: String,
    },

    #[clap(name = "delete", about = "Delete the file on disk.", arg_required_else_help = true)]
    Delete {
        #[clap(short, long, required = true, help = "The path to the file.")]
        input: String,
    },

    // #[clap(name = "export", about = "Exports the files and directories under the specified path", arg_required_else_help = true)]
    // Export {
    //     #[clap(short, long, required = true, help = "The path to the file or directory.")]
    //     path: String,
    // },
    #[clap(name = "schema", about = "Retrieve JSON schema.")]
    Schema,
}