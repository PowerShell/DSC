// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand};
use dsc_lib::schemas::dsc_repo::RecognizedSchemaVersion;
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
    Export {
        /// The schema version folder(s) to export. Repeatable. Defaults to `vNext`.
        #[clap(long = "schema-version", help = t!("args.schemaExportVersionHelp").to_string())]
        schema_versions: Vec<RecognizedSchemaVersion>,
        /// A release version that expands to its patch, minor, and major version folders.
        #[clap(long = "release", conflicts_with = "schema_versions", help = t!("args.schemaExportReleaseHelp").to_string())]
        release: Option<String>,
    }
}
