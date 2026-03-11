// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::Parser;
use dsc_lib::schemas::dsc_repo::RecognizedSchemaVersion;
use rust_i18n::i18n;
use thiserror::Error;

use crate::{
    args::{Args, SchemaSubCommand, SubCommand},
    schemas::export::{SchemaExportError, export_schemas}
};

mod args;
pub(crate) mod schemas {
    pub(crate) mod export;
}

#[derive(Debug, Error)]
pub(crate) enum XTaskError {
    #[error(transparent)]
    SchemaExport(#[from] SchemaExportError)
}

i18n!("locales", fallback = "en-us");

fn main() -> Result<(), XTaskError> {
    let args = Args::parse();

    match args.subcommand {
        SubCommand::Schema { sub_command } => match sub_command {
            SchemaSubCommand::Export => {
                export_schemas(RecognizedSchemaVersion::VNext)?;
                Ok(())
            },
        },
    }
}
