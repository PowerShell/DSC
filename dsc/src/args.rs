// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use dsc_lib::dscresources::command_resource::TraceLevel;
use dsc_lib::progress::ProgressFormat;
use rust_i18n::t;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    PrettyJson,
    Yaml,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum GetOutputFormat {
    Json,
    JsonArray,
    PassThrough,
    PrettyJson,
    Yaml,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum ListOutputFormat {
    Json,
    PrettyJson,
    Yaml,
    TableNoTruncate,
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
    #[clap(short = 'p', long, help = t!("args.progressFormat").to_string(), value_enum)]
    pub progress_format: Option<ProgressFormat>,
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
        #[clap(short, long, help = t!("args.parameters").to_string())]
        parameters: Option<String>,
        #[clap(short = 'f', long, help = t!("args.parametersFile").to_string())]
        parameters_file: Option<String>,
        #[clap(short = 'r', long, help = t!("args.systemRoot").to_string())]
        system_root: Option<String>,
        // Used to inform when DSC is used as a group resource to modify it's output
        #[clap(long, hide = true)]
        as_group: bool,
        #[clap(long, hide = true)]
        as_assert: bool,
        // Used to inform when DSC is used as a include group resource
        #[clap(long, hide = true)]
        as_include: bool,
    },
    #[clap(name = "extension", about = t!("args.extensionAbout").to_string())]
    Extension {
        #[clap(subcommand)]
        subcommand: ExtensionSubCommand,
    },
    #[clap(name = "function", about = t!("args.functionAbout").to_string())]
    Function {
        #[clap(subcommand)]
        subcommand: FunctionSubCommand,
    },
    #[clap(name = "mcp", about = t!("args.mcpAbout").to_string())]
    Mcp,
    #[clap(name = "resource", about = t!("args.resourceAbout").to_string())]
    Resource {
        #[clap(subcommand)]
        subcommand: ResourceSubCommand,
    },
    #[clap(name = "schema", about = t!("args.schemaAbout").to_string())]
    Schema {
        #[clap(name = "type", short, long, help = t!("args.schemaType").to_string(), value_enum)]
        dsc_type: SchemaType,
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
        #[clap(short = 'w', long, visible_aliases = ["dry-run", "noop"], help = t!("args.whatIf").to_string())]
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
pub enum ExtensionSubCommand {
    #[clap(name = "list", about = t!("args.listExtensionAbout").to_string())]
    List {
        /// Optional extension name to filter the list
        extension_name: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<ListOutputFormat>,
    },
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum FunctionSubCommand {
    #[clap(name = "list", about = t!("args.listFunctionAbout").to_string())]
    List {
        /// Optional function name to filter the list
        function_name: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<ListOutputFormat>,
    },
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum ResourceSubCommand {
    #[clap(name = "list", about = t!("args.listAbout").to_string())]
    List {
        /// Optional resource name to filter the list
        resource_name: Option<String>,
        /// Optional adapter filter to apply to the list of resources
        #[clap(short = 'a', long = "adapter", help = t!("args.adapter").to_string())]
        adapter_name: Option<String>,
        #[clap(short, long, help = t!("args.description").to_string())]
        description: Option<String>,
        #[clap(short, long, help = t!("args.tags").to_string())]
        tags: Option<Vec<String>>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<ListOutputFormat>,
    },
    #[clap(name = "get", about = t!("args.resourceGet").to_string(), arg_required_else_help = true)]
    Get {
        #[clap(short, long, help = t!("args.getAll").to_string())]
        all: bool,
        #[clap(short, long, help = t!("args.resource").to_string())]
        resource: String,
        #[clap(short, long, help = t!("args.version").to_string())]
        version: Option<String>,
        #[clap(short, long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<GetOutputFormat>,
    },
    #[clap(name = "set", about = "Invoke the set operation to a resource", arg_required_else_help = true)]
    Set {
        #[clap(short, long, help = t!("args.resource").to_string())]
        resource: String,
        #[clap(short, long, help = t!("args.version").to_string())]
        version: Option<String>,
        #[clap(short, long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<OutputFormat>,
        #[clap(short = 'w', long, visible_aliases = ["dry-run", "noop"], help = t!("args.whatIf").to_string())]
        what_if: bool,
    },
    #[clap(name = "test", about = "Invoke the test operation to a resource", arg_required_else_help = true)]
    Test {
        #[clap(short, long, help = t!("args.resource").to_string())]
        resource: String,
        #[clap(short, long, help = t!("args.version").to_string())]
        version: Option<String>,
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
        #[clap(short, long, help = t!("args.version").to_string())]
        version: Option<String>,
        #[clap(short, long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
    },
    #[clap(name = "schema", about = "Get the JSON schema for a resource", arg_required_else_help = true)]
    Schema {
        #[clap(short, long, help = t!("args.resource").to_string())]
        resource: String,
        #[clap(short, long, help = t!("args.version").to_string())]
        version: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<OutputFormat>,
    },
    #[clap(name = "export", about = "Retrieve all resource instances", arg_required_else_help = true)]
    Export {
        #[clap(short, long, help = t!("args.resource").to_string())]
        resource: String,
        #[clap(short, long, help = t!("args.version").to_string())]
        version: Option<String>,
        #[clap(short, long, help = t!("args.input").to_string(), conflicts_with = "file")]
        input: Option<String>,
        #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
        file: Option<String>,
        #[clap(short = 'o', long, help = t!("args.outputFormat").to_string())]
        output_format: Option<OutputFormat>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SchemaType {
    Configuration,
    ConfigurationGetResult,
    ConfigurationSetResult,
    ConfigurationTestResult,
    DscResource,
    ExtensionDiscoverResult,
    ExtensionManifest,
    FunctionDefinition,
    GetResult,
    Include,
    ManifestList,
    ResolveResult,
    Resource,
    ResourceManifest,
    RestartRequired,
    SetResult,
    TestResult,
}
