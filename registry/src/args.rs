// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "registry", version = "0.0.1", about = t!("cli.about").to_string(), long_about = None)]
pub struct Arguments {

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum ConfigSubCommand {
    #[clap(name = "get", about = t!("cli.config.get.about").to_string())]
    Get {
        #[clap(short, long, required = true, help = t!("cli.config.args.input.help").to_string())]
        input: String,
    },
    #[clap(name = "set", about = t!("cli.config.set.about").to_string())]
    Set {
        #[clap(short, long, required = true, help = t!("cli.config.args.input.help").to_string())]
        input: String,
        #[clap(short = 'w', long, help = t!("cli.config.args.what_if.help").to_string())]
        what_if: bool,
    },
    #[clap(name = "delete", about = t!("cli.config.delete.about").to_string())]
    Delete {
        #[clap(short, long, required = true, help = t!("cli.config.args.input.help").to_string())]
        input: String,
    },
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(name = "query", about = t!("cli.query.about").to_string(), arg_required_else_help = true)]
    Query {
        #[clap(short, long, required = true, help = t!("cli.query.args.key_path.help").to_string())]
        key_path: String,
        #[clap(short, long, help = t!("cli.query.args.value_name.help").to_string())]
        value_name: Option<String>,
        #[clap(short, long, help = t!("cli.query.args.recurse.help").to_string())]
        recurse: bool,
    },
    #[clap(name = "set", about = t!("cli.set.about").to_string())]
    Set {
        #[clap(short, long, required = true, help = t!("cli.set.args.key_path.help").to_string())]
        key_path: String,
        #[clap(short, long, help = t!("cli.set.args.value.help").to_string())]
        value: String,
    },
    #[clap(name = "remove", about = t!("cli.remove.about").to_string(), arg_required_else_help = true)]
    Remove {
        #[clap(short, long, required = true, help = t!("cli.remove.args.key_path.help").to_string())]
        key_path: String,
        #[clap(short, long, help = t!("cli.remove.args.value_name.help").to_string())]
        value_name: Option<String>,
        #[clap(short, long, help = t!("cli.remove.args.recurse.help").to_string())]
        recurse: bool,
    },
    #[clap(name = "find", about = t!("cli.find.about").to_string(), arg_required_else_help = true)]
    Find {
        #[clap(short, long, required = true, help = t!("cli.find.args.key_path.help").to_string())]
        key_path: String,
        #[clap(short, long, required = true, help = t!("cli.find.args.find.help").to_string())]
        find: String,
        #[clap(short, long, help = t!("cli.find.args.recurse.help").to_string())]
        recurse: bool,
        #[clap(long, help = t!("cli.find.args.keys_only.help").to_string())]
        keys_only: bool,
        #[clap(long, help = t!("cli.find.args.values_only.help").to_string())]
        values_only: bool,
    },
    #[clap(name = "config", about = t!("cli.config.about").to_string(), arg_required_else_help = true)]
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
