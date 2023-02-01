use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "registry", version = "0.0.1", about = "Manage state of Windows registry", long_about = None)]
pub struct Arguments {

    #[clap(subcommand)]
    pub subcommand: SubCommand,
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
    #[clap(name = "test", about = "Validate registry matches input JSON.")]
    Test,
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
        recurse: Option<bool>,
        #[clap(short, long, help = "Only find keys.")]
        keys_only: Option<bool>,
        #[clap(short, long, help = "Only find values.")]
        values_only: Option<bool>,
    },
}
