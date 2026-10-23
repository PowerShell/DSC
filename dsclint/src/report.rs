// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Render lint results for the terminal.
//!
//! Each [`dsc_lib::lint::LintDiagnostic`] is rendered as a [`miette`] report that shows the problem
//! in place in the document it was found in, followed by the counts by level.

use std::collections::HashMap;

use dsc_lib::diagnostics::SourceMap;
use dsc_lib::lint::{LintDiagnosticReport, LintResult};
use miette::{GraphicalReportHandler, GraphicalTheme};
use rust_i18n::t;

/// The lines shown around each labeled line of a snippet.
const CONTEXT_LINES: usize = 2;

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
    use dsc_lib::lint::{LintCategory, LintDiagnostic, LintLevel};

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
