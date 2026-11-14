// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::schemas::{dsc_repo::DscRepoSchema, transforms::idiomaticize_string_enum};

use super::{
    context::LintContext,
    diagnostic::{LintCategory, LintDiagnostic, LintLevel, SourceLocation},
    options::RuleLevel,
    pointer::JsonPointer,
};

/// When a rule can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline, transform = idiomaticize_string_enum)]
pub enum RuleTier {
    /// The rule only needs the document.
    Static,
    /// The rule needs the installed resources, so it only runs when discovery is enabled.
    Resource,
}

impl std::fmt::Display for RuleTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            RuleTier::Static => "static",
            RuleTier::Resource => "resource",
        };
        write!(f, "{text}")
    }
}

/// The signature of a rule's check function.
pub type CheckFn = fn(&LintContext<'_>, &mut DiagnosticSink<'_>);

/// A lint rule.
pub struct Rule {
    /// The kebab-case identifier of the rule, such as `duplicate-resource`.
    pub id: &'static str,
    /// The area of the document the rule inspects.
    pub category: LintCategory,
    /// When the rule can run.
    pub tier: RuleTier,
    /// The level the rule reports at unless overridden.
    pub default_level: RuleLevel,
    /// Returns the localized description of what the rule checks.
    pub description: fn() -> String,
    /// Runs the rule against a document, reporting problems to the sink.
    pub check: CheckFn,
}

/// Describes a rule and the level it is configured to report at.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "rules", folder_path = "outputs/lint")]
#[schemars(
    transform = LintRuleInfo::transform_export_schema_uris,
    transform = LintRuleInfo::transform_schema_docs
)]
pub struct LintRuleInfo {
    /// The identifier of the rule.
    pub id: String,
    /// The area of the document the rule inspects.
    pub category: LintCategory,
    /// When the rule can run.
    pub tier: RuleTier,
    /// The level the rule reports at unless overridden.
    pub default_level: RuleLevel,
    /// The level the rule is configured to report at.
    pub level: RuleLevel,
    /// Whether the level of the rule can't be changed.
    pub fixed: bool,
    /// What the rule checks.
    pub description: String,
}

impl LintRuleInfo {
    /// Describe a rule at its configured level.
    ///
    /// # Arguments
    ///
    /// * `rule` - The rule.
    /// * `level` - The level the rule is configured to report at.
    #[must_use]
    pub fn from_rule(rule: &Rule, level: RuleLevel) -> Self {
        Self {
            id: rule.id.to_string(),
            category: rule.category,
            tier: rule.tier,
            default_level: rule.default_level,
            level,
            fixed: false,
            description: (rule.description)(),
        }
    }
}

/// Collects the diagnostics one rule reports, stamping each with the rule's identity and level
/// so a rule can't report under another name.
pub struct DiagnosticSink<'a> {
    rule: &'static str,
    category: LintCategory,
    level: LintLevel,
    diagnostics: &'a mut Vec<LintDiagnostic>,
}

impl<'a> DiagnosticSink<'a> {
    /// Create a sink for one rule.
    ///
    /// # Arguments
    ///
    /// * `rule` - The identifier of the rule that reports through this sink.
    /// * `category` - The category of the rule.
    /// * `level` - The level the rule reports at.
    /// * `diagnostics` - Where reported diagnostics are appended.
    #[must_use]
    pub fn new(rule: &'static str, category: LintCategory, level: LintLevel, diagnostics: &'a mut Vec<LintDiagnostic>) -> Self {
        Self {
            rule,
            category,
            level,
            diagnostics,
        }
    }

    /// Report a problem at a location in the document.
    ///
    /// # Arguments
    ///
    /// * `path` - The location of the problem.
    /// * `message` - A description of the problem.
    pub fn report(&mut self, path: &JsonPointer, message: String) {
        self.report_at(path, None, message);
    }

    /// Report a problem at a location in the document with a position in the source text.
    ///
    /// # Arguments
    ///
    /// * `path` - The location of the problem.
    /// * `location` - The position in the source text, when known.
    /// * `message` - A description of the problem.
    pub fn report_at(&mut self, path: &JsonPointer, location: Option<SourceLocation>, message: String) {
        self.diagnostics.push(LintDiagnostic {
            level: self.level,
            rule: self.rule.to_string(),
            category: self.category,
            message,
            path: path.to_string(),
            file: None,
            location,
        });
    }
}
