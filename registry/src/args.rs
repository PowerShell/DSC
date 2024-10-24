// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "registry", version = "0.0.1", about = "Manage state of Windows registry", long_about = None)]
pub struct Arguments {

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum ConfigSubCommand {
    #[clap(name = "get", about = "Retrieve registry configuration.")]
    Get {
        #[clap(short, long, required = true, help = "The registry JSON input.")]
        input: String,
    },
    #[clap(name = "set", about = "Apply registry configuration.")]
    Set {
        #[clap(short, long, required = true, help = "The registry JSON input.")]
        input: String,
        #[clap(short = 'w', long, help = "Run as a what-if operation instead of applying the registry configuration")]
        what_if: bool,
    },
    #[clap(name = "delete", about = "Delete registry configuration.")]
    Delete {
        #[clap(short, long, required = true, help = "The registry JSON input.")]
        input: String,
    },
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(name = "query", about = "Query a registry key or value.", arg_required_else_help = true)]
    Query {
        #[clap(short, long, required = true, help = "The registry key path to query.")]
        key_path: String,
        #[clap(short, long, help = "The name of the value to query.")]
        value_name: Option<String>,
        #[clap(short, long, help = "Recursively query subkeys.")]
        recurse: bool,
    },
    #[clap(name = "set", about = "Set a registry key or value.")]
    Set {
        #[clap(short, long, required = true, help = "The registry key path to set.")]
        key_path: String,
        #[clap(short, long, help = "The value to set.")]
        value: String,
    },
    #[clap(name = "remove", about = "Remove a registry key or value.", arg_required_else_help = true)]
    Remove {
        #[clap(short, long, required = true, help = "The registry key path to remove.")]
        key_path: String,
        #[clap(short, long, help = "The name of the value to remove.")]
        value_name: Option<String>,
        #[clap(short, long, help = "Recursively remove subkeys.")]
        recurse: bool,
    },
    #[clap(name = "find", about = "Find a registry key or value.", arg_required_else_help = true)]
    Find {
        #[clap(short, long, required = true, help = "The registry key path to start find.")]
        key_path: String,
        #[clap(short, long, required = true, help = "The string to find.")]
        find: String,
        #[clap(short, long, help = "Recursively find.")]
        recurse: bool,
        #[clap(long, help = "Only find keys.")]
        keys_only: bool,
        #[clap(long, help = "Only find values.")]
        values_only: bool,
    },
    #[clap(name = "config", about = "Manage registry configuration.", arg_required_else_help = true)]
    Config {
        #[clap(subcommand)]
        subcommand: ConfigSubCommand,
    },
    #[clap(name = "schema", about = t!("cli.schema.about").to_string())]
    Schema {
        #[clap(short, long, help = t!("cli.schema.args.values_only.help").to_string())]
        enhanced: bool,
    },
}
