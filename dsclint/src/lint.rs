// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The lint command: read the documents, run the linter, and write the result.

use crate::args::{Args, OutputFormat};
use crate::report::render_lint_result;
use crate::tablewriter::Table;
use crate::util::{
    EXIT_DSC_ERROR, EXIT_INVALID_ARGS, EXIT_INVALID_INPUT, EXIT_JSON_ERROR, EXIT_VALIDATION_FAILED,
    current_version, merge_parameters, parse_parameters, read_input, write_object,
};
use dsc_lib::dscerror::DscError;
use dsc_lib::lint::{LintError, LintLevel, LintOptions, LintResult, Linter};
use dsc_lib::progress::ProgressFormat;
use rust_i18n::t;
use serde_json::Value;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{error, info};

/// Lint the configuration documents named by the arguments, or list the rules.
///
/// # Arguments
///
/// * `args` - The command-line arguments.
///
/// # Errors
///
/// Returns `EXIT_INVALID_ARGS` for an unknown rule or missing input, `EXIT_INVALID_INPUT` when a
/// document or the parameters can't be read, `EXIT_VALIDATION_FAILED` when a diagnostic at or
/// above the `--fail-on` level was reported, `EXIT_JSON_ERROR` when the result can't be
/// serialized, and `EXIT_DSC_ERROR` for other failures.
pub fn run(args: &Args) -> Result<(), ExitCode> {
    let parameters = resolve_parameters(args.parameters.as_ref(), args.parameters_file.as_ref())?;
    let options = LintOptions {
        discovery: !args.no_discovery,
        rule_levels: args.rules.clone(),
        parameters,
        system_root: args.system_root.as_ref().map(PathBuf::from),
        dsc_version: Some(current_version()),
        progress_format: args.progress_format.unwrap_or(ProgressFormat::Default),
        ..LintOptions::default()
    };
    let mut linter = match Linter::new(options) {
        Ok(linter) => linter,
        Err(DscError::Lint(err @ (LintError::UnknownRule(_) | LintError::InvalidRuleOverride(_) | LintError::InvalidRuleLevel(_)))) => {
            error!("{}: {err}", t!("lint.invalidLintRule"));
            return Err(ExitCode::from(EXIT_INVALID_ARGS));
        },
        Err(err) => {
            error!("{}: {err}", t!("lint.lintFailed"));
            return Err(ExitCode::from(EXIT_DSC_ERROR));
        },
    };

    if args.list_rules {
        return list_rules(&linter, args.output_format.as_ref());
    }

    let mut documents: Vec<(Option<String>, String)> = Vec::new();
    if let Some(input) = &args.input {
        documents.push((None, read_input(Some(input), None)?));
    }
    for file in &args.file {
        let document = read_input(None, Some(file))?;
        let name = if file == "-" { None } else { Some(file.clone()) };
        documents.push((name, document));
    }
    if documents.is_empty() {
        error!("{}", t!("lint.noInput"));
        return Err(ExitCode::from(EXIT_INVALID_ARGS));
    }

    let mut result = LintResult::from_diagnostics(Vec::new(), true);
    for (file, document) in &documents {
        linter.set_file(file.clone());
        let document_result = match linter.lint(document) {
            Ok(document_result) => document_result,
            Err(DscError::Lint(err @ LintError::InvalidParameters(_))) => {
                error!("{}: {err}", t!("lint.lintFailed"));
                return Err(ExitCode::from(EXIT_INVALID_INPUT));
            },
            Err(err) => {
                error!("{}: {err}", t!("lint.lintFailed"));
                return Err(ExitCode::from(EXIT_DSC_ERROR));
            },
        };
        result.merge(document_result);
    }

    if args.output_format.is_none() && io::stdout().is_terminal() {
        let descriptions = linter.rules().into_iter().map(|rule| (rule.id, rule.description)).collect();
        print!("{}", render_lint_result(&result, &documents, &descriptions, true));
    } else {
        let json = match serde_json::to_string(&result) {
            Ok(json) => json,
            Err(err) => {
                error!("JSON: {err}");
                return Err(ExitCode::from(EXIT_JSON_ERROR));
            }
        };
        write_object(&json, args.output_format.as_ref())?;
    }

    let threshold = args.fail_on.unwrap_or(LintLevel::Error);
    if result.has_diagnostic_at_least(threshold) {
        return Err(ExitCode::from(EXIT_VALIDATION_FAILED));
    }
    Ok(())
}

/// Combine the inline parameters with the parameters file, if either was given.
///
/// # Arguments
///
/// * `parameters` - The inline parameters, as JSON or YAML.
/// * `parameters_file` - The path to a parameters file, or `-` to read it from STDIN.
///
/// # Returns
///
/// The parameters, or `None` when neither was given. Inline values take precedence over values
/// from the file.
///
/// # Errors
///
/// Returns `EXIT_INVALID_INPUT` when the file can't be read or either input isn't valid JSON or
/// YAML.
fn resolve_parameters(parameters: Option<&String>, parameters_file: Option<&String>) -> Result<Option<Value>, ExitCode> {
    let file_parameters = match parameters_file {
        Some(path) => Some(read_input(None, Some(path))?),
        None => None,
    };

    let parsed = match (file_parameters, parameters) {
        (Some(file_content), Some(inline_content)) => {
            info!("{}", t!("lint.mergingParameters"));
            merge_parameters(&file_content, inline_content)
        },
        (Some(file_content), None) => parse_parameters(&file_content, "FileParameters"),
        (None, Some(inline_content)) => parse_parameters(inline_content, "InlineParameters"),
        (None, None) => return Ok(None),
    };

    match parsed {
        Ok(value) => Ok(Some(value)),
        Err(err) => {
            error!("{}: {err}", t!("lint.invalidParameters"));
            Err(ExitCode::from(EXIT_INVALID_INPUT))
        }
    }
}

/// Print the lint rules with the level each reports at in this run.
///
/// # Arguments
///
/// * `linter` - The linter whose rule levels to show.
/// * `format` - The output format; a table when not given and stdout is a terminal.
///
/// # Errors
///
/// Returns `EXIT_JSON_ERROR` when the rules can't be serialized.
fn list_rules(linter: &Linter, format: Option<&OutputFormat>) -> Result<(), ExitCode> {
    let rules = linter.rules();
    if format.is_none() && io::stdout().is_terminal() {
        let mut table = Table::new(&[
            t!("lint.tableHeader_rule").to_string().as_ref(),
            t!("lint.tableHeader_category").to_string().as_ref(),
            t!("lint.tableHeader_level").to_string().as_ref(),
            t!("lint.tableHeader_tier").to_string().as_ref(),
            t!("lint.tableHeader_description").to_string().as_ref(),
        ]);
        for rule in rules {
            table.add_row(vec![
                rule.id,
                rule.category.to_string(),
                rule.level.to_string(),
                rule.tier.to_string(),
                rule.description,
            ]);
        }
        table.print(true);
        return Ok(());
    }

    let json = match serde_json::to_string(&rules) {
        Ok(json) => json,
        Err(err) => {
            error!("JSON: {err}");
            return Err(ExitCode::from(EXIT_JSON_ERROR));
        }
    };
    write_object(&json, format)
}
