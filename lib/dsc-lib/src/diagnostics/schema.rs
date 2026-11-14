// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Schema validation failures that point at the text they were found in.

// TODO: Remove when updating to rustc 1.94; false positive from thiserror + rust-i18n t!() macro
#![allow(unused_assignments)]

use std::fmt::{self, Display};

use miette::{Diagnostic, LabeledSpan, NamedSource, SourceCode};
use rust_i18n::t;
use serde_json::Value;

use crate::dscresources::dscresource::{SchemaViolation, SchemaViolationDetail};

use super::source_map::SourceMap;

/// Property names within this edit distance of an unknown property are suggested as corrections.
const MAX_SUGGESTION_DISTANCE: usize = 2;
/// The most property names to list when there is no close match.
const MAX_LISTED_PROPERTIES: usize = 12;

/// Where the value that was validated against a schema came from.
///
/// A host that knows the document a value was taken from can re-anchor an error about input to
/// the document; an error about what a resource returned has no place in the document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JsonOrigin {
    /// The desired state, filter, or properties a caller passed to the resource.
    #[default]
    Input,
    /// The state the resource returned.
    Output,
}

/// Every violation found when validating a value against a JSON schema.
///
/// The error is a [`miette::Diagnostic`]: when source text is attached with
/// [`with_source`](Self::with_source), each violation becomes a label on the span of the text it
/// applies to, and unknown properties get a suggestion in the help text when the schema is
/// attached with [`with_schema`](Self::with_schema). Without source text the error still lists
/// every violation and its JSON pointer in its message.
///
/// An error that was [`redacted`](Self::redacted) because the value holds secure data never
/// accepts source text again, so a host that re-anchors errors can't reintroduce the value.
#[derive(Debug)]
pub struct SchemaValidationError {
    subject: String,
    violations: Vec<SchemaViolation>,
    origin: JsonOrigin,
    redacted: bool,
    source: Option<NamedSource<String>>,
    labels: Vec<LabeledSpan>,
    help: Option<String>,
}

impl SchemaValidationError {
    /// Create an error for a set of violations.
    ///
    /// The origin is [`JsonOrigin::Input`] until [`with_origin`](Self::with_origin) says
    /// otherwise.
    ///
    /// # Arguments
    ///
    /// * `subject` - What was validated, as it should read in the message, like
    ///   `input for 'Microsoft.DSC.Debug/Echo'` or `Configuration`.
    /// * `violations` - The violations found; must not be empty.
    #[must_use]
    pub fn new(subject: impl Into<String>, violations: Vec<SchemaViolation>) -> Self {
        Self {
            subject: subject.into(),
            violations,
            origin: JsonOrigin::default(),
            redacted: false,
            source: None,
            labels: Vec::new(),
            help: None,
        }
    }

    /// Record where the validated value came from.
    ///
    /// # Arguments
    ///
    /// * `origin` - Whether the value was input to a resource or output from one.
    #[must_use]
    pub fn with_origin(mut self, origin: JsonOrigin) -> Self {
        self.origin = origin;
        self
    }

    /// Use the schema to suggest corrections for unknown properties in the help text.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema the value was validated against.
    #[must_use]
    pub fn with_schema(mut self, schema: &Value) -> Self {
        let mut suggestions = Vec::new();
        let mut listed = Vec::new();
        for violation in &self.violations {
            let SchemaViolationDetail::UnexpectedProperties(unexpected) = &violation.detail else {
                continue;
            };
            let Some(known) = known_properties(schema, &violation.schema_path) else {
                continue;
            };
            for property in unexpected {
                match closest(property, &known) {
                    Some(suggestion) => suggestions.push(t!("diagnostics.schema.didYouMean", property = property, suggestion = suggestion).to_string()),
                    None if listed.is_empty() && !known.is_empty() && known.len() <= MAX_LISTED_PROPERTIES => {
                        listed.push(t!("diagnostics.schema.knownProperties", properties = known.join(", ")).to_string());
                    },
                    None => {},
                }
            }
        }
        suggestions.extend(listed);
        if !suggestions.is_empty() {
            self.help = Some(suggestions.join("\n"));
        }
        self
    }

    /// Attach the text the validated value came from, so the report can show each violation in
    /// place.
    ///
    /// Replaces any text attached before, so a host that knows the document a value was taken
    /// from can point at the document instead of the value.
    ///
    /// # Arguments
    ///
    /// * `text` - The document text, as JSON or YAML.
    /// * `name` - How to name the text in the report, like `input` or a file path.
    /// * `base` - The JSON pointer in `text` that the violations' instance paths are relative
    ///   to; empty when the text is the validated value itself.
    ///
    /// # Returns
    ///
    /// The error with the text attached, or unchanged when the text can't be parsed or the error
    /// was [`redacted`](Self::redacted).
    #[must_use]
    pub fn with_source(mut self, text: &str, name: &str, base: &str) -> Self {
        if self.redacted {
            return self;
        }
        let Some(map) = SourceMap::parse(text) else {
            return self;
        };
        let mut labels = Vec::new();
        for violation in &self.violations {
            labels.extend(labels_for(violation, &map, base));
        }
        if labels.is_empty() {
            return self;
        }
        self.labels = labels;
        self.source = Some(NamedSource::new(name, map.text().to_string()));
        self
    }

    /// Replace every violation message and drop the source text, for a value that holds secure
    /// data the report must not echo.
    ///
    /// The error stays redacted: a later [`with_source`](Self::with_source) is ignored.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to show in place of each violation's.
    #[must_use]
    pub fn redacted(mut self, message: &str) -> Self {
        for violation in &mut self.violations {
            violation.message = message.to_string();
            violation.alternatives.clear();
        }
        self.redacted = true;
        self.source = None;
        self.labels.clear();
        self.help = None;
        self
    }

    /// What was validated.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Where the validated value came from.
    #[must_use]
    pub fn origin(&self) -> JsonOrigin {
        self.origin
    }

    /// Whether the error was [`redacted`](Self::redacted) because the value holds secure data.
    #[must_use]
    pub fn is_redacted(&self) -> bool {
        self.redacted
    }

    /// The violations found.
    #[must_use]
    pub fn violations(&self) -> &[SchemaViolation] {
        &self.violations
    }

    /// Whether the report can show the violations in their source text.
    #[must_use]
    pub fn has_source(&self) -> bool {
        self.source.is_some()
    }
}

impl Display for SchemaValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", t!("dscerror.schema"), self.subject)?;
        for (index, violation) in self.violations.iter().enumerate() {
            write!(f, "{} {}", if index == 0 { ":" } else { ";" }, violation.message)?;
            if !violation.instance_path.is_empty() {
                write!(f, " {}", t!("diagnostics.schema.at", path = violation.instance_path))?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for SchemaValidationError {}

impl Diagnostic for SchemaValidationError {
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.help.as_ref().map(|help| Box::new(help) as Box<dyn Display + 'a>)
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        self.source.as_ref().map(|source| source as &dyn SourceCode)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        if self.labels.is_empty() {
            None
        } else {
            Some(Box::new(self.labels.iter().cloned()))
        }
    }
}

/// The labels that show a violation in the source text.
///
/// # Arguments
///
/// * `violation` - The violation.
/// * `map` - The source map of the text.
/// * `base` - The pointer the violation's instance path is relative to.
fn labels_for(violation: &SchemaViolation, map: &SourceMap, base: &str) -> Vec<LabeledSpan> {
    let pointer = format!("{base}{}", violation.instance_path);
    match &violation.detail {
        SchemaViolationDetail::UnexpectedProperties(unexpected) => {
            unexpected.iter().filter_map(|property| {
                let member = format!("{pointer}/{}", escape(property));
                let span = map.key_span(&member).or_else(|| map.span(&member))?;
                Some(label(map, span, t!("diagnostics.schema.unexpectedProperty", property = property).to_string()))
            }).collect()
        },
        SchemaViolationDetail::MissingProperty(property) => {
            let Some((_, span)) = map.nearest_span(&pointer) else {
                return Vec::new();
            };
            vec![label(map, span, t!("diagnostics.schema.missingProperty", property = property).to_string())]
        },
        SchemaViolationDetail::None => {
            if let Some(span) = map.value_span(&pointer) {
                return vec![label(map, span, violation.message.clone())];
            }
            // The text doesn't spell the value out, so point at the closest enclosing value.
            let Some((_, span)) = map.nearest_span(&pointer) else {
                return Vec::new();
            };
            let text = if violation.instance_path.is_empty() {
                violation.message.clone()
            } else {
                format!("{} {}", violation.message, t!("diagnostics.schema.at", path = violation.instance_path))
            };
            vec![label(map, span, text)]
        },
    }
}

/// A label over the first line of a span.
fn label(map: &SourceMap, span: std::ops::Range<usize>, text: String) -> LabeledSpan {
    let first_line_end = map.text()[span.clone()].find('\n').map_or(span.end, |offset| span.start + offset);
    let span = span.start..first_line_end.max(span.start);
    LabeledSpan::new(Some(text), span.start, span.len())
}

/// Escape a member name as a JSON pointer reference token.
fn escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// The property names the schema declares at the object schema a keyword belongs to.
///
/// # Arguments
///
/// * `schema` - The root schema.
/// * `schema_path` - The pointer to the keyword that failed, like `/additionalProperties`.
fn known_properties(schema: &Value, schema_path: &str) -> Option<Vec<String>> {
    let parent = &schema_path[..schema_path.rfind('/')?];
    let properties = schema.pointer(&format!("{parent}/properties"))?.as_object()?;
    Some(properties.keys().cloned().collect())
}

/// The known property that most resembles an unknown one, if any does.
///
/// # Arguments
///
/// * `property` - The unknown property.
/// * `known` - The known property names.
fn closest<'a>(property: &str, known: &'a [String]) -> Option<&'a str> {
    if let Some(same) = known.iter().find(|candidate| candidate.eq_ignore_ascii_case(property)) {
        return Some(same);
    }
    known.iter()
        .map(|candidate| (edit_distance(&property.to_lowercase(), &candidate.to_lowercase()), candidate))
        .filter(|(distance, _)| *distance <= MAX_SUGGESTION_DISTANCE)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, candidate)| candidate.as_str())
}

/// The Levenshtein distance between two strings.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        current[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            current[j + 1] = (previous[j + 1] + 1).min(current[j] + 1).min(previous[j] + cost);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dscresources::dscresource::validate_json_all;
    use serde_json::json;

    fn echo_schema() -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["output"],
            "properties": {
                "output": { "type": ["string", "object"] },
                "verbose": { "type": "boolean" }
            }
        })
    }

    fn error_for(text: &str) -> SchemaValidationError {
        let schema = echo_schema();
        let value: Value = serde_yaml::from_str(text).unwrap();
        let violations = validate_json_all("test", &schema, &value).unwrap();
        SchemaValidationError::new("input for 'Test/Echo'", violations)
            .with_schema(&schema)
            .with_source(text, "input", "")
    }

    #[test]
    fn unknown_property_is_labeled_at_its_key_with_a_suggestion() {
        let text = "{\n  \"otput\": \"hi\",\n  \"output\": \"x\"\n}";
        let error = error_for(text);
        assert!(error.has_source());
        let labels: Vec<LabeledSpan> = error.labels().unwrap().collect();
        assert_eq!(labels.len(), 1);
        assert_eq!(&text[labels[0].offset()..labels[0].offset() + labels[0].len()], "\"otput\"");
        assert_eq!(labels[0].label().unwrap(), "'otput' isn't a known property");
        assert_eq!(error.help().unwrap().to_string(), "Did you mean 'output' instead of 'otput'?");
        assert_eq!(error.to_string(), "Schema: input for 'Test/Echo': Additional properties are not allowed ('otput' was unexpected)");
    }

    #[test]
    fn missing_property_is_labeled_at_the_enclosing_object() {
        let text = "verbose: true\n";
        let error = error_for(text);
        let labels: Vec<LabeledSpan> = error.labels().unwrap().collect();
        assert_eq!(labels[0].label().unwrap(), "missing required property 'output'");
        assert_eq!(labels[0].offset(), 0);
        assert!(error.help().is_none());
    }

    #[test]
    fn wrong_type_is_labeled_at_the_value_and_named_in_the_message() {
        let text = "output: hi\nverbose: nope\n";
        let error = error_for(text);
        let labels: Vec<LabeledSpan> = error.labels().unwrap().collect();
        assert_eq!(&text[labels[0].offset()..labels[0].offset() + labels[0].len()], "nope");
        assert!(error.to_string().ends_with("is not of type \"boolean\" at '/verbose'"));
    }

    #[test]
    fn base_pointer_anchors_violations_inside_a_document() {
        let document = "resources:\n- name: a\n  type: Test/Echo\n  properties:\n    otput: hi\n";
        let schema = echo_schema();
        let properties = json!({"otput": "hi"});
        let violations = validate_json_all("test", &schema, &properties).unwrap();
        let error = SchemaValidationError::new("input for 'Test/Echo'", violations)
            .with_source(document, "demo.dsc.yaml", "/resources/0/properties");
        let labels: Vec<LabeledSpan> = error.labels().unwrap().collect();
        let unknown = labels.iter().find(|label| label.label() == Some("'otput' isn't a known property")).unwrap();
        assert_eq!(&document[unknown.offset()..unknown.offset() + unknown.len()], "otput");
    }

    #[test]
    fn redaction_drops_source_and_messages() {
        let error = error_for("output: hi\nverbose: secret\n").redacted("redacted");
        assert!(error.is_redacted());
        assert!(!error.has_source());
        assert!(error.labels().is_none());
        assert!(error.help().is_none());
        assert_eq!(error.to_string(), "Schema: input for 'Test/Echo': redacted at '/verbose'");
    }

    #[test]
    fn redaction_survives_attaching_source_again() {
        let document = "resources:\n- name: a\n  type: Test/Echo\n  properties:\n    output: hi\n    verbose: secret\n";
        let error = error_for("output: hi\nverbose: secret\n")
            .redacted("redacted")
            .with_source(document, "demo.dsc.yaml", "/resources/0/properties");
        assert!(error.is_redacted());
        assert!(!error.has_source());
        assert!(error.labels().is_none());
        assert!(!format!("{error:?}").contains("secret"));
    }

    #[test]
    fn origin_defaults_to_input() {
        let error = error_for("verbose: true\n");
        assert_eq!(error.origin(), JsonOrigin::Input);
        assert_eq!(error.with_origin(JsonOrigin::Output).origin(), JsonOrigin::Output);
    }

    #[test]
    fn lists_known_properties_without_a_close_match() {
        let error = error_for("output: hi\ncompletelyDifferent: 1\n");
        assert_eq!(error.help().unwrap().to_string(), "Known properties: output, verbose");
    }

    #[test]
    fn edit_distance_counts_edits() {
        assert_eq!(edit_distance("output", "otput"), 1);
        assert_eq!(edit_distance("keyPath", "keypath"), 1);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("same", "same"), 0);
    }
}
