// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Get default shell, eventually to be used for `sshd_config` and repeatable keywords
    Get,
    /// Set default shell, eventually to be used for `sshd_config` and repeatable keywords
    Set {
        #[clap(short = 'i', long, help = "input to set in sshd_config")]
        input: String
    },
    /// Export `sshd_config`
    Export,
    Schema {
        // Used to inform which schema to generate
        #[clap(long, hide = true)]
        as_global: bool,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct DefaultShell {
    pub shell: Option<String>,
    pub cmd_option: Option<String>,
    pub escape_arguments: Option<bool>,
    pub shell_arguments: Option<Vec<String>>,
}
