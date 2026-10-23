// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{collections::HashMap, fmt::Display, path::PathBuf, str::FromStr};

use clap::ValueEnum;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{progress::ProgressFormat, schemas::transforms::idiomaticize_string_enum, types::SemanticVersion};

use super::{diagnostic::LintLevel, LintError};

/// The kind of DSC document being linted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    /// A configuration document.
    Configuration,
    /// A resource manifest.
    ResourceManifest,
    /// An extension manifest.
    ExtensionManifest,
}

impl Display for DocumentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            DocumentKind::Configuration => t!("lint.options.documentKindConfiguration"),
            DocumentKind::ResourceManifest => t!("lint.options.documentKindResourceManifest"),
            DocumentKind::ExtensionManifest => t!("lint.options.documentKindExtensionManifest"),
        };
        write!(f, "{text}")
    }
}

/// The level a rule reports at, or `off` to disable the rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema, ValueEnum)]
#[serde(rename_all = "camelCase")]
#[schemars(inline, transform = idiomaticize_string_enum)]
pub enum RuleLevel {
    /// The rule doesn't run.
    Off,
    /// The rule reports at the info level.
    Info,
    /// The rule reports at the warning level.
    Warning,
    /// The rule reports at the error level.
    Error,
}

impl RuleLevel {
    /// The diagnostic level for this rule level, or `None` when the rule is off.
    #[must_use]
    pub fn as_lint_level(self) -> Option<LintLevel> {
        match self {
            RuleLevel::Off => None,
            RuleLevel::Info => Some(LintLevel::Info),
            RuleLevel::Warning => Some(LintLevel::Warning),
            RuleLevel::Error => Some(LintLevel::Error),
        }
    }
}

impl From<LintLevel> for RuleLevel {
    fn from(level: LintLevel) -> Self {
        match level {
            LintLevel::Info => RuleLevel::Info,
            LintLevel::Warning => RuleLevel::Warning,
            LintLevel::Error => RuleLevel::Error,
        }
    }
}

impl Display for RuleLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            RuleLevel::Off => "off",
            RuleLevel::Info => "info",
            RuleLevel::Warning => "warning",
            RuleLevel::Error => "error",
        };
        write!(f, "{text}")
    }
}

impl FromStr for RuleLevel {
    type Err = LintError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text.trim().to_ascii_lowercase().as_str() {
            "off" => Ok(RuleLevel::Off),
            "info" => Ok(RuleLevel::Info),
            "warning" => Ok(RuleLevel::Warning),
            "error" => Ok(RuleLevel::Error),
            _ => Err(LintError::InvalidRuleLevel(text.to_string())),
        }
    }
}

/// Overrides the level of one rule, written as `<rule>=<level>` on the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleLevelOverride {
    /// The identifier of the rule.
    pub id: String,
    /// The level the rule reports at.
    pub level: RuleLevel,
}

impl FromStr for RuleLevelOverride {
    type Err = LintError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let Some((id, level)) = text.split_once('=') else {
            return Err(LintError::InvalidRuleOverride(text.to_string()));
        };
        let id = id.trim();
        if id.is_empty() {
            return Err(LintError::InvalidRuleOverride(text.to_string()));
        }
        Ok(Self {
            id: id.to_string(),
            level: level.parse()?,
        })
    }
}

impl Display for RuleLevelOverride {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.id, self.level)
    }
}

/// The `lint` section of the DSC settings file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LintSettings {
    /// Per-rule settings keyed by rule identifier.
    #[serde(default)]
    pub rules: HashMap<String, RuleSetting>,
}

/// The settings for one rule in the DSC settings file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RuleSetting {
    /// The level the rule reports at.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<RuleLevel>,
}

/// Options that control a lint run.
#[derive(Debug, Clone)]
pub struct LintOptions {
    /// The kind of document; detected from `$schema` when `None`.
    pub document_kind: Option<DocumentKind>,
    /// Whether to discover installed resources and run the rules that inspect them.
    pub discovery: bool,
    /// Whether to read rule levels from the DSC settings files.
    pub use_settings: bool,
    /// Rule level overrides, applied after the settings file and before the policy file.
    pub rule_levels: Vec<RuleLevelOverride>,
    /// Parameter values that would be passed to the configuration, in either input format that
    /// `dsc config` accepts.
    pub parameters: Option<Value>,
    /// The system root the configuration would be applied with, if not the running OS.
    pub system_root: Option<PathBuf>,
    /// The DSC version to check `directives.version` against.
    pub dsc_version: Option<SemanticVersion>,
    /// How to report progress during resource discovery.
    pub progress_format: ProgressFormat,
    /// The file the document was read from, recorded on each diagnostic.
    pub file: Option<String>,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            document_kind: None,
            discovery: true,
            use_settings: true,
            rule_levels: Vec::new(),
            parameters: None,
            system_root: None,
            dsc_version: None,
            progress_format: ProgressFormat::None,
            file: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("no-unused-parameters=off", "no-unused-parameters", RuleLevel::Off; "off")]
    #[test_case("duplicate-resource = Warning", "duplicate-resource", RuleLevel::Warning; "spaces and case")]
    #[test_case("x=ERROR", "x", RuleLevel::Error; "error")]
    fn parses_rule_overrides(text: &str, id: &str, level: RuleLevel) {
        let parsed: RuleLevelOverride = text.parse().unwrap();
        assert_eq!(parsed, RuleLevelOverride { id: id.to_string(), level });
    }

    #[test_case("no-equals"; "missing separator")]
    #[test_case("=off"; "missing id")]
    #[test_case("rule=loud"; "unknown level")]
    fn rejects_invalid_rule_overrides(text: &str) {
        assert!(text.parse::<RuleLevelOverride>().is_err());
    }

    #[test]
    fn settings_deserialize() {
        let settings: LintSettings = serde_json::from_value(serde_json::json!({
            "rules": { "no-unused-variables": { "level": "info" } }
        })).unwrap();
        assert_eq!(settings.rules["no-unused-variables"].level, Some(RuleLevel::Info));
    }
}
