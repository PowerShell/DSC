// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "testResource", version = "0.1.0", about = "This resource returns test data.", long_about = None)]
pub struct Args {
    /// The subcommand to run
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(name = "list", about = "Returns some test resources.")]
    List,
    #[clap(name = "listmissingrequires", about = "Returns some test resources with invalid schema.")]
    ListMissingRequires,
}
