// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand, ValueEnum};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum TraceFormat {
    Default,
    Plaintext,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum TraceLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace
}

#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
    #[clap(short = 'l', long, help = "Trace level to use", value_enum)]
    pub trace_level: Option<TraceLevel>,
    #[clap(short = 'f', long, help = "Trace format to use", value_enum, default_value = "json")]
    pub trace_format: TraceFormat,
}

#[derive(Subcommand)]
pub enum Command {
    /// Get default shell and `sshd_config`, eventually to be used for repeatable keywords
    Get {
        #[clap(short = 'i', long, help = t!("args.getInput").to_string())]
        input: Option<String>,
        #[clap(short = 's', long, hide = true)]
        setting: Setting,
    },
    /// Set default shell, eventually to be used for `sshd_config` and repeatable keywords
    Set {
        #[clap(short = 'i', long, help = t!("args.setInput").to_string())]
        input: String,
        #[clap(short = 's', long, hide = true)]
        setting: Setting,
    },
    /// Export `sshd_config`, eventually to be used for repeatable keywords
    Export {
        #[clap(short = 'i', long, help = t!("args.exportInput").to_string())]
        input: Option<String>,
        #[clap(short = 'c', long, help = t!("args.exportCompare").to_string())]
        compare: bool,
    },
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
    SshdConfigRepeat,
    SshdConfigRepeatList,
    WindowsGlobal,
}
