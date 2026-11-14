// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::{Parser, ValueEnum};
use dsc_lib::dscresources::command_resource::TraceLevel;
use dsc_lib::lint::{LintLevel, RuleLevelOverride};
use dsc_lib::progress::ProgressFormat;
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
#[clap(name = "dsclint", version = env!("CARGO_PKG_VERSION"), about = t!("args.about").to_string(), long_about = None)]
pub struct Args {
    #[clap(short = 'i', long, help = t!("args.input").to_string(), conflicts_with = "file")]
    pub input: Option<String>,
    #[clap(short = 'f', long, help = t!("args.file").to_string(), conflicts_with = "input")]
    pub file: Vec<String>,
    #[clap(short = 'p', long, help = t!("args.parameters").to_string())]
    pub parameters: Option<String>,
    #[clap(long, help = t!("args.parametersFile").to_string())]
    pub parameters_file: Option<String>,
    #[clap(short = 'r', long, help = t!("args.systemRoot").to_string())]
    pub system_root: Option<String>,
    #[clap(long, help = t!("args.noDiscovery").to_string())]
    pub no_discovery: bool,
    #[clap(long, help = t!("args.failOn").to_string(), value_enum)]
    pub fail_on: Option<LintLevel>,
    #[clap(long = "rule", value_name = "RULE=LEVEL", help = t!("args.rule").to_string())]
    pub rules: Vec<RuleLevelOverride>,
    #[clap(long, help = t!("args.listRules").to_string(), conflicts_with_all = ["input", "file"])]
    pub list_rules: bool,
    #[clap(short = 'o', long, help = t!("args.outputFormat").to_string(), value_enum)]
    pub output_format: Option<OutputFormat>,
    #[clap(short = 'l', long, help = t!("args.traceLevel").to_string(), value_enum)]
    pub trace_level: Option<TraceLevel>,
    #[clap(short = 't', long, help = t!("args.traceFormat").to_string(), value_enum)]
    pub trace_format: Option<TraceFormat>,
    // `-p` is taken by `--parameters`, as in `dsc config`, so this option has no short form.
    #[clap(long, help = t!("args.progressFormat").to_string(), value_enum)]
    pub progress_format: Option<ProgressFormat>,
    #[clap(long, help = t!("args.ignoreSettingsFile").to_string())]
    pub ignore_settings_file: bool,
}
