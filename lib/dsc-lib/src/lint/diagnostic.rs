// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::fmt::Display;

use clap::ValueEnum;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::schemas::{dsc_repo::DscRepoSchema, transforms::idiomaticize_string_enum};

/// The severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, JsonSchema, ValueEnum)]
#[serde(rename_all = "camelCase")]
#[schemars(inline, transform = idiomaticize_string_enum)]
pub enum LintLevel {
    /// A suggestion that doesn't affect whether the document works.
    Info,
    /// A problem that is likely to cause unexpected behavior.
    Warning,
    /// A problem that prevents the document from working.
    Error,
}

impl Display for LintLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            LintLevel::Info => "info",
            LintLevel::Warning => "warning",
            LintLevel::Error => "error",
        };
        write!(f, "{text}")
    }
}

/// The area of a document that a rule inspects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline, transform = idiomaticize_string_enum)]
pub enum LintCategory {
    /// The structure of the document and its resources.
    Document,
    /// Expressions and the functions they call.
    Expression,
    /// Parameter and variable declarations and their values.
    Parameter,
    /// Dependencies between resources.
    Dependency,
    /// Features that DSC has deprecated or doesn't support.
    Deprecation,
    /// The environment the document targets.
    Environment,
    /// Checks that need the installed resources.
    Resource,
}

impl Display for LintCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            LintCategory::Document => "document",
            LintCategory::Expression => "expression",
            LintCategory::Parameter => "parameter",
            LintCategory::Dependency => "dependency",
            LintCategory::Deprecation => "deprecation",
            LintCategory::Environment => "environment",
            LintCategory::Resource => "resource",
        };
        write!(f, "{text}")
    }
}

/// A position in the source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(inline)]
pub struct SourceLocation {
    /// The line number, starting at 1.
    pub line: usize,
    /// The column number, starting at 1.
    pub column: usize,
}

/// A problem found in a document.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(inline)]
pub struct LintDiagnostic {
    /// The severity of the problem.
    pub level: LintLevel,
    /// The identifier of the rule that reported the problem.
    pub rule: String,
    /// The category of the rule that reported the problem.
    pub category: LintCategory,
    /// A description of the problem.
    pub message: String,
    /// JSON pointer (RFC 6901) to the location of the problem in the document. An empty string
    /// identifies the document root.
    pub path: String,
    /// The file the document was read from, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// The position of the problem in the source text, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
}

/// Counts of diagnostics by level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(inline)]
pub struct LintSummary {
    /// The number of error-level diagnostics.
    pub errors: usize,
    /// The number of warning-level diagnostics.
    pub warnings: usize,
    /// The number of info-level diagnostics.
    pub info: usize,
}

/// The result of linting one or more documents.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(base_name = "result", folder_path = "outputs/lint")]
#[schemars(
    transform = LintResult::transform_export_schema_uris,
    transform = LintResult::transform_schema_docs
)]
pub struct LintResult {
    /// The problems found, ordered by path and rule.
    pub diagnostics: Vec<LintDiagnostic>,
    /// Counts of the problems found by level.
    pub summary: LintSummary,
    /// Whether no error-level problems were found.
    pub valid: bool,
    /// Whether resource discovery ran, which enables the rules that inspect installed resources.
    pub discovery: bool,
}

impl LintResult {
    /// Build a result from a set of diagnostics, sorting them and computing the summary.
    ///
    /// # Arguments
    ///
    /// * `diagnostics` - The diagnostics found.
    /// * `discovery` - Whether resource discovery ran.
    #[must_use]
    pub fn from_diagnostics(mut diagnostics: Vec<LintDiagnostic>, discovery: bool) -> Self {
        diagnostics.sort_by(|a, b| a.file.cmp(&b.file).then_with(|| a.path.cmp(&b.path)).then_with(|| a.rule.cmp(&b.rule)));
        let mut summary = LintSummary::default();
        for diagnostic in &diagnostics {
            match diagnostic.level {
                LintLevel::Error => summary.errors += 1,
                LintLevel::Warning => summary.warnings += 1,
                LintLevel::Info => summary.info += 1,
            }
        }
        Self {
            valid: summary.errors == 0,
            discovery,
            summary,
            diagnostics,
        }
    }

    /// Combine another result into this one, for example when linting several files.
    ///
    /// # Arguments
    ///
    /// * `other` - The result to merge; its diagnostics are appended and re-sorted.
    pub fn merge(&mut self, other: LintResult) {
        let discovery = self.discovery && other.discovery;
        let mut diagnostics = std::mem::take(&mut self.diagnostics);
        diagnostics.extend(other.diagnostics);
        *self = Self::from_diagnostics(diagnostics, discovery);
    }

    /// Whether any diagnostic is at or above the given level.
    ///
    /// # Arguments
    ///
    /// * `level` - The threshold.
    #[must_use]
    pub fn has_diagnostic_at_least(&self, level: LintLevel) -> bool {
        self.diagnostics.iter().any(|diagnostic| diagnostic.level >= level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diagnostic(level: LintLevel, rule: &str, path: &str) -> LintDiagnostic {
        LintDiagnostic {
            level,
            rule: rule.to_string(),
            category: LintCategory::Document,
            message: String::new(),
            path: path.to_string(),
            file: None,
            location: None,
        }
    }

    #[test]
    fn from_diagnostics_sorts_and_summarizes() {
        let result = LintResult::from_diagnostics(vec![
            diagnostic(LintLevel::Warning, "b", "/resources/1"),
            diagnostic(LintLevel::Error, "a", "/resources/0"),
            diagnostic(LintLevel::Info, "a", "/resources/1"),
        ], true);
        assert_eq!(result.summary, LintSummary { errors: 1, warnings: 1, info: 1 });
        assert!(!result.valid);
        assert!(result.discovery);
        let order: Vec<(&str, &str)> = result.diagnostics.iter().map(|d| (d.path.as_str(), d.rule.as_str())).collect();
        assert_eq!(order, vec![("/resources/0", "a"), ("/resources/1", "a"), ("/resources/1", "b")]);
    }

    #[test]
    fn merge_combines_results() {
        let mut first = LintResult::from_diagnostics(vec![diagnostic(LintLevel::Warning, "a", "")], true);
        let second = LintResult::from_diagnostics(vec![diagnostic(LintLevel::Error, "b", "")], false);
        first.merge(second);
        assert_eq!(first.diagnostics.len(), 2);
        assert!(!first.valid);
        assert!(!first.discovery);
        assert!(first.has_diagnostic_at_least(LintLevel::Error));
    }

    #[test]
    fn levels_are_ordered() {
        assert!(LintLevel::Error > LintLevel::Warning);
        assert!(LintLevel::Warning > LintLevel::Info);
    }
}
