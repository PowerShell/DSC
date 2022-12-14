use clap::{Parser, Subcommand};

/// Struct containing the parsed command line arguments
#[derive(Debug, Parser)]
#[clap(name = "config", version = "0.0.1", about = "Discover and invoke DSC resources", long_about = None)]
pub struct Args {
    /// The subcommand to run
    #[clap(subcommand)]
    pub subcommand: SubCommand,
    /// Whether to use the cache or not
    #[clap(short = 'n', long)]
    pub no_cache: bool,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(about = "List resources")]
    List {
        /// Optional filter to apply to the list of resources
        resource_name: Option<String>,
    },
    #[clap(about = "Get the resource", arg_required_else_help = true)]
    Get {
        /// The resource to invoke `get` on
        resource_name: String,
    },
    #[clap(about = "Set the resource", arg_required_else_help = true)]
    Set {
        /// The resource to invoke `set` on
        resource_name: String,
    },
    #[clap(about = "Test the resource", arg_required_else_help = true)]
    Test {
        /// The resource to invoke `test` on
        resource_name: String,
    },
    #[clap(about = "Flush the resource cache")]
    Flush,
}
