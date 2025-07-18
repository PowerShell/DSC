// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand, ValueEnum};
use rust_i18n::t;
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
    Get {
        #[clap(short = 'e', long, help = t!("args.getDefaults").to_string())]
        exclude_defaults: bool,
        #[clap(short = 'i', long, help = t!("args.getInput").to_string())]
        input: Option<String>,
        #[clap(short = 's', long, hide = true)]
        setting: Setting,
    },
    /// Set default shell, eventually to be used for `sshd_config` and repeatable keywords
    Set {
        #[clap(short = 'i', long, help = t!("args.setInput").to_string())]
        input: String
    },
    /// Export `sshd_config`
    Export,
    Schema {
        // Used to inform which schema to generate
        #[clap(short = 's', long, hide = true)]
        setting: Setting,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct DefaultShell {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    #[serde(rename = "cmdOption", skip_serializing_if = "Option::is_none")]
    pub cmd_option: Option<String>,
    #[serde(rename = "escapeArguments", skip_serializing_if = "Option::is_none")]
    pub escape_arguments: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum Setting {
    SshdConfig,
    WindowsGlobal
}