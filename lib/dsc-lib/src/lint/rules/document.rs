// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Rules about the structure of a configuration document and its resource declarations.

use rust_i18n::t;
use serde_json::Value;

use crate::{
    lint::{
        context::LintContext,
        diagnostic::LintCategory,
        options::RuleLevel,
        pointer::JsonPointer,
        rule::{DiagnosticSink, Rule, RuleTier},
    },
    types::ResourceVersionReq,
};

/// The keys DSC recognizes under `Microsoft.DSC` metadata.
const MICROSOFT_DSC_METADATA_KEYS: [&str; 9] = [
    "duration",
    "endDatetime",
    "executionType",
    "operation",
    "restartRequired",
    "copyLoops",
    "securityContext",
    "startDatetime",
    "version",
];

/// The rules in this module.
#[must_use]
pub fn rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "duplicate-resource",
            category: LintCategory::Document,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.document.duplicateResourceDescription").to_string(),
            check: duplicate_resource,
        },
        Rule {
            id: "duplicate-resource-name",
            category: LintCategory::Document,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.document.duplicateResourceNameDescription").to_string(),
            check: duplicate_resource_name,
        },
        Rule {
            id: "empty-resources",
            category: LintCategory::Document,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.document.emptyResourcesDescription").to_string(),
            check: empty_resources,
        },
        Rule {
            id: "unknown-microsoft-dsc-metadata",
            category: LintCategory::Document,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.document.unknownMicrosoftDscMetadataDescription").to_string(),
            check: unknown_microsoft_dsc_metadata,
        },
        Rule {
            id: "api-version-alias",
            category: LintCategory::Document,
            tier: RuleTier::Static,
            default_level: RuleLevel::Info,
            description: || t!("lint.rules.document.apiVersionAliasDescription").to_string(),
            check: api_version_alias,
        },
        Rule {
            id: "date-version",
            category: LintCategory::Document,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.document.dateVersionDescription").to_string(),
            check: date_version,
        },
        Rule {
            id: "missing-require-version",
            category: LintCategory::Document,
            tier: RuleTier::Static,
            default_level: RuleLevel::Off,
            description: || t!("lint.rules.document.missingRequireVersionDescription").to_string(),
            check: missing_require_version,
        },
    ]
}

/// Report resources that share a name and a type with an earlier sibling.
///
/// Resources in copy loops are skipped because their names are only known at runtime.
fn duplicate_resource(context: &LintContext, sink: &mut DiagnosticSink) {
    for group in context.sibling_groups() {
        for (position, &later) in group.iter().enumerate() {
            let later_site = &context.resources[later];
            if later_site.resource.copy.is_some() {
                continue;
            }
            let duplicate = group[..position].iter().any(|&earlier| {
                let earlier_site = &context.resources[earlier];
                earlier_site.resource.copy.is_none()
                    && earlier_site.resource.name == later_site.resource.name
                    && earlier_site.resource.resource_type == later_site.resource.resource_type
            });
            if duplicate {
                sink.report(
                    &later_site.pointer,
                    t!(
                        "lint.rules.document.duplicateResource",
                        name = later_site.resource.name,
                        type_name = later_site.resource.resource_type
                    ).to_string(),
                );
            }
        }
    }
}

/// Report resources that share a name with an earlier sibling of a different type.
fn duplicate_resource_name(context: &LintContext, sink: &mut DiagnosticSink) {
    for group in context.sibling_groups() {
        for (position, &later) in group.iter().enumerate() {
            let later_site = &context.resources[later];
            if later_site.resource.copy.is_some() {
                continue;
            }
            let same_name = group[..position].iter().map(|&earlier| &context.resources[earlier]).find(|earlier_site| {
                earlier_site.resource.copy.is_none()
                    && earlier_site.resource.name == later_site.resource.name
                    && earlier_site.resource.resource_type != later_site.resource.resource_type
            });
            if let Some(earlier_site) = same_name {
                sink.report(
                    &later_site.pointer,
                    t!(
                        "lint.rules.document.duplicateResourceName",
                        name = later_site.resource.name,
                        first_type = earlier_site.resource.resource_type,
                        second_type = later_site.resource.resource_type
                    ).to_string(),
                );
            }
        }
    }
}

/// Report a configuration without resources.
fn empty_resources(context: &LintContext, sink: &mut DiagnosticSink) {
    if context.config.resources.is_empty() {
        sink.report(&JsonPointer::root().push("resources"), t!("lint.rules.document.emptyResources").to_string());
    }
}

/// Report unrecognized keys under `Microsoft.DSC` metadata on the document and on each resource.
fn unknown_microsoft_dsc_metadata(context: &LintContext, sink: &mut DiagnosticSink) {
    check_microsoft_dsc_metadata(context.raw, &JsonPointer::root(), sink);
    for (index, site) in context.resources.iter().enumerate() {
        if let Some(resource) = context.raw_resource(index) {
            check_microsoft_dsc_metadata(resource, &site.pointer, sink);
        }
    }
}

/// Report the unrecognized keys under the `Microsoft.DSC` metadata of one node.
///
/// # Arguments
///
/// * `node` - The document or resource node.
/// * `base` - The pointer to `node`.
/// * `sink` - Where to report.
fn check_microsoft_dsc_metadata(node: &Value, base: &JsonPointer, sink: &mut DiagnosticSink) {
    let Some(metadata) = node.get("metadata").and_then(|metadata| metadata.get("Microsoft.DSC")).and_then(Value::as_object) else {
        return;
    };
    let pointer = base.push("metadata").push("Microsoft.DSC");
    for key in metadata.keys() {
        if !MICROSOFT_DSC_METADATA_KEYS.contains(&key.as_str()) {
            sink.report(&pointer.push(key), t!("lint.rules.document.unknownMicrosoftDscMetadata", key = key).to_string());
        }
    }
}

/// Report resources written with `apiVersion` instead of `requireVersion`.
fn api_version_alias(context: &LintContext, sink: &mut DiagnosticSink) {
    for (index, site) in context.resources.iter().enumerate() {
        if context.raw_resource(index).is_some_and(|resource| resource.get("apiVersion").is_some()) {
            sink.report(&site.pointer.push("apiVersion"), t!("lint.rules.document.apiVersionAlias").to_string());
        }
    }
}

/// Report resources that require a date-based version.
fn date_version(context: &LintContext, sink: &mut DiagnosticSink) {
    for (index, site) in context.resources.iter().enumerate() {
        if let Some(version @ ResourceVersionReq::Date(_)) = &site.resource.require_version {
            sink.report(&version_pointer(context, index), t!("lint.rules.document.dateVersion", version = version).to_string());
        }
    }
}

/// Report resources that don't pin a version.
fn missing_require_version(context: &LintContext, sink: &mut DiagnosticSink) {
    for site in &context.resources {
        if site.resource.require_version.is_none() {
            sink.report(
                &site.pointer,
                t!(
                    "lint.rules.document.missingRequireVersion",
                    name = site.resource.name,
                    type_name = site.resource.resource_type
                ).to_string(),
            );
        }
    }
}

/// The pointer to the version requirement of a resource, whichever key the author used.
///
/// # Arguments
///
/// * `context` - The lint context.
/// * `index` - The index of the resource in the context's resource list.
fn version_pointer(context: &LintContext, index: usize) -> JsonPointer {
    let site = &context.resources[index];
    if context.raw_resource(index).is_some_and(|resource| resource.get("apiVersion").is_some()) {
        site.pointer.push("apiVersion")
    } else {
        site.pointer.push("requireVersion")
    }
}

#[cfg(test)]
mod tests {
    use crate::lint::test_util::{lint_yaml, rule_paths};

    #[test]
    fn reports_duplicate_resources_case_insensitively() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Same
  type: Microsoft.DSC.Debug/Echo
- name: Same
  type: microsoft.dsc.debug/echo
- name: Other
  type: Microsoft.DSC.Debug/Echo
"#);
        assert_eq!(rule_paths(&result, "duplicate-resource"), vec!["/resources/1"]);
        assert!(rule_paths(&result, "duplicate-resource-name").is_empty());
    }

    #[test]
    fn skips_copy_loops_for_duplicates() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[concat('a', copyIndex())]"
  type: Microsoft.DSC.Debug/Echo
  copy:
    name: loop
    count: 2
- name: "[concat('a', copyIndex())]"
  type: Microsoft.DSC.Debug/Echo
  copy:
    name: loop
    count: 2
"#);
        assert!(rule_paths(&result, "duplicate-resource").is_empty());
    }

    #[test]
    fn reports_duplicate_names_across_types() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Same
  type: Microsoft.DSC.Debug/Echo
- name: Same
  type: Microsoft/OSInfo
"#);
        assert_eq!(rule_paths(&result, "duplicate-resource-name"), vec!["/resources/1"]);
        assert!(rule_paths(&result, "duplicate-resource").is_empty());
    }

    #[test]
    fn nested_resources_are_separate_groups() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Same
  type: Microsoft.DSC/Group
  resources:
  - name: Same
    type: Microsoft.DSC/Group
"#);
        assert!(rule_paths(&result, "duplicate-resource").is_empty());
    }

    #[test]
    fn reports_empty_resources() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources: []
"#);
        assert_eq!(rule_paths(&result, "empty-resources"), vec!["/resources"]);
    }

    #[test]
    fn reports_unknown_microsoft_dsc_metadata_keys() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
    customKey: 1
  Other:
    anything: true
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  metadata:
    Microsoft.DSC:
      nope: 2
"#);
        assert_eq!(
            rule_paths(&result, "unknown-microsoft-dsc-metadata"),
            vec!["/metadata/Microsoft.DSC/customKey", "/resources/0/metadata/Microsoft.DSC/nope"]
        );
    }

    #[test]
    fn reports_api_version_alias_and_date_versions() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  apiVersion: "2025-01-01"
- name: b
  type: Microsoft.DSC.Debug/Echo
  requireVersion: "^1.0"
"#);
        assert_eq!(rule_paths(&result, "api-version-alias"), vec!["/resources/0/apiVersion"]);
        assert_eq!(rule_paths(&result, "date-version"), vec!["/resources/0/apiVersion"]);
    }

    #[test]
    fn missing_require_version_is_off_by_default() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
"#);
        assert!(rule_paths(&result, "missing-require-version").is_empty());
    }
}
