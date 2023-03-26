use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    PrettyJson,
    Yaml,
}

#[derive(Debug, Parser)]
#[clap(name = "dsc", version = "0.2.0", about = "Apply configuration or invoke specific DSC resources", long_about = None)]
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
    #[clap(name = "config", about = "Apply a configuration document")]
    Config {
        #[clap(subcommand)]
        subcommand: ConfigSubCommand,
    },
    #[clap(name = "resource", about = "Invoke a specific DSC resource")]
    Resource {
        #[clap(subcommand)]
        subcommand: ResourceSubCommand,
    },
    #[clap(name = "schema", about = "Get the JSON schema for a DSC type")]
    Schema {
        #[clap(name = "type", short, long, help = "The type of DSC schema to get")]
        dsc_type: DscType,
    },
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum ConfigSubCommand {
    #[clap(name = "get", about = "Retrieve the current configuration")]
    Get,
    #[clap(name = "set", about = "Set the current configuration")]
    Set,
    #[clap(name = "test", about = "Test the current configuration")]
    Test,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum ResourceSubCommand {
    #[clap(name = "list", about = "List or find resources")]
    List {
        /// Optional filter to apply to the list of resources
        resource_name: Option<String>,
    },
    #[clap(name = "get", about = "Invoke the get operation to a resource", arg_required_else_help = true)]
    Get {
        #[clap(short, long, help = "The name or DscResource JSON of the resource to invoke `get` on")]
        resource: String,
        #[clap(short, long, help = "The input to pass to the resource as JSON")]
        input: Option<String>,
    },
    #[clap(name = "set", about = "Invoke the set operation to a resource", arg_required_else_help = true)]
    Set {
        #[clap(short, long, help = "The name or DscResource JSON of the resource to invoke `set` on")]
        resource: String,
        #[clap(short, long, help = "The input to pass to the resource as JSON")]
        input: Option<String>,
    },
    #[clap(name = "test", about = "Invoke the test operation to a resource", arg_required_else_help = true)]
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
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum DscType {
    GetResult,
    SetResult,
    TestResult,
    DscResource,
    ResourceManifest,
    Configuration,
    ConfigurationAndResources,
    ConfigurationGetResult,
    ConfigurationSetResult,
    ConfigurationTestResult,
}
