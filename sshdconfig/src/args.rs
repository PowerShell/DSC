// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand};

use crate::metadata::RepeatableKeyword;

#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Export sshd_config
    Export,
    /// Set a value in sshd_config (not yet implemented)
    Set {
        /// The input to set
        #[clap(short = 'i', long, help = "input to set in sshd_config")]
        input: Option<String>,
        #[clap(subcommand, help = "set commands")]
        subcommand: Option<SetCommand>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Subcommand)]
pub enum SetCommand {
    /// Set the default shell
    DefaultShell {
        /// The path to the shell executable
        #[clap(short = 's', long, help = "path to the shell executable")]
        shell: String,
        /// Additional command options
        ///
        #[clap(short = 'c', long, help = "additional command options", default_value = "-c")]
        cmd_option: Option<String>,
        #[clap(short = 'e', long, help = "skip escaping arguments", default_value = "false")]
        escape_arguments: bool,
        #[clap(short = 'a', long, help = "additional shell arguments")]
        shell_arguments: Option<Vec<String>>,
    },
    /// Set repeatable keywords
    RepeatableKeyworld {
        keyword: RepeatableKeyword,
        name: String,
        value: Option<String>,
    }
}
