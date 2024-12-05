// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use dsc_lib::dscresources::command_resource::TraceLevel;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    PrettyJson,
    Yaml,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum, Deserialize)]
pub enum TraceFormat {
    Default,
    Plaintext,
    Json,
    #[clap(hide = true)]
    PassThrough,
}

#[derive(Debug, Parser)]
#[clap(name = "dsc", version = env!("CARGO_PKG_VERSION"), about = "Apply configuration or invoke specific DSC resources", long_about = None)]
pub struct Args {
    /// The subcommand to run
    #[clap(subcommand)]
    pub subcommand: SubCommand,
    #[clap(short = 'l', long, help = "Trace level to use", value_enum)]
    pub trace_level: Option<TraceLevel>,
    #[clap(short = 't', long, help = "Trace format to use", value_enum)]
    pub trace_format: Option<TraceFormat>,
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
        #[clap(short = 'r', long, help = "Specify the operating system root path if not targeting the current running OS")]
        system_root: Option<String>,
        // Used to inform when DSC is used as a group resource to modify it's output
        #[clap(long, hide = true)]
        as_group: bool,
        // Used to inform when DSC is used as a include group resource
        #[clap(long, hide = true)]
        as_include: bool,
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
        #[clap(short = 'o', long, help = "The output format to use")]
        output_format: Option<OutputFormat>,
    },
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum ConfigSubCommand {
    #[clap(name = "get", about = "Retrieve the current configuration")]
    Get {
        #[clap(short = 'i', long, help = "The input document as JSON or YAML to pass to the configuration or resource", conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = "The path to a file used as input to the configuration or resource. Use '-' for the file to read from STDIN.", conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = "The output format to use")]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "set", about = "Set the current configuration")]
    Set {
        #[clap(short = 'i', long, help = "The input document as JSON or YAML to pass to the configuration or resource", conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = "The path to a file used as input to the configuration or resource. Use '-' for the file to read from STDIN.", conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = "The output format to use")]
        output_format: Option<OutputFormat>,
        #[clap(short = 'w', long, help = "Run as a what-if operation instead of executing the configuration or resource")]
        what_if: bool,
    },
    #[clap(name = "test", about = "Test the current configuration")]
    Test {
        #[clap(short = 'i', long, help = "The input document as JSON or YAML to pass to the configuration or resource", conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = "The path to a file used as input to the configuration or resource. Use '-' for the file to read from STDIN.", conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = "The output format to use")]
        output_format: Option<OutputFormat>,
        // Used by Assertion resource to return `test` result as a `get` result
        #[clap(long, hide = true)]
        as_get: bool,
        // Used by Assertion resource to return `test` result as a configuration `test` result
        #[clap(long, hide = true)]
        as_config: bool,
    },
    #[clap(name = "validate", about = "Validate the current configuration", hide = true)]
    Validate {
        #[clap(short = 'i', long, help = "The document to pass to the configuration or resource", conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = "The path to a file used as input to the configuration or resource. Use '-' for the file to read from STDIN.", conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = "The output format to use")]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "export", about = "Export the current configuration")]
    Export {
        #[clap(short = 'i', long, help = "The document to pass to the configuration or resource", conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = "The path to a file used as input to the configuration or resource. Use '-' for the file to read from STDIN.", conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = "The output format to use")]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "resolve", about = "Resolve the current configuration", hide = true)]
    Resolve {
        #[clap(short = 'i', long, help = "The document to pass to the configuration or resource", conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = "The path to a file used as input to the configuration or resource. Use '-' for the file to read from STDIN.", conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = "The output format to use")]
        output_format: Option<OutputFormat>,
    }
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum ResourceSubCommand {
    #[clap(name = "list", about = "List or find resources")]
    List {
        /// Optional filter to apply to the list of resources
        resource_name: Option<String>,
        /// Optional adapter filter to apply to the list of resources
        #[clap(short = 'a', long = "adapter", help = "Adapter filter to limit the resource search")]
        adapter_name: Option<String>,
        #[clap(short, long, help = "Description keyword to search for in the resource description")]
        description: Option<String>,
        #[clap(short, long, help = "Tag to search for in the resource tags")]
        tags: Option<Vec<String>>,
        #[clap(short = 'o', long, help = "The output format to use")]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "get", about = "Invoke the get operation to a resource", arg_required_else_help = true)]
    Get {
        #[clap(short, long, help = "Get all instances of the resource")]
        all: bool,
        #[clap(short, long, help = "The name or DscResource JSON of the resource to invoke `get` on")]
        resource: String,
        #[clap(short, long, help = "The input to pass to the resource as JSON or YAML", conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = "The path to a JSON or YAML file used as input to the configuration or resource. Use '-' as the file to read from STDIN.", conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = "The output format to use")]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "set", about = "Invoke the set operation to a resource", arg_required_else_help = true)]
    Set {
        #[clap(short, long, help = "The name or DscResource JSON of the resource to invoke `set` on")]
        resource: String,
        #[clap(short, long, help = "The input to pass to the resource as JSON or YAML", conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = "The path to a JSON or YAML file used as input to the configuration or resource. Use '-' for the file to read from STDIN.", conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = "The output format to use")]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "test", about = "Invoke the test operation to a resource", arg_required_else_help = true)]
    Test {
        #[clap(short, long, help = "The name or DscResource JSON of the resource to invoke `test` on")]
        resource: String,
        #[clap(short, long, help = "The input to pass to the resource as JSON or YAML", conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = "The path to a JSON or YAML file used as input to the configuration or resource. Use '-' for the file to read from STDIN.", conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = "The output format to use")]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "delete", about = "Invoke the delete operation to a resource", arg_required_else_help = true)]
    Delete {
        #[clap(short, long, help = "The name or DscResource JSON of the resource to invoke `delete` on")]
        resource: String,
        #[clap(short, long, help = "The input to pass to the resource as JSON or YAML", conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = "The path to a JSON or YAML file used as input to the configuration or resource. Use '-' for the file to read from STDIN.", conflicts_with = "input")]
        file: Option<String>,
    },
    #[clap(name = "schema", about = "Get the JSON schema for a resource", arg_required_else_help = true)]
    Schema {
        #[clap(short, long, help = "The name of the resource to get the JSON schema")]
        resource: String,
        #[clap(short = 'o', long, help = "The output format to use")]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "export", about = "Retrieve all resource instances", arg_required_else_help = true)]
    Export {
        #[clap(short, long, help = "The name or DscResource JSON of the resource to invoke `export` on")]
        resource: String,
        #[clap(short = 'o', long, help = "The output format to use")]
        output_format: Option<OutputFormat>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DscType {
    GetResult,
    SetResult,
    TestResult,
    ResolveResult,
    DscResource,
    ResourceManifest,
    Include,
    Configuration,
    ConfigurationGetResult,
    ConfigurationSetResult,
    ConfigurationTestResult,
}
