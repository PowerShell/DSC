// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Present lint diagnostics as [`miette::Diagnostic`]s, so a host can show each problem in the
//! document text it was found in.

use std::fmt::{self, Display};

use miette::{Diagnostic, LabeledSpan, NamedSource, Severity, SourceCode};
use rust_i18n::t;
use serde_json::Value;

use crate::{
    configure::parameters::{SECURE_VALUE_REDACTED, is_secure_value},
    diagnostics::SourceMap,
    util::parse_input_to_value,
};

use super::{diagnostic::{LintDiagnostic, LintLevel}, pointer::JsonPointer};

/// Parse document text for showing diagnostics in it, with every secure value replaced.
///
/// The default value of each `secureString` or `secureObject` parameter and every wrapped
/// `{"secureString": ...}` or `{"secureObject": ...}` value is replaced in the text by
/// `<secureValue>`, so a snippet, including the lines of context around the labeled line, never
/// shows a secret the author hard-coded. Diagnostics keep their paths, because the structure of
/// the document is unchanged.
///
/// # Arguments
///
/// * `text` - The document text, as JSON or YAML.
///
/// # Returns
///
/// The source map of the redacted text, or `None` when the text can't be mapped or a secure
/// value can't be located in it; a host then reports the diagnostics without a snippet.
#[must_use]
pub fn redacted_source_map(text: &str) -> Option<SourceMap> {
    let map = SourceMap::parse(text)?;
    let raw = parse_input_to_value(text).ok()?;
    let mut pointers = Vec::new();
    if let Some(parameters) = raw.get("parameters").and_then(Value::as_object) {
        for (name, parameter) in parameters {
            let secure_type = parameter.get("type").and_then(Value::as_str)
                .is_some_and(|kind| kind == "secureString" || kind == "secureObject");
            if secure_type && parameter.get("defaultValue").is_some() {
                pointers.push(JsonPointer::root().push("parameters").push(name).push("defaultValue").to_string());
            }
        }
    }
    collect_secure_values(&raw, &JsonPointer::root(), &mut pointers);
    map.redacted(&pointers, SECURE_VALUE_REDACTED)
}

/// Record the pointer of every wrapped secure value at or below a value.
///
/// # Arguments
///
/// * `value` - The value to search.
/// * `pointer` - The pointer of `value` in the document.
/// * `pointers` - Where the pointers found are appended.
fn collect_secure_values(value: &Value, pointer: &JsonPointer, pointers: &mut Vec<String>) {
    if is_secure_value(value) {
        pointers.push(pointer.to_string());
        return;
    }
    match value {
        Value::Object(members) => {
            for (key, member) in members {
                collect_secure_values(member, &pointer.push(key), pointers);
            }
        },
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_secure_values(item, &pointer.push_index(index), pointers);
            }
        },
        _ => {},
    }
}

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

    #[test]
    fn redacted_source_hides_secure_values_and_keeps_locations() {
        let text = "parameters:\n  password:\n    type: secureString\n    defaultValue: hunter2\n  blob:\n    type: secureObject\n    defaultValue:\n      key: letmein\n  plain:\n    type: string\n    defaultValue: visible\nresources:\n- name: a\n  type: T/E\n  properties:\n    token:\n      secureString: topsecret\n    output: hi\n";
        let map = redacted_source_map(text).unwrap();
        for secret in ["hunter2", "letmein", "topsecret"] {
            assert!(!map.text().contains(secret), "{}", map.text());
        }
        assert!(map.text().contains("visible"));
        assert_eq!(&map.text()[map.value_span("/parameters/password/defaultValue").unwrap()], "\"<secureValue>\"");
        assert_eq!(&map.text()[map.value_span("/parameters/blob/defaultValue").unwrap()], "\"<secureValue>\"");
        assert_eq!(&map.text()[map.value_span("/resources/0/properties/token").unwrap()], "\"<secureValue>\"");
        let report = LintDiagnosticReport::new(&diagnostic("/parameters/password/defaultValue", LintLevel::Warning), Some(&map), None);
        let label = report.labels().unwrap().next().unwrap();
        assert_eq!(&map.text()[label.offset()..label.offset() + label.len()], "defaultValue");
        assert_eq!(map.location(label.offset()), (4, 5));
    }

    #[test]
    fn redacted_source_handles_json_documents() {
        let text = r#"{"parameters": {"password": {"type": "secureString", "defaultValue": "hunter2"}}, "resources": [{"name": "a", "type": "T/E", "properties": {"token": {"secureObject": {"k": "letmein"}}}}]}"#;
        let map = redacted_source_map(text).unwrap();
        assert!(!map.text().contains("hunter2") && !map.text().contains("letmein"), "{}", map.text());
        assert!(map.value_span("/resources/0/properties/token").is_some());
        assert!(map.value_span("/resources/0/name").is_some());
    }

    #[test]
    fn text_without_secure_values_is_unchanged_and_unmappable_text_has_no_map() {
        let text = "resources:\n- name: a\n  type: T/E\n";
        assert_eq!(redacted_source_map(text).unwrap().text(), text);
        assert!(redacted_source_map("a: [b").is_none());
    }
}
