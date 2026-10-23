// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Static analysis for DSC configuration documents.
//!
//! The linter parses a document, validates it against the configuration document schema, and
//! runs a set of rules that find problems before the document is applied. Rules never invoke
//! resource operations; with discovery enabled they only read the manifests of installed
//! resources.
//!
//! ```rust,ignore
//! use dsc_lib::lint::{Linter, LintOptions};
//!
//! let mut linter = Linter::new(LintOptions::default())?;
//! let result = linter.lint(document_text)?;
//! for diagnostic in &result.diagnostics {
//!     println!("{} {} {}: {}", diagnostic.level, diagnostic.rule, diagnostic.path, diagnostic.message);
//! }
//! ```

use std::collections::HashMap;

use miette::Diagnostic;
use rust_i18n::t;
use serde_json::Value;
use thiserror::Error;
use tracing::{debug, warn};

use crate::{
    DscManager,
    configure::{
        config_doc::{Configuration, Resource, ResourceDiscoveryMode},
        get_require_adapter_from_directive,
        parameters::import_parameters,
    },
    diagnostics::SourceMap,
    discovery::discovery_trait::DiscoveryFilter,
    dscerror::DscError,
    dscresources::{dscresource::validate_json_all, resource_manifest::ResourceManifest},
    extensions::extension_manifest::ExtensionManifest,
    parser::Statement,
    progress::ProgressFormat,
    schemas::dsc_repo::{DscRepoSchema, RecognizedSchemaVersion, SchemaForm, SchemaUriPrefix},
    util::{get_setting, parse_input_to_value},
};

pub mod context;
pub mod diagnostic;
pub mod expression_index;
pub mod options;
pub mod pointer;
pub mod report;
pub mod rule;
pub mod rules;

pub use diagnostic::{LintCategory, LintDiagnostic, LintLevel, LintResult, LintSummary, SourceLocation};
pub use options::{DocumentKind, LintOptions, LintSettings, RuleLevel, RuleLevelOverride, RuleSetting};
pub use pointer::JsonPointer;
pub use report::{LintDiagnosticReport, redacted_source_map};
pub use rule::{CheckFn, DiagnosticSink, LintRuleInfo, Rule, RuleTier};

use context::{collect_resource_sites, LintContext, ResolvedResource};
use expression_index::ExpressionIndex;

/// The fixed rule that reports a document that isn't valid JSON or YAML.
pub const PARSE_ERROR_RULE: &str = "parse-error";
/// The fixed rule that reports violations of the configuration document schema.
pub const SCHEMA_INVALID_RULE: &str = "schema-invalid";
/// The fixed rule that reports a document kind the linter doesn't support yet.
pub const UNSUPPORTED_DOCUMENT_TYPE_RULE: &str = "unsupported-document-type";
/// The fixed rule that reports a resource discovery failure.
pub const DISCOVERY_FAILED_RULE: &str = "discovery-failed";

/// The rules whose level can't be changed.
const FIXED_RULES: [&str; 4] = [PARSE_ERROR_RULE, SCHEMA_INVALID_RULE, UNSUPPORTED_DOCUMENT_TYPE_RULE, DISCOVERY_FAILED_RULE];

/// Errors raised by the linter itself, as opposed to problems found in a document.
#[derive(Debug, Error, Diagnostic)]
pub enum LintError {
    /// A rule level override names a rule that doesn't exist.
    #[error("{t}: '{0}'", t = t!("lint.mod.unknownRule"))]
    UnknownRule(String),
    /// A rule level override isn't written as `<rule>=<level>`.
    #[error("{t}: '{0}'", t = t!("lint.mod.invalidRuleOverride"))]
    InvalidRuleOverride(String),
    /// A rule level isn't `off`, `info`, `warning`, or `error`.
    #[error("{t}: '{0}'", t = t!("lint.mod.invalidRuleLevel"))]
    InvalidRuleLevel(String),
    /// The `lint` section of the settings file can't be read.
    #[error("{t}: {0}", t = t!("lint.mod.invalidSettings"))]
    InvalidSettings(String),
    /// The supplied parameters aren't in a format DSC accepts.
    #[error("{t}: {0}", t = t!("lint.mod.invalidParameters"))]
    InvalidParameters(String),
}

/// A rule together with the level it reports at in this run.
struct ActiveRule {
    rule: Rule,
    level: LintLevel,
}

/// Lints configuration documents.
///
/// Create one with [`Linter::new`] and reuse it for several documents; it holds the compiled
/// expression parser and the effective rule levels.
pub struct Linter {
    options: LintOptions,
    rules: Vec<ActiveRule>,
    levels: HashMap<&'static str, RuleLevel>,
    parser: Statement,
    config_schema: Value,
}

impl Linter {
    /// Create a linter.
    ///
    /// # Arguments
    ///
    /// * `options` - The options of the run.
    ///
    /// # Errors
    ///
    /// Returns [`LintError::UnknownRule`] when an override names an unknown rule,
    /// [`LintError::InvalidSettings`] when the settings file's `lint` section is malformed, or
    /// another `DscError` when the expression parser or the document schema can't be prepared.
    pub fn new(options: LintOptions) -> Result<Self, DscError> {
        let levels = effective_levels(&options)?;
        let rules = rules::all_rules().into_iter().filter_map(|rule| {
            let level = levels.get(rule.id).copied().unwrap_or(rule.default_level).as_lint_level()?;
            Some(ActiveRule { rule, level })
        }).collect();
        let config_schema = serde_json::to_value(Configuration::generate_schema(
            RecognizedSchemaVersion::default(),
            SchemaForm::Bundled,
            SchemaUriPrefix::AkaDotMs,
        ))?;
        Ok(Self {
            options,
            rules,
            levels,
            parser: Statement::new()?,
            config_schema,
        })
    }

    /// The options of the run.
    #[must_use]
    pub fn options(&self) -> &LintOptions {
        &self.options
    }

    /// Set the file name recorded on the diagnostics of the documents linted from now on.
    ///
    /// # Arguments
    ///
    /// * `file` - The file the next documents are read from, or `None` for inline input.
    pub fn set_file(&mut self, file: Option<String>) {
        self.options.file = file;
    }

    /// Describe every rule, fixed and configurable, with the level it reports at in this run.
    #[must_use]
    pub fn rules(&self) -> Vec<LintRuleInfo> {
        let mut list = fixed_rule_infos();
        for rule in rules::all_rules() {
            let level = self.levels.get(rule.id).copied().unwrap_or(rule.default_level);
            list.push(LintRuleInfo::from_rule(&rule, level));
        }
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    /// Lint a document given as JSON or YAML text.
    ///
    /// Problems in the document are reported as diagnostics, including a document that doesn't
    /// parse. Every diagnostic gets the line and column of its location in the text.
    ///
    /// # Arguments
    ///
    /// * `document` - The document text.
    ///
    /// # Errors
    ///
    /// Returns [`LintError::InvalidParameters`] when the supplied parameters can't be imported,
    /// a `DscError::Validation` when the configuration document schema can't be compiled, or
    /// another `DscError` when discovery itself fails in a way that isn't reported as a
    /// diagnostic.
    pub fn lint(&mut self, document: &str) -> Result<LintResult, DscError> {
        match parse_document(document) {
            Ok(raw) => {
                let mut result = self.lint_parsed(&raw, Vec::new())?;
                if let Some(map) = SourceMap::parse(document) {
                    fill_locations(&mut result.diagnostics, &map);
                }
                Ok(result)
            },
            Err(diagnostic) => Ok(LintResult::from_diagnostics(stamp_file(vec![*diagnostic], &self.options.file), false)),
        }
    }

    /// Lint a document that is already parsed.
    ///
    /// # Arguments
    ///
    /// * `document` - The document.
    ///
    /// # Errors
    ///
    /// See [`lint`](Self::lint).
    pub fn lint_value(&mut self, document: &Value) -> Result<LintResult, DscError> {
        self.lint_parsed(document, Vec::new())
    }

    /// Run the pipeline on a parsed document.
    ///
    /// # Arguments
    ///
    /// * `raw` - The document.
    /// * `diagnostics` - Diagnostics already collected for the document.
    fn lint_parsed(&mut self, raw: &Value, mut diagnostics: Vec<LintDiagnostic>) -> Result<LintResult, DscError> {
        if let Some(diagnostic) = self.check_document_kind(raw) {
            diagnostics.push(diagnostic);
            return Ok(LintResult::from_diagnostics(stamp_file(diagnostics, &self.options.file), false));
        }

        for violation in validate_json_all("Configuration", &self.config_schema, raw)? {
            diagnostics.push(fixed_diagnostic(
                SCHEMA_INVALID_RULE,
                LintCategory::Document,
                LintLevel::Error,
                &JsonPointer::from_pointer(&violation.instance_path),
                None,
                violation.message,
            ));
        }

        let config: Configuration = match serde_json::from_value(raw.clone()) {
            Ok(config) => config,
            Err(err) => {
                if diagnostics.is_empty() {
                    diagnostics.push(fixed_diagnostic(
                        SCHEMA_INVALID_RULE,
                        LintCategory::Document,
                        LintLevel::Error,
                        &JsonPointer::root(),
                        None,
                        t!("lint.mod.documentInvalid", error = err).to_string(),
                    ));
                }
                return Ok(LintResult::from_diagnostics(stamp_file(diagnostics, &self.options.file), false));
            },
        };

        let supplied_parameters = match &self.options.parameters {
            Some(parameters) => Some(import_parameters(parameters).map_err(|err| LintError::InvalidParameters(err.to_string()))?),
            None => None,
        };

        let Self { options, rules, parser, .. } = self;
        let sites = collect_resource_sites(&config);
        let expressions = ExpressionIndex::build(raw, &sites, parser);
        let mut context = LintContext {
            raw,
            config: &config,
            resources: sites,
            expressions,
            options,
            functions: parser.function_dispatcher(),
            supplied_parameters,
            discovered: None,
        };

        for active in rules.iter().filter(|active| active.rule.tier == RuleTier::Static) {
            run_rule(active, &context, &mut diagnostics);
        }

        let mut discovery_ran = false;
        if options.discovery && !skips_discovery(&config) {
            match discover(&context, options.progress_format) {
                Ok(discovered) => {
                    context.discovered = Some(discovered);
                    discovery_ran = true;
                    for active in rules.iter().filter(|active| active.rule.tier == RuleTier::Resource) {
                        run_rule(active, &context, &mut diagnostics);
                    }
                },
                Err(err) => {
                    debug!("{}", t!("lint.mod.discoveryFailed", error = err));
                    diagnostics.push(fixed_diagnostic(
                        DISCOVERY_FAILED_RULE,
                        LintCategory::Resource,
                        LintLevel::Warning,
                        &JsonPointer::root(),
                        None,
                        t!("lint.mod.discoveryFailed", error = err).to_string(),
                    ));
                },
            }
        }

        Ok(LintResult::from_diagnostics(stamp_file(diagnostics, &options.file), discovery_ran))
    }

    /// Report a document whose kind the linter doesn't support.
    ///
    /// # Arguments
    ///
    /// * `raw` - The document.
    fn check_document_kind(&self, raw: &Value) -> Option<LintDiagnostic> {
        let kind = self.options.document_kind.or_else(|| detect_document_kind(raw))?;
        if kind == DocumentKind::Configuration {
            return None;
        }
        Some(fixed_diagnostic(
            UNSUPPORTED_DOCUMENT_TYPE_RULE,
            LintCategory::Document,
            LintLevel::Error,
            &JsonPointer::root().push("$schema"),
            None,
            t!("lint.mod.unsupportedDocumentType", kind = kind).to_string(),
        ))
    }
}

/// Parse a document as JSON or YAML, reporting a failure as a diagnostic.
///
/// Input that starts with `{` is reported with the JSON parser's message and position so the
/// message matches what the author wrote; everything else uses the YAML parser's. Text that is
/// empty or only whitespace is reported as empty rather than as whatever the YAML parser makes
/// of it.
///
/// # Arguments
///
/// * `document` - The document text.
///
/// # Errors
///
/// Returns the `parse-error` diagnostic when the text is empty or neither valid JSON nor valid
/// YAML.
fn parse_document(document: &str) -> Result<Value, Box<LintDiagnostic>> {
    if document.trim().is_empty() {
        return Err(Box::new(fixed_diagnostic(
            PARSE_ERROR_RULE,
            LintCategory::Document,
            LintLevel::Error,
            &JsonPointer::root(),
            None,
            t!("lint.mod.emptyDocument").to_string(),
        )));
    }
    let yaml_error = match parse_input_to_value(document) {
        Ok(value) => return Ok(value),
        Err(err) => err,
    };
    let (message, location) = match (document.trim_start().starts_with('{'), serde_json::from_str::<Value>(document), &yaml_error) {
        (true, Err(err), _) => (err.to_string(), Some(SourceLocation { line: err.line(), column: err.column() })),
        (_, _, DscError::Yaml(err)) => (err.to_string(), err.location().map(|location| SourceLocation { line: location.line(), column: location.column() })),
        (_, _, other) => (other.to_string(), None),
    };
    Err(Box::new(fixed_diagnostic(
        PARSE_ERROR_RULE,
        LintCategory::Document,
        LintLevel::Error,
        &JsonPointer::root(),
        location,
        t!("lint.mod.parseError", error = message).to_string(),
    )))
}

/// Detect the kind of document from its `$schema` URI.
///
/// # Arguments
///
/// * `raw` - The document.
///
/// # Returns
///
/// The kind, or `None` when there is no recognized `$schema`.
fn detect_document_kind(raw: &Value) -> Option<DocumentKind> {
    let uri = raw.get("$schema")?.as_str()?.to_string();
    if Configuration::is_recognized_schema_uri(&uri) {
        Some(DocumentKind::Configuration)
    } else if ResourceManifest::is_recognized_schema_uri(&uri) {
        Some(DocumentKind::ResourceManifest)
    } else if ExtensionManifest::is_recognized_schema_uri(&uri) {
        Some(DocumentKind::ExtensionManifest)
    } else {
        None
    }
}

/// Whether the document asks DSC to discover resources during deployment instead of up front.
///
/// # Arguments
///
/// * `config` - The configuration.
fn skips_discovery(config: &Configuration) -> bool {
    config.directives.as_ref().and_then(|directives| directives.resource_discovery.as_ref()) == Some(&ResourceDiscoveryMode::DuringDeployment)
}

/// The discovery filter for a resource declaration, matching what `dsc config` uses.
///
/// # Arguments
///
/// * `resource` - The resource declaration.
fn filter_for(resource: &Resource) -> DiscoveryFilter {
    DiscoveryFilter::new(
        &resource.resource_type,
        resource.require_version.clone(),
        get_require_adapter_from_directive(&resource.directives),
    )
}

/// Discover the resource types the document uses.
///
/// # Arguments
///
/// * `context` - The lint context, for its resource declarations.
/// * `progress_format` - How to report discovery progress.
///
/// # Returns
///
/// One entry per resource declaration, in the context's order.
///
/// # Errors
///
/// Returns the discovery error when resources can't be discovered at all.
fn discover(context: &LintContext, progress_format: ProgressFormat) -> Result<Vec<ResolvedResource>, DscError> {
    let mut manager = DscManager::new();
    let mut filters: Vec<DiscoveryFilter> = Vec::new();
    for site in &context.resources {
        let filter = filter_for(site.resource);
        if !filters.contains(&filter) {
            filters.push(filter);
        }
    }
    manager.find_resources(&filters, progress_format)?;

    let mut resolved = Vec::with_capacity(context.resources.len());
    for site in &context.resources {
        let found = manager.find_resource(&filter_for(site.resource))?.cloned();
        let type_exists = if found.is_some() {
            true
        } else if site.resource.require_version.is_some() {
            let any_version = DiscoveryFilter::new(&site.resource.resource_type, None, get_require_adapter_from_directive(&site.resource.directives));
            manager.find_resource(&any_version)?.is_some()
        } else {
            false
        };
        resolved.push(ResolvedResource { found, type_exists });
    }
    Ok(resolved)
}

/// Run one rule, collecting what it reports.
///
/// # Arguments
///
/// * `active` - The rule and its level.
/// * `context` - The lint context.
/// * `diagnostics` - Where reported diagnostics are appended.
fn run_rule(active: &ActiveRule, context: &LintContext, diagnostics: &mut Vec<LintDiagnostic>) {
    let mut sink = DiagnosticSink::new(active.rule.id, active.rule.category, active.level, diagnostics);
    (active.rule.check)(context, &mut sink);
}

/// Build a diagnostic for one of the fixed rules.
///
/// # Arguments
///
/// * `rule` - The rule identifier.
/// * `category` - The rule category.
/// * `level` - The level to report at.
/// * `path` - The location of the problem.
/// * `location` - The position in the source text, when known.
/// * `message` - A description of the problem.
fn fixed_diagnostic(rule: &str, category: LintCategory, level: LintLevel, path: &JsonPointer, location: Option<SourceLocation>, message: String) -> LintDiagnostic {
    LintDiagnostic {
        level,
        rule: rule.to_string(),
        category,
        message,
        path: path.to_string(),
        file: None,
        location,
    }
}

/// Record the source file on diagnostics that don't have one yet.
///
/// # Arguments
///
/// * `diagnostics` - The diagnostics.
/// * `file` - The file the document was read from, if known.
fn stamp_file(mut diagnostics: Vec<LintDiagnostic>, file: &Option<String>) -> Vec<LintDiagnostic> {
    for diagnostic in &mut diagnostics {
        if diagnostic.file.is_none() {
            diagnostic.file.clone_from(file);
        }
    }
    diagnostics
}

/// Record the line and column of each diagnostic's path on diagnostics that don't have a
/// location yet.
///
/// A path the document doesn't spell out, like a property that is missing, gets the location of
/// its closest enclosing value.
///
/// # Arguments
///
/// * `diagnostics` - The diagnostics.
/// * `map` - The source map of the document text.
fn fill_locations(diagnostics: &mut [LintDiagnostic], map: &SourceMap) {
    for diagnostic in diagnostics.iter_mut().filter(|diagnostic| diagnostic.location.is_none()) {
        if let Some((_, span)) = map.nearest_span(&diagnostic.path) {
            let (line, column) = map.location(span.start);
            diagnostic.location = Some(SourceLocation { line, column });
        }
    }
}

/// Whether a rule's level is fixed.
///
/// # Arguments
///
/// * `id` - The rule identifier.
fn is_fixed_rule(id: &str) -> bool {
    FIXED_RULES.contains(&id)
}

/// Describe the fixed rules.
fn fixed_rule_infos() -> Vec<LintRuleInfo> {
    let info = |id: &str, category: LintCategory, level: LintLevel, description: String| LintRuleInfo {
        id: id.to_string(),
        category,
        tier: if category == LintCategory::Resource { RuleTier::Resource } else { RuleTier::Static },
        default_level: level.into(),
        level: level.into(),
        fixed: true,
        description,
    };
    vec![
        info(PARSE_ERROR_RULE, LintCategory::Document, LintLevel::Error, t!("lint.mod.parseErrorDescription").to_string()),
        info(SCHEMA_INVALID_RULE, LintCategory::Document, LintLevel::Error, t!("lint.mod.schemaInvalidDescription").to_string()),
        info(UNSUPPORTED_DOCUMENT_TYPE_RULE, LintCategory::Document, LintLevel::Error, t!("lint.mod.unsupportedDocumentTypeDescription").to_string()),
        info(DISCOVERY_FAILED_RULE, LintCategory::Resource, LintLevel::Warning, t!("lint.mod.discoveryFailedDescription").to_string()),
    ]
}

/// Compute the level of every configurable rule for a run.
///
/// Precedence, lowest first: the rule's default, the settings file, the overrides in the
/// options, and the policy file.
///
/// # Arguments
///
/// * `options` - The options of the run.
///
/// # Errors
///
/// Returns [`LintError::UnknownRule`] when an override names an unknown rule, or
/// [`LintError::InvalidSettings`] when the settings can't be read.
fn effective_levels(options: &LintOptions) -> Result<HashMap<&'static str, RuleLevel>, LintError> {
    let mut levels: HashMap<&'static str, RuleLevel> = rules::all_rules().iter().map(|rule| (rule.id, rule.default_level)).collect();
    let (settings, policy) = if options.use_settings { load_settings()? } else { (None, None) };
    if let Some(settings) = settings {
        apply_settings(&mut levels, &settings);
    }
    for override_ in &options.rule_levels {
        if is_fixed_rule(&override_.id) {
            warn!("{}", t!("lint.mod.fixedRuleOverrideIgnored", rule = override_.id));
            continue;
        }
        let Some(level) = levels.get_mut(override_.id.as_str()) else {
            return Err(LintError::UnknownRule(override_.id.clone()));
        };
        *level = override_.level;
    }
    if let Some(policy) = policy {
        apply_settings(&mut levels, &policy);
    }
    Ok(levels)
}

/// Read the `lint` section of the settings and policy files.
///
/// # Returns
///
/// The settings-file section and the policy-file section; either is `None` when not configured.
///
/// # Errors
///
/// Returns [`LintError::InvalidSettings`] when a section isn't a valid `lint` setting.
fn load_settings() -> Result<(Option<LintSettings>, Option<LintSettings>), LintError> {
    let Ok(value) = get_setting("lint") else {
        return Ok((None, None));
    };
    Ok((parse_settings(value.setting)?, parse_settings(value.policy)?))
}

/// Parse one `lint` settings value.
///
/// # Arguments
///
/// * `value` - The value, or `Null` when the file doesn't configure linting.
///
/// # Errors
///
/// Returns [`LintError::InvalidSettings`] when the value isn't a valid `lint` setting.
fn parse_settings(value: Value) -> Result<Option<LintSettings>, LintError> {
    if value.is_null() {
        return Ok(None);
    }
    serde_json::from_value(value).map(Some).map_err(|err| LintError::InvalidSettings(err.to_string()))
}

/// Apply the rule levels from a settings section, ignoring fixed and unknown rules.
///
/// # Arguments
///
/// * `levels` - The levels to update.
/// * `settings` - The settings section.
fn apply_settings(levels: &mut HashMap<&'static str, RuleLevel>, settings: &LintSettings) {
    for (id, setting) in &settings.rules {
        let Some(level) = setting.level else {
            continue;
        };
        if is_fixed_rule(id) {
            warn!("{}", t!("lint.mod.fixedRuleOverrideIgnored", rule = id));
            continue;
        }
        match levels.get_mut(id.as_str()) {
            Some(current) => *current = level,
            None => warn!("{}", t!("lint.mod.unknownRuleInSettings", rule = id)),
        }
    }
}

#[cfg(test)]
pub(crate) mod test_util {
    use super::{LintOptions, LintResult, Linter};

    /// Lint a document with discovery and the settings file disabled.
    pub(crate) fn lint_yaml(document: &str) -> LintResult {
        lint_with(document, LintOptions {
            discovery: false,
            use_settings: false,
            ..Default::default()
        })
    }

    /// Lint a document with the given options.
    pub(crate) fn lint_with(document: &str, options: LintOptions) -> LintResult {
        Linter::new(options).unwrap().lint(document).unwrap()
    }

    /// The paths of the diagnostics a rule reported, in result order.
    pub(crate) fn rule_paths(result: &LintResult, rule: &str) -> Vec<String> {
        result.diagnostics.iter().filter(|diagnostic| diagnostic.rule == rule).map(|diagnostic| diagnostic.path.clone()).collect()
    }

    /// The messages of the diagnostics a rule reported, in result order.
    pub(crate) fn rule_messages(result: &LintResult, rule: &str) -> Vec<String> {
        result.diagnostics.iter().filter(|diagnostic| diagnostic.rule == rule).map(|diagnostic| diagnostic.message.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{test_util::{lint_with, lint_yaml, rule_paths}, *};

    const CLEAN: &str = r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  greeting:
    type: string
    defaultValue: hello
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[parameters('greeting')]"
"#;

    #[test]
    fn clean_document_is_valid() {
        let result = lint_yaml(CLEAN);
        assert!(result.valid, "{:?}", result.diagnostics);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert!(!result.discovery);
    }

    #[test]
    fn reports_yaml_parse_errors_with_location() {
        let result = lint_yaml("resources:\n- name: a\n  type: [unclosed\n");
        assert_eq!(rule_paths(&result, PARSE_ERROR_RULE), vec![""]);
        assert!(result.diagnostics[0].location.is_some());
        assert!(!result.valid);
    }

    #[test]
    fn reports_an_empty_document_as_a_parse_error() {
        for document in ["", "  \n\t\n"] {
            let result = lint_yaml(document);
            assert_eq!(rule_paths(&result, PARSE_ERROR_RULE), vec![""], "{document:?}");
            assert_eq!(result.diagnostics.len(), 1, "{document:?}");
            assert!(!result.valid);
        }
    }

    #[test]
    fn reports_json_parse_errors_with_json_message() {
        let result = lint_yaml("{\"resources\": [}");
        assert_eq!(rule_paths(&result, PARSE_ERROR_RULE), vec![""]);
        let location = result.diagnostics[0].location.unwrap();
        assert_eq!(location.line, 1);
        assert!(location.column > 1);
    }

    #[test]
    fn reports_schema_violations_with_paths() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  unknownKey: true
"#);
        let paths = rule_paths(&result, SCHEMA_INVALID_RULE);
        assert!(paths.iter().any(|path| path == "/resources/0"), "{:?}", result.diagnostics);
        assert!(!result.valid);
    }

    #[test]
    fn reports_missing_schema_uri() {
        let result = lint_yaml("resources: []\n");
        assert!(!rule_paths(&result, SCHEMA_INVALID_RULE).is_empty(), "{:?}", result.diagnostics);
    }

    #[test]
    fn rejects_manifests() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json
type: Test/Example
version: 1.0.0
"#);
        assert_eq!(rule_paths(&result, UNSUPPORTED_DOCUMENT_TYPE_RULE), vec!["/$schema"]);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn overrides_change_levels_and_disable_rules() {
        let document = r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  unused:
    type: string
    defaultValue: x
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
"#;
        let warned = lint_yaml(document);
        assert_eq!(rule_paths(&warned, "no-unused-parameters"), vec!["/parameters/unused"]);
        assert_eq!(warned.diagnostics[0].level, LintLevel::Warning);

        let raised = lint_with(document, LintOptions {
            discovery: false,
            use_settings: false,
            rule_levels: vec!["no-unused-parameters=error".parse().unwrap()],
            ..Default::default()
        });
        assert_eq!(raised.diagnostics[0].level, LintLevel::Error);
        assert!(!raised.valid);

        let silenced = lint_with(document, LintOptions {
            discovery: false,
            use_settings: false,
            rule_levels: vec!["no-unused-parameters=off".parse().unwrap()],
            ..Default::default()
        });
        assert!(rule_paths(&silenced, "no-unused-parameters").is_empty());
    }

    #[test]
    fn unknown_rule_override_is_an_error() {
        let result = Linter::new(LintOptions {
            use_settings: false,
            rule_levels: vec!["no-such-rule=off".parse().unwrap()],
            ..Default::default()
        });
        assert!(matches!(result, Err(DscError::Lint(LintError::UnknownRule(id))) if id == "no-such-rule"));
    }

    #[test]
    fn fixed_rule_override_is_ignored() {
        let linter = Linter::new(LintOptions {
            use_settings: false,
            rule_levels: vec!["parse-error=off".parse().unwrap()],
            ..Default::default()
        }).unwrap();
        let result = linter.rules();
        let parse_error = result.iter().find(|rule| rule.id == PARSE_ERROR_RULE).unwrap();
        assert!(parse_error.fixed);
        assert_eq!(parse_error.level, RuleLevel::Error);
    }

    #[test]
    fn lists_rules_sorted_with_effective_levels() {
        let linter = Linter::new(LintOptions {
            use_settings: false,
            rule_levels: vec!["duplicate-resource=info".parse().unwrap()],
            ..Default::default()
        }).unwrap();
        let rules = linter.rules();
        let ids: Vec<&str> = rules.iter().map(|rule| rule.id.as_str()).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
        let duplicate = rules.iter().find(|rule| rule.id == "duplicate-resource").unwrap();
        assert_eq!(duplicate.default_level, RuleLevel::Error);
        assert_eq!(duplicate.level, RuleLevel::Info);
        assert!(!duplicate.fixed);
        assert!(rules.iter().any(|rule| rule.tier == RuleTier::Resource));
    }

    #[test]
    fn invalid_supplied_parameters_are_an_error() {
        let result = Linter::new(LintOptions {
            discovery: false,
            use_settings: false,
            parameters: Some(json!({"not": "parameters"})),
            ..Default::default()
        }).unwrap().lint(CLEAN);
        assert!(matches!(result, Err(DscError::Lint(LintError::InvalidParameters(_)))));
    }

    #[test]
    fn records_the_file_on_diagnostics() {
        let result = lint_with("resources: []\n", LintOptions {
            discovery: false,
            use_settings: false,
            file: Some("config.yaml".to_string()),
            ..Default::default()
        });
        assert!(result.diagnostics.iter().all(|diagnostic| diagnostic.file.as_deref() == Some("config.yaml")));
    }

    #[test]
    fn during_deployment_discovery_skips_the_resource_tier() {
        let result = lint_with(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
directives:
  resourceDiscovery: duringDeployment
resources:
- name: a
  type: Not.Installed/Anywhere
"#, LintOptions {
            use_settings: false,
            ..Default::default()
        });
        assert!(!result.discovery);
        assert!(rule_paths(&result, "unknown-resource-type").is_empty());
    }
}
