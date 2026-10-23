// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Render errors for the terminal.
//!
//! Errors from `dsc-lib` are [`miette::Diagnostic`]s. Most carry only a message, but the ones
//! found while validating a document or resource input also carry the text they apply to, with a
//! label on each problem and sometimes a suggestion. [`render`] turns such an error into a report
//! that shows the problem in place, in the style the trace format asks for.

use std::io::{IsTerminal, stderr};
use std::sync::OnceLock;

use dsc_lib_telemetry::StderrFormat;
use miette::{Diagnostic, GraphicalReportHandler, GraphicalTheme};

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
    use dsc_lib::{diagnostics::SchemaValidationError, dscerror::DscError, dscresources::dscresource::validate_json_all};
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
}
