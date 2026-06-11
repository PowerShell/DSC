// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand};
use rust_i18n::t;

#[derive(Parser)]
#[clap(name = "registry", version = env!("CARGO_PKG_VERSION"), about = t!("args.about").to_string(), long_about = None)]
pub struct Arguments {

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum AdapterSubCommand {
    #[clap(name = "get", about = t!("args.adapterGetAbout").to_string())]
    Get {
        #[clap(short, long, required = true, help = t!("args.adapterArgsInputHelp").to_string())]
        input: String,
        #[clap(short, long, required = true, help = t!("args.adapterArgsAdaptedResourceHelp").to_string())]
        adapted_resource: String,
    },
    #[clap(name = "set", about = t!("args.adapterSetAbout").to_string())]
    Set {
        #[clap(short, long, required = true, help = t!("args.adapterArgsInputHelp").to_string())]
        input: String,
        #[clap(short, long, required = true, help = t!("args.adapterArgsAdaptedResourceHelp").to_string())]
        adapted_resource: String,
    },
    #[clap(name = "export", about = t!("args.adapterExportAbout").to_string())]
    Export {
        #[clap(short, long, required = true, help = t!("args.adapterArgsInputHelp").to_string())]
        input: String,
        #[clap(short, long, required = true, help = t!("args.adapterArgsAdaptedResourceHelp").to_string())]
        adapted_resource: String,
    },
    #[clap(name = "schema", about = t!("args.adapterSchemaAbout").to_string())]
    Schema
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum ConfigSubCommand {
    #[clap(name = "get", about = t!("args.configGetAbout").to_string())]
    Get {
        #[clap(short, long, required = true, help = t!("args.configArgsInputHelp").to_string())]
        input: String,
        #[clap(short, long, hide = true)]
        list: bool,
    },
    #[clap(name = "set", about = t!("args.configSetAbout").to_string())]
    Set {
        #[clap(short, long, required = true, help = t!("args.configArgsInputHelp").to_string())]
        input: String,
        #[clap(short, long, hide = true)]
        list: bool,
        #[clap(short = 'w', long, help = t!("args.configArgsWhatIfHelp").to_string())]
        what_if: bool,
    },
    #[clap(name = "delete", about = t!("args.configDeleteAbout").to_string())]
    Delete {
        #[clap(short, long, required = true, help = t!("args.configArgsInputHelp").to_string())]
        input: String,
        #[clap(short = 'w', long, help = t!("args.configArgsWhatIfHelp").to_string())]
        what_if: bool,
    },
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(name = "query", about = t!("args.queryAbout").to_string(), arg_required_else_help = true)]
    Query {
        #[clap(short, long, required = true, help = t!("args.queryArgsKeyPathHelp").to_string())]
        key_path: String,
        #[clap(short, long, help = t!("args.queryArgsValueNameHelp").to_string())]
        value_name: Option<String>,
        #[clap(short, long, help = t!("args.queryArgsRecurseHelp").to_string())]
        recurse: bool,
    },
    #[clap(name = "set", about = t!("args.setAbout").to_string())]
    Set {
        #[clap(short, long, required = true, help = t!("args.setArgsKeyPathHelp").to_string())]
        key_path: String,
        #[clap(short, long, help = t!("args.setArgsValueHelp").to_string())]
        value: String,
    },
    #[clap(name = "remove", about = t!("args.removeAbout").to_string(), arg_required_else_help = true)]
    Remove {
        #[clap(short, long, required = true, help = t!("args.removeArgsKeyPathHelp").to_string())]
        key_path: String,
        #[clap(short, long, help = t!("args.removeArgsValueNameHelp").to_string())]
        value_name: Option<String>,
        #[clap(short, long, help = t!("args.removeArgsRecurseHelp").to_string())]
        recurse: bool,
    },
    #[clap(name = "find", about = t!("args.findAbout").to_string(), arg_required_else_help = true)]
    Find {
        #[clap(short, long, required = true, help = t!("args.findArgsKeyPathHelp").to_string())]
        key_path: String,
        #[clap(short, long, required = true, help = t!("args.findArgsFindHelp").to_string())]
        find: String,
        #[clap(short, long, help = t!("args.findArgsRecurseHelp").to_string())]
        recurse: bool,
        #[clap(long, help = t!("args.findArgsKeysOnlyHelp").to_string())]
        keys_only: bool,
        #[clap(long, help = t!("args.findArgsValuesOnlyHelp").to_string())]
        values_only: bool,
    },
    #[clap(name = "adapter", about = t!("args.adapterAbout").to_string(), arg_required_else_help = true)]
    Adapter {
        #[clap(subcommand)]
        subcommand: AdapterSubCommand,
    },
    #[clap(name = "config", about = t!("args.configAbout").to_string(), arg_required_else_help = true)]
    Config {
        #[clap(subcommand)]
        subcommand: ConfigSubCommand,
    },
    #[clap(name = "schema", about = t!("args.schemaAbout").to_string())]
    Schema {
        #[clap(short, long, help = t!("args.schemaArgsListHelp").to_string())]
        list: bool,
    }
}
