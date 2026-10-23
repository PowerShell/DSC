// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Present lint diagnostics as [`miette::Diagnostic`]s, so a host can show each problem in the
//! document text it was found in.

use std::fmt::{self, Display};

use miette::{Diagnostic, LabeledSpan, NamedSource, Severity, SourceCode};
use rust_i18n::t;

use crate::diagnostics::SourceMap;

use super::diagnostic::{LintDiagnostic, LintLevel};

/// A lint diagnostic together with the text it points into.
///
/// The rule identifier is the diagnostic code, the level is the severity, and the description of
/// the rule is the help text. When the document text is available, the problem's location is
/// labeled in it.
#[derive(Debug)]
pub struct LintDiagnosticReport {
    message: String,
    rule: String,
    severity: Severity,
    help: Option<String>,
    source: Option<NamedSource<String>>,
    label: Option<LabeledSpan>,
}

impl LintDiagnosticReport {
    /// Create a report for a diagnostic.
    ///
    /// # Arguments
    ///
    /// * `diagnostic` - The diagnostic.
    /// * `source` - The source map of the document the diagnostic was found in, when the text is
    ///   available.
    /// * `help` - What the rule requires, shown as the help text.
    #[must_use]
    pub fn new(diagnostic: &LintDiagnostic, source: Option<&SourceMap>, help: Option<String>) -> Self {
        let severity = match diagnostic.level {
            LintLevel::Error => Severity::Error,
            LintLevel::Warning => Severity::Warning,
            LintLevel::Info => Severity::Advice,
        };
        let located = source.and_then(|map| {
            let (_, span) = map.nearest_span(&diagnostic.path)?;
            let first_line_end = map.text()[span.clone()].find('\n').map_or(span.end, |offset| span.start + offset);
            let span = span.start..first_line_end.max(span.start);
            let name = diagnostic.file.clone().unwrap_or_else(|| t!("lint.report.inputSourceName").to_string());
            Some((NamedSource::new(name, map.text().to_string()), LabeledSpan::new(None, span.start, span.len())))
        });
        let (source, label) = match located {
            Some((source, label)) => (Some(source), Some(label)),
            None => (None, None),
        };
        Self {
            message: diagnostic.message.clone(),
            rule: diagnostic.rule.clone(),
            severity,
            help,
            source,
            label,
        }
    }
}

impl Display for LintDiagnosticReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for LintDiagnosticReport {}

impl Diagnostic for LintDiagnosticReport {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(&self.rule))
    }

    fn severity(&self) -> Option<Severity> {
        Some(self.severity)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.help.as_ref().map(|help| Box::new(help) as Box<dyn Display + 'a>)
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        self.source.as_ref().map(|source| source as &dyn SourceCode)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        self.label.as_ref().map(|label| Box::new(std::iter::once(label.clone())) as Box<dyn Iterator<Item = LabeledSpan> + '_>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint::diagnostic::LintCategory;

    fn diagnostic(path: &str, level: LintLevel) -> LintDiagnostic {
        LintDiagnostic {
            level,
            rule: "duplicate-resource".to_string(),
            category: LintCategory::Document,
            message: "declared twice".to_string(),
            path: path.to_string(),
            file: Some("demo.dsc.yaml".to_string()),
            location: None,
        }
    }

    #[test]
    fn report_labels_the_path_in_the_document() {
        let text = "resources:\n- name: a\n  type: T/E\n- name: a\n  type: T/E\n";
        let map = SourceMap::parse(text).unwrap();
        let report = LintDiagnosticReport::new(&diagnostic("/resources/1", LintLevel::Error), Some(&map), Some("Names must be unique.".to_string()));
        assert_eq!(report.code().unwrap().to_string(), "duplicate-resource");
        assert_eq!(report.severity(), Some(Severity::Error));
        assert_eq!(report.help().unwrap().to_string(), "Names must be unique.");
        assert_eq!(report.to_string(), "declared twice");
        let label = report.labels().unwrap().next().unwrap();
        assert_eq!(&text[label.offset()..label.offset() + label.len()], "name: a");
        assert_eq!(label.offset(), text.rfind("name: a").unwrap());
        assert!(report.source_code().is_some());
    }

    #[test]
    fn report_without_text_has_no_snippet() {
        let report = LintDiagnosticReport::new(&diagnostic("/resources/0", LintLevel::Info), None, None);
        assert_eq!(report.severity(), Some(Severity::Advice));
        assert!(report.labels().is_none());
        assert!(report.source_code().is_none());
        assert!(report.help().is_none());
    }
}
