// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use dsc_lib::dscresources::command_resource::TraceLevel;
use rust_i18n::t;
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
#[clap(name = "dsc", version = env!("CARGO_PKG_VERSION"), about = t!("args.about").to_string(), long_about = None)]
pub struct Args {
    /// The subcommand to run
    #[clap(subcommand)]
    pub subcommand: SubCommand,
    #[clap(short = 'l', long, help = t!("args.traceLevel").to_string(), value_enum)]
    pub trace_level: Option<TraceLevel>,
    #[clap(short = 't', long, help = t!("args.traceFormat").to_string(), value_enum)]
    pub trace_format: Option<TraceFormat>,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum SubCommand {
    #[clap(name = "completer", about = t!("args.completer").to_string())]
    Completer {
        /// The shell to generate a completion script for
        shell: Shell,
    },
    #[clap(name = "config", about = t!("args.configAbout").to_string())]
    Config {
        #[clap(subcommand)]
        subcommand: ConfigSubCommand,
        #[clap(short, long, help = t!("args.parameters").to_string(), conflicts_with = "parameters_file")]
        parameters: Option<String>,
        #[clap(short = 'f', long, help = t!("args.parametersFile").to_string(), conflicts_with = "parameters")]
        parameters_file: Option<String>,
        #[clap(short = 'r', long, help = t!("args.systemRoot").to_string())]
        system_root: Option<String>,
        // Used to inform when DSC is used as a group resource to modify it's output
        #[clap(long, hide = true)]
        as_group: bool,
        // Used to inform when DSC is used as a include group resource
        #[clap(long, hide = true)]
        as_include: bool,
    },
    #[clap(name = "resource", about = t!("args.resourceAbout").to_string())]
    Resource {
        #[clap(subcommand)]
        subcommand: ResourceSubCommand,
    },
    #[clap(name = "schema", about = t!("args.schemaAbout").to_string())]
    Schema {
        #[clap(name = "type", short, long, help = t!("args.schemaType").to_string(), value_enum)]
        dsc_type: DscType,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string(), value_enum)]
        output_format: Option<OutputFormat>,
    },
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum ConfigSubCommand {
    #[clap(name = "get", about = t!("args.getAbout").to_string())]
    Get {
        #[clap(short = 'i', long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string(), value_enum)]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "set", about = t!("args.setAbout").to_string())]
    Set {
        #[clap(short = 'i', long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<OutputFormat>,
        #[clap(short = 'w', long, help = t!("args.whatIf").to_string())]
        what_if: bool,
    },
    #[clap(name = "test", about = t!("args.testAbout").to_string())]
    Test {
        #[clap(short = 'i', long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<OutputFormat>,
        // Used by Assertion resource to return `test` result as a `get` result
        #[clap(long, hide = true)]
        as_get: bool,
        // Used by Assertion resource to return `test` result as a configuration `test` result
        #[clap(long, hide = true)]
        as_config: bool,
    },
    #[clap(name = "validate", about = t!("args.validateAbout").to_string(), hide = true)]
    Validate {
        #[clap(short = 'i', long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "export", about = t!("args.exportAbout").to_string())]
    Export {
        #[clap(short = 'i', long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "resolve", about = t!("args.resolveAbout").to_string(), hide = true)]
    Resolve {
        #[clap(short = 'i', long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<OutputFormat>,
    }
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum ResourceSubCommand {
    #[clap(name = "list", about = t!("args.listAbout").to_string())]
    List {
        /// Optional filter to apply to the list of resources
        resource_name: Option<String>,
        /// Optional adapter filter to apply to the list of resources
        #[clap(short = 'a', long = "adapter", help = t!("args.adapter").to_string())]
        adapter_name: Option<String>,
        #[clap(short, long, help = t!("args.description").to_string())]
        description: Option<String>,
        #[clap(short, long, help = t!("args.tags").to_string())]
        tags: Option<Vec<String>>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "get", about = t!("args.resourceGet").to_string(), arg_required_else_help = true)]
    Get {
        #[clap(short, long, help = t!("args.getAll").to_string())]
        all: bool,
        #[clap(short, long, help = t!("args.resource").to_string())]
        resource: String,
        #[clap(short, long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "set", about = "Invoke the set operation to a resource", arg_required_else_help = true)]
    Set {
        #[clap(short, long, help = t!("args.resource").to_string())]
        resource: String,
        #[clap(short, long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "test", about = "Invoke the test operation to a resource", arg_required_else_help = true)]
    Test {
        #[clap(short, long, help = t!("args.resource").to_string())]
        resource: String,
        #[clap(short, long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "delete", about = "Invoke the delete operation to a resource", arg_required_else_help = true)]
    Delete {
        #[clap(short, long, help = t!("args.resource").to_string())]
        resource: String,
        #[clap(short, long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
    },
    #[clap(name = "schema", about = "Get the JSON schema for a resource", arg_required_else_help = true)]
    Schema {
        #[clap(short, long, help = t!("args.resource").to_string())]
        resource: String,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "export", about = "Retrieve all resource instances", arg_required_else_help = true)]
    Export {
        #[clap(short, long, help = t!("args.resource").to_string())]
        resource: String,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
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
