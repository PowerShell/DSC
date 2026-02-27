// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand};
use rust_i18n::t;

#[derive(Debug, Parser)]
#[clap(name = "xtask", about = t!("args.about").to_string(), long_about = None)]
pub struct Args {
    /// The subcommand to run
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(name = "schema", about = t!("args.schemaAbout").to_string())]
    Schema {
        #[clap(subcommand)]
        sub_command: SchemaSubCommand
    },
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SchemaSubCommand {
    #[clap(name = "export", about = t!("args.schemaExportAbout").to_string())]
    Export
}
