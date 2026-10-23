// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Render errors for the terminal.
//!
//! Errors from `dsc-lib` are [`miette::Diagnostic`]s. Most carry only a message, but the ones
//! found while validating a document or resource input also carry the text they apply to, with a
//! label on each problem and sometimes a suggestion. [`render`] turns such an error into a report
//! that shows the problem in place, in the style the trace format asks for.

use std::collections::HashMap;
use std::io::{IsTerminal, stderr};
use std::sync::OnceLock;

use dsc_lib::diagnostics::SourceMap;
use dsc_lib::lint::{LintDiagnosticReport, LintResult};
use dsc_lib_telemetry::StderrFormat;
use miette::{Diagnostic, GraphicalReportHandler, GraphicalTheme};
use rust_i18n::t;

/// The lines shown around each labeled line of a snippet.
const CONTEXT_LINES: usize = 2;

static STDERR_FORMAT: OnceLock<StderrFormat> = OnceLock::new();

/// Record the trace format so that reports match the rest of the trace output.
///
/// Only the first call takes effect.
///
/// # Arguments
///
/// * `format` - The format trace output is written in.
pub fn set_stderr_format(format: StderrFormat) {
    let _ = STDERR_FORMAT.set(format);
}

/// Render an error for the error log.
///
/// An error that carries source text or help is rendered as a report with the text, the labels
/// on it, and the help; any other error is rendered as its message. When the trace format is
/// JSON the message alone is used, so that consumers get one line per error.
///
/// # Arguments
///
/// * `error` - The error to render.
#[must_use]
pub fn render(error: &dyn Diagnostic) -> String {
    let format = STDERR_FORMAT.get().copied().unwrap_or(StderrFormat::Default);
    let has_detail = error.source_code().is_some() || error.help().is_some();
    if !has_detail || format == StderrFormat::Json {
        return error.to_string();
    }

    let handler = handler(format == StderrFormat::Default && stderr().is_terminal());
    let mut report = String::new();
    if handler.render_report(&mut report, error).is_err() {
        return error.to_string();
    }
    report.trim_end().to_string()
}

/// Render the diagnostics of a lint run as reports that show each problem in its document,
/// followed by the counts by level.
///
/// # Arguments
///
/// * `result` - The lint result.
/// * `documents` - The text of each linted document, keyed by the file name recorded on its
///   diagnostics; `None` for inline input.
/// * `descriptions` - What each rule requires, by rule identifier, shown as the help text.
/// * `colored` - Whether to style the reports with ANSI colors.
#[must_use]
pub fn render_lint_result(result: &LintResult, documents: &[(Option<String>, String)], descriptions: &HashMap<String, String>, colored: bool) -> String {
    let maps: HashMap<&Option<String>, SourceMap> = documents.iter()
        .filter_map(|(file, text)| SourceMap::parse(text).map(|map| (file, map)))
        .collect();
    let handler = handler(colored);
    let mut output = String::new();
    for diagnostic in &result.diagnostics {
        let report = LintDiagnosticReport::new(diagnostic, maps.get(&diagnostic.file), descriptions.get(&diagnostic.rule).cloned());
        if handler.render_report(&mut output, &report).is_err() {
            output.push_str(&format!("{} {}: {}\n", diagnostic.level, diagnostic.rule, diagnostic.message));
        }
        output.push('\n');
    }
    output.push_str(&t!(
        "report.lintSummary",
        errors = result.summary.errors,
        warnings = result.summary.warnings,
        info = result.summary.info
    ));
    output.push('\n');
    output
}

/// A report handler for the terminal, drawing with unicode and, unless `NO_COLOR` asks
/// otherwise, ANSI colors.
///
/// # Arguments
///
/// * `colored` - Whether colors are wanted.
fn handler(colored: bool) -> GraphicalReportHandler {
    let no_color = std::env::var("NO_COLOR").is_ok_and(|value| value != "0");
    let theme = if colored && !no_color {
        GraphicalTheme::unicode()
    } else {
        GraphicalTheme::unicode_nocolor()
    };
    GraphicalReportHandler::new_themed(theme).with_context_lines(CONTEXT_LINES)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dsc_lib::{diagnostics::SchemaValidationError, dscerror::DscError, dscresources::dscresource::validate_json_all, lint::{LintCategory, LintDiagnostic, LintLevel}};
    use serde_json::json;

    fn schema_error() -> DscError {
        let schema = json!({"type": "object", "additionalProperties": false, "properties": {"output": {"type": "string"}}});
        let text = "{\n  \"otput\": \"hi\"\n}";
        let violations = validate_json_all("test", &schema, &serde_json::from_str(text).unwrap()).unwrap();
        let error = SchemaValidationError::new("input for 'Test/Echo'", violations)
            .with_schema(&schema)
            .with_source(text, "input", "");
        DscError::SchemaValidation(Box::new(error))
    }

    #[test]
    fn plain_errors_render_as_their_message() {
        let error = DscError::Operation("failed".to_string());
        assert_eq!(render(&error), error.to_string());
    }

    #[test]
    fn errors_with_source_render_a_snippet_and_help() {
        let report = render(&schema_error());
        assert!(report.contains("input for 'Test/Echo'"), "{report}");
        assert!(report.contains("\"otput\""), "{report}");
        assert!(report.contains("'otput' isn't a known property"), "{report}");
        assert!(report.contains("Did you mean 'output'"), "{report}");
        assert!(report.contains("[input:2:3]"), "{report}");
    }

    #[test]
    fn lint_results_render_each_diagnostic_in_its_document() {
        let text = "resources:\n- name: a\n  type: T/E\n- name: a\n  type: T/E\n".to_string();
        let diagnostic = LintDiagnostic {
            level: LintLevel::Warning,
            rule: "duplicate-resource".to_string(),
            category: LintCategory::Document,
            message: "declared twice".to_string(),
            path: "/resources/1".to_string(),
            file: Some("demo.dsc.yaml".to_string()),
            location: None,
        };
        let result = LintResult::from_diagnostics(vec![diagnostic], false);
        let descriptions = HashMap::from([("duplicate-resource".to_string(), "Names must be unique.".to_string())]);
        let output = render_lint_result(&result, &[(Some("demo.dsc.yaml".to_string()), text)], &descriptions, false);
        assert!(output.contains("duplicate-resource"), "{output}");
        assert!(output.contains("declared twice"), "{output}");
        assert!(output.contains("[demo.dsc.yaml:4:3]"), "{output}");
        assert!(output.contains("Names must be unique."), "{output}");
        assert!(output.contains("errors: 0, warnings: 1, info: 0"), "{output}");
    }
}
