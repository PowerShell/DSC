// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use crate::util::LogLevel;

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    PrettyJson,
    Yaml,
}

#[derive(Debug, Parser)]
#[clap(name = "dsc", version = env!("CARGO_PKG_VERSION"), about = "Apply configuration or invoke specific DSC resources", long_about = None)]
pub struct Args {
    /// The subcommand to run
    #[clap(subcommand)]
    pub subcommand: SubCommand,
    /// The output format to use
    #[clap(short = 'f', long)]
    pub format: Option<OutputFormat>,
    #[clap(short = 'i', long, help = "The input to pass to the configuration or resource", conflicts_with = "input_file")]
    pub input: Option<String>,
    #[clap(short = 'p', long, help = "The path to a file used as input to the configuration or resource")]
    pub input_file: Option<String>,
    #[clap(short = 'l', long = "logging-level", help = "Log level to display", value_enum, default_value = "info")]
    pub logging_level: LogLevel,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(name = "completer", about = "Generate a shell completion script")]
    Completer {
        /// The shell to generate a completion script for
        shell: Shell,
    },
    #[clap(name = "config", about = "Apply a configuration document")]
    Config {
        #[clap(subcommand)]
        subcommand: ConfigSubCommand,
        #[clap(short, long, help = "Parameters to pass to the configuration as JSON or YAML", conflicts_with = "parameters_file")]
        parameters: Option<String>,
        #[clap(short = 'f', long, help = "Parameters to pass to the configuration as a JSON or YAML file", conflicts_with = "parameters")]
        parameters_file: Option<String>,
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
    #[clap(name = "validate", about = "Validate the current configuration", hide = true)]
    Validate,
    #[clap(name = "export", about = "Export the current configuration")]
    Export
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum ResourceSubCommand {
    #[clap(name = "list", about = "List or find resources")]
    List {
        /// Optional filter to apply to the list of resources
        resource_name: Option<String>,
        #[clap(short, long, help = "Description keyword to search for in the resource description")]
        description: Option<String>,
        #[clap(short, long, help = "Tag to search for in the resource tags")]
        tags: Option<Vec<String>>,
    },
    #[clap(name = "get", about = "Invoke the get operation to a resource", arg_required_else_help = true)]
    Get {
        #[clap(short, long, help = "Get all instances of the resource")]
        all: bool,
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
    #[clap(name = "export", about = "Retrieve all resource instances", arg_required_else_help = true)]
    Export {
        #[clap(short, long, help = "The name or DscResource JSON of the resource to invoke `export` on")]
        resource: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DscType {
    GetResult,
    SetResult,
    TestResult,
    DscResource,
    ResourceManifest,
    Configuration,
    ConfigurationGetResult,
    ConfigurationSetResult,
    ConfigurationTestResult,
}
