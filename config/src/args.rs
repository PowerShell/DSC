use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    PrettyJson,
    Yaml,
}

#[derive(Debug, Parser)]
#[clap(name = "config", version = "0.1.0", about = "Discover and invoke DSC resources", long_about = None)]
pub struct Args {
    /// The subcommand to run
    #[clap(subcommand)]
    pub subcommand: SubCommand,
    /// Whether to use the cache or not
    #[clap(short = 'n', long)]
    pub no_cache: bool,
    #[clap(short = 'f', long)]
    pub format: Option<OutputFormat>,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(name = "list", about = "List resources")]
    List {
        /// Optional filter to apply to the list of resources
        resource_name: Option<String>,
    },
    #[clap(name = "get", about = "Get the resource", arg_required_else_help = true)]
    Get {
        #[clap(short, long, help = "The name or DscResource JSON of the resource to invoke `get` on")]
        resource: String,
        #[clap(short, long, help = "The input to pass to the resource as JSON")]
        input: Option<String>,
    },
    #[clap(name = "set", about = "Set the resource", arg_required_else_help = true)]
    Set {
        #[clap(short, long, help = "The name or DscResource JSON of the resource to invoke `set` on")]
        resource: String,
        #[clap(short, long, help = "The input to pass to the resource as JSON")]
        input: Option<String>,
    },
    #[clap(name = "test", about = "Test the resource", arg_required_else_help = true)]
    Test {
        #[clap(short, long, help = "The name or DscResource JSON of the resource to invoke `test` on")]
        resource: String,
        #[clap(short, long, help = "The input to pass to the resource as JSON")]
        input: Option<String>,
    },
    #[clap(name = "schema", about = "Get the JSON schema for a resource", arg_required_else_help = true)]
    Schema {
        #[clap(short, long, help = "The name of the resource to get the JSON schema")]
        resource: String,
    },
    #[clap(name = "dscschema", about = "Get the JSON schema for a DSC type", arg_required_else_help = true)]
    DscSchema {
        #[clap(name = "type", short, long, help = "The name of the DSC type to get the JSON schema")]
        dsc_type: DscType,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum DscType {
    GetResult,
    SetResult,
    TestResult,
    ResourceManifest,
}
