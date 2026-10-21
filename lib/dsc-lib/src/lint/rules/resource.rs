// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Rules that inspect the installed resources a configuration document uses. They only run
//! when discovery is enabled and never invoke a resource operation.

use rust_i18n::t;
use serde_json::Value;

use crate::{
    dscresources::{
        dscresource::{validate_json_all, Capability, DscResource, ImplementedAs, SchemaViolation},
        resource_manifest::{Kind, SchemaKind},
    },
    lint::{
        context::{LintContext, ResolvedResource},
        diagnostic::LintCategory,
        expression_index::ExpressionScope,
        options::RuleLevel,
        pointer::JsonPointer,
        rule::{DiagnosticSink, Rule, RuleTier},
    },
    parser::ParsedStatement,
};

/// The rules in this module.
#[must_use]
pub fn rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "unknown-resource-type",
            category: LintCategory::Resource,
            tier: RuleTier::Resource,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.resource.unknownResourceTypeDescription").to_string(),
            check: unknown_resource_type,
        },
        Rule {
            id: "unsatisfied-require-version",
            category: LintCategory::Resource,
            tier: RuleTier::Resource,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.resource.unsatisfiedRequireVersionDescription").to_string(),
            check: unsatisfied_require_version,
        },
        Rule {
            id: "deprecated-resource",
            category: LintCategory::Resource,
            tier: RuleTier::Resource,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.resource.deprecatedResourceDescription").to_string(),
            check: deprecated_resource,
        },
        Rule {
            id: "property-schema-mismatch",
            category: LintCategory::Resource,
            tier: RuleTier::Resource,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.resource.propertySchemaMismatchDescription").to_string(),
            check: property_schema_mismatch,
        },
        Rule {
            id: "nested-resources-unsupported",
            category: LintCategory::Resource,
            tier: RuleTier::Resource,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.resource.nestedResourcesUnsupportedDescription").to_string(),
            check: nested_resources_unsupported,
        },
        Rule {
            id: "exist-false-unsupported",
            category: LintCategory::Resource,
            tier: RuleTier::Resource,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.resource.existFalseUnsupportedDescription").to_string(),
            check: exist_false_unsupported,
        },
        Rule {
            id: "canonical-type-name",
            category: LintCategory::Resource,
            tier: RuleTier::Resource,
            default_level: RuleLevel::Info,
            description: || t!("lint.rules.resource.canonicalTypeNameDescription").to_string(),
            check: canonical_type_name,
        },
    ]
}

/// The resource declarations paired with what discovery found for them.
///
/// # Arguments
///
/// * `context` - The lint context.
///
/// # Returns
///
/// One entry per resource declaration, or an empty list when discovery didn't run.
fn resolved<'a>(context: &'a LintContext<'_>) -> Vec<(usize, &'a ResolvedResource)> {
    context.discovered.as_ref().map_or_else(Vec::new, |discovered| discovered.iter().enumerate().collect())
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

/// The schema for a resource's properties without invoking the resource: the schema discovery
/// loaded, or the schema embedded in the manifest.
///
/// # Arguments
///
/// * `resource` - The discovered resource.
fn embedded_schema(resource: &DscResource) -> Option<Value> {
    if let Some(schema) = &resource.schema {
        return Some(Value::Object(schema.clone()));
    }
    if let Some(SchemaKind::Embedded(schema)) = resource.manifest.as_ref().and_then(|manifest| manifest.schema.as_ref()) {
        return Some(schema.clone());
    }
    None
}

/// Report resource types that aren't installed.
fn unknown_resource_type(context: &LintContext, sink: &mut DiagnosticSink) {
    for (index, resolved) in resolved(context) {
        if resolved.found.is_none() && !resolved.type_exists {
            let site = &context.resources[index];
            sink.report(&site.pointer.push("type"), t!("lint.rules.resource.unknownResourceType", type_name = site.resource.resource_type).to_string());
        }
    }
}

/// Report version requirements that no installed version of the type satisfies.
fn unsatisfied_require_version(context: &LintContext, sink: &mut DiagnosticSink) {
    for (index, resolved) in resolved(context) {
        if resolved.found.is_none() && resolved.type_exists
            && let Some(version) = &context.resources[index].resource.require_version {
                sink.report(
                    &version_pointer(context, index),
                    t!(
                        "lint.rules.resource.unsatisfiedRequireVersion",
                        type_name = context.resources[index].resource.resource_type,
                        version = version
                    ).to_string(),
                );
            }
    }
}

/// Report resources whose type is deprecated.
fn deprecated_resource(context: &LintContext, sink: &mut DiagnosticSink) {
    for (index, resolved) in resolved(context) {
        if let Some(message) = resolved.found.as_ref().and_then(|found| found.deprecation_message.as_ref()) {
            let site = &context.resources[index];
            sink.report(
                &site.pointer.push("type"),
                t!("lint.rules.resource.deprecatedResource", type_name = site.resource.resource_type, message = message).to_string(),
            );
        }
    }
}

/// Keywords whose outcome depends only on the shape of a value (its type, member names, or
/// length) and not on the values nested in it, so an expression nested in the value can't
/// change it.
const STRUCTURAL_KEYWORDS: [&str; 9] = [
    "type",
    "required",
    "additionalProperties",
    "propertyNames",
    "minProperties",
    "maxProperties",
    "dependentRequired",
    "minItems",
    "maxItems",
];

/// The properties of a resource as DSC passes them to the resource, as far as that is known
/// statically.
///
/// # Arguments
///
/// * `context` - The lint context.
/// * `index` - The index of the resource in the context's resource list.
/// * `properties` - The properties as written.
///
/// # Returns
///
/// The properties with escaped strings (`[[...`) replaced by their value, and the pointers of
/// the expressions, whose values are only known at runtime.
fn runtime_properties(context: &LintContext, index: usize, properties: &Value) -> (Value, Vec<JsonPointer>) {
    let base = context.resources[index].pointer.push("properties");
    let mut value = properties.clone();
    let mut unknown = Vec::new();
    for site in context.expressions.sites() {
        if site.scope != ExpressionScope::Resource(index) || !site.pointer.starts_with(&base) {
            continue;
        }
        match &site.parsed {
            Ok(ParsedStatement::Escaped(text)) => {
                let relative = &site.pointer.as_str()[base.as_str().len()..];
                if let Some(slot) = value.pointer_mut(relative) {
                    *slot = Value::String(text.clone());
                }
            },
            Ok(ParsedStatement::Literal(_)) => {},
            Ok(ParsedStatement::Expression(_)) | Err(_) => unknown.push(site.pointer.clone()),
        }
    }
    (value, unknown)
}

/// Whether a violation holds whatever the expressions in the properties evaluate to.
///
/// A violation at or below an expression is unknown until runtime. Above an expression, only
/// violations of a structural keyword are definite. A failed `anyOf` or `oneOf` is definite only
/// when every subschema has a definite violation, because otherwise the runtime values could
/// satisfy one of them.
///
/// # Arguments
///
/// * `violation` - The violation.
/// * `base` - The pointer to the properties the violation paths are relative to.
/// * `unknown` - The pointers of the expressions in the properties.
fn is_definite(violation: &SchemaViolation, base: &JsonPointer, unknown: &[JsonPointer]) -> bool {
    if !violation.alternatives.is_empty() {
        return violation.alternatives.iter().all(|alternative| alternative.iter().any(|nested| is_definite(nested, base, unknown)));
    }
    let pointer = base.join(&violation.instance_path);
    if unknown.iter().any(|expression| pointer.starts_with(expression)) {
        return false;
    }
    let contains_expression = unknown.iter().any(|expression| expression.starts_with(&pointer));
    !contains_expression || STRUCTURAL_KEYWORDS.contains(&violation.keyword.as_str())
}

/// Report property values that don't validate against the resource's embedded schema.
///
/// The properties are validated as DSC passes them to the resource, with escaped strings
/// unescaped. Violations that an expression's runtime value could resolve are dropped; see
/// [`is_definite`].
fn property_schema_mismatch(context: &LintContext, sink: &mut DiagnosticSink) {
    for (index, resolved) in resolved(context) {
        let Some(found) = &resolved.found else {
            continue;
        };
        if found.implemented_as != Some(ImplementedAs::Command) {
            continue;
        }
        let Some(schema) = embedded_schema(found) else {
            continue;
        };
        let site = &context.resources[index];
        let base = site.pointer.push("properties");
        let Some(properties) = base.resolve(context.raw) else {
            continue;
        };
        let (properties, unknown) = runtime_properties(context, index, properties);
        let violations = match validate_json_all(found.type_name.as_ref(), &schema, &properties) {
            Ok(violations) => violations,
            Err(err) => {
                sink.report(
                    &base,
                    t!("lint.rules.resource.propertySchemaCompileFailed", type_name = site.resource.resource_type, error = LintContext::error_message(&err)).to_string(),
                );
                continue;
            },
        };
        for violation in violations {
            if !is_definite(&violation, &base, &unknown) {
                continue;
            }
            sink.report(
                &base.join(&violation.instance_path),
                t!("lint.rules.resource.propertySchemaMismatch", type_name = site.resource.resource_type, message = violation.message).to_string(),
            );
        }
    }
}

/// Report nested resources under a type that isn't a group or adapter.
fn nested_resources_unsupported(context: &LintContext, sink: &mut DiagnosticSink) {
    for (index, resolved) in resolved(context) {
        let site = &context.resources[index];
        if site.resource.resources.is_none() {
            continue;
        }
        let Some(found) = &resolved.found else {
            continue;
        };
        if !matches!(found.kind, Kind::Group | Kind::Adapter) {
            sink.report(
                &site.pointer.push("resources"),
                t!(
                    "lint.rules.resource.nestedResourcesUnsupported",
                    name = site.resource.name,
                    type_name = site.resource.resource_type,
                    kind = format!("{:?}", found.kind).to_lowercase()
                ).to_string(),
            );
        }
    }
}

/// Report `_exist: false` on a type that can't delete instances.
fn exist_false_unsupported(context: &LintContext, sink: &mut DiagnosticSink) {
    for (index, resolved) in resolved(context) {
        let site = &context.resources[index];
        let exists = site.resource.properties.as_ref().and_then(|properties| properties.get("_exist"));
        if exists != Some(&Value::Bool(false)) {
            continue;
        }
        let Some(found) = &resolved.found else {
            continue;
        };
        let can_delete = found.capabilities.contains(&Capability::Delete) || found.capabilities.contains(&Capability::SetHandlesExist);
        if !can_delete {
            sink.report(
                &site.pointer.push("properties").push("_exist"),
                t!("lint.rules.resource.existFalseUnsupported", type_name = site.resource.resource_type).to_string(),
            );
        }
    }
}

/// Report resource types written with different casing than their manifest.
fn canonical_type_name(context: &LintContext, sink: &mut DiagnosticSink) {
    for (index, resolved) in resolved(context) {
        let Some(found) = &resolved.found else {
            continue;
        };
        let site = &context.resources[index];
        let written: &str = site.resource.resource_type.as_ref();
        let canonical: &str = found.type_name.as_ref();
        if written != canonical {
            sink.report(
                &site.pointer.push("type"),
                t!("lint.rules.resource.canonicalTypeName", type_name = written, canonical = canonical).to_string(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use crate::{
        configure::config_doc::Configuration,
        dscresources::dscresource::{Capability, DscResource, ImplementedAs},
        dscresources::resource_manifest::Kind,
        lint::{
            context::{collect_resource_sites, LintContext, ResolvedResource},
            diagnostic::{LintDiagnostic, LintLevel},
            expression_index::ExpressionIndex,
            options::LintOptions,
            rule::DiagnosticSink,
        },
        parser::Statement,
        types::FullyQualifiedTypeName,
    };

    use super::rules;

    fn echo(kind: Kind, capabilities: Vec<Capability>, schema: Option<Value>) -> DscResource {
        DscResource {
            type_name: FullyQualifiedTypeName::parse("Microsoft.DSC.Debug/Echo").unwrap(),
            kind,
            capabilities,
            implemented_as: Some(ImplementedAs::Command),
            schema: schema.and_then(|schema| schema.as_object().cloned()),
            ..DscResource::new()
        }
    }

    fn run(document: &str, discovered: Vec<ResolvedResource>, rule_id: &str) -> Vec<LintDiagnostic> {
        let raw: Value = serde_yaml::from_str(document).unwrap();
        let config: Configuration = serde_json::from_value(raw.clone()).unwrap();
        let options = LintOptions::default();
        let mut parser = Statement::new().unwrap();
        let sites = collect_resource_sites(&config);
        let expressions = ExpressionIndex::build(&raw, &sites, &mut parser);
        let context = LintContext {
            raw: &raw,
            config: &config,
            resources: sites,
            expressions,
            options: &options,
            functions: parser.function_dispatcher(),
            supplied_parameters: None,
            discovered: Some(discovered),
        };
        let rule = rules().into_iter().find(|rule| rule.id == rule_id).unwrap();
        let mut diagnostics = Vec::new();
        let mut sink = DiagnosticSink::new(rule.id, rule.category, LintLevel::Error, &mut diagnostics);
        (rule.check)(&context, &mut sink);
        diagnostics
    }

    fn paths(diagnostics: &[LintDiagnostic]) -> Vec<&str> {
        diagnostics.iter().map(|diagnostic| diagnostic.path.as_str()).collect()
    }

    const DOCUMENT: &str = r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: a
  type: microsoft.dsc.debug/echo
  requireVersion: "^9.0"
  properties:
    output: literal
    dynamic: "[parameters('x')]"
    _exist: false
  resources:
  - name: nested
    type: Microsoft.DSC.Debug/Echo
"#;

    #[test]
    fn reports_unknown_types_and_unsatisfied_versions() {
        let discovered = vec![
            ResolvedResource { found: None, type_exists: true },
            ResolvedResource { found: None, type_exists: false },
        ];
        assert_eq!(paths(&run(DOCUMENT, discovered, "unknown-resource-type")), vec!["/resources/0/resources/0/type"]);
        let discovered = vec![
            ResolvedResource { found: None, type_exists: true },
            ResolvedResource { found: None, type_exists: false },
        ];
        assert_eq!(paths(&run(DOCUMENT, discovered, "unsatisfied-require-version")), vec!["/resources/0/requireVersion"]);
    }

    #[test]
    fn reports_deprecated_resources_and_casing() {
        let mut deprecated = echo(Kind::Resource, vec![Capability::Get], None);
        deprecated.deprecation_message = Some("use something else".to_string());
        let discovered = vec![
            ResolvedResource { found: Some(deprecated), type_exists: true },
            ResolvedResource { found: Some(echo(Kind::Resource, vec![Capability::Get], None)), type_exists: true },
        ];
        assert_eq!(paths(&run(DOCUMENT, discovered, "deprecated-resource")), vec!["/resources/0/type"]);
        let discovered = vec![
            ResolvedResource { found: Some(echo(Kind::Resource, vec![Capability::Get], None)), type_exists: true },
            ResolvedResource { found: Some(echo(Kind::Resource, vec![Capability::Get], None)), type_exists: true },
        ];
        assert_eq!(paths(&run(DOCUMENT, discovered, "canonical-type-name")), vec!["/resources/0/type"]);
    }

    #[test]
    fn reports_schema_violations_outside_expressions() {
        let schema = json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "output": { "type": "integer" },
                "dynamic": { "type": "integer" },
                "_exist": { "type": "boolean" }
            }
        });
        let discovered = vec![
            ResolvedResource { found: Some(echo(Kind::Resource, vec![Capability::Get], Some(schema))), type_exists: true },
            ResolvedResource { found: Some(echo(Kind::Resource, vec![Capability::Get], None)), type_exists: true },
        ];
        assert_eq!(paths(&run(DOCUMENT, discovered, "property-schema-mismatch")), vec!["/resources/0/properties/output"]);
    }

    #[test]
    fn combinators_allow_expressions_but_keep_definite_errors() {
        let document = r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    items:
    - name: dynamic
      scope: "[parameters('scope')]"
    - name: literal
      scope: nowhere
    - name: brokenSibling
      scope: "[parameters('scope')]"
      count: notAnInt
    count: "[[5]"
    mode: "[[x]"
"#;
        let branch = |scope: &str| json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "scope": { "const": scope },
                "count": { "type": "integer" }
            }
        });
        let schema = json!({
            "type": "object",
            "properties": {
                "items": { "type": "array", "items": { "anyOf": [branch("currentUser"), branch("machine")] } },
                "count": { "type": "integer" },
                "mode": { "enum": ["[x]"] }
            }
        });
        let discovered = vec![
            ResolvedResource { found: Some(echo(Kind::Resource, vec![Capability::Get], Some(schema))), type_exists: true },
        ];
        let diagnostics = run(document, discovered, "property-schema-mismatch");
        let mut reported = paths(&diagnostics);
        reported.sort_unstable();
        assert_eq!(reported, vec!["/resources/0/properties/count", "/resources/0/properties/items/1", "/resources/0/properties/items/2"]);
    }

    #[test]
    fn reports_nested_resources_and_exist_false_on_unsupported_types() {
        let discovered = vec![
            ResolvedResource { found: Some(echo(Kind::Resource, vec![Capability::Get, Capability::Set], None)), type_exists: true },
            ResolvedResource { found: Some(echo(Kind::Resource, vec![Capability::Get], None)), type_exists: true },
        ];
        assert_eq!(paths(&run(DOCUMENT, discovered, "nested-resources-unsupported")), vec!["/resources/0/resources"]);
        let discovered = vec![
            ResolvedResource { found: Some(echo(Kind::Resource, vec![Capability::Get, Capability::Set], None)), type_exists: true },
            ResolvedResource { found: Some(echo(Kind::Resource, vec![Capability::Get], None)), type_exists: true },
        ];
        assert_eq!(paths(&run(DOCUMENT, discovered, "exist-false-unsupported")), vec!["/resources/0/properties/_exist"]);
        let discovered = vec![
            ResolvedResource { found: Some(echo(Kind::Group, vec![Capability::Get, Capability::Delete], None)), type_exists: true },
            ResolvedResource { found: Some(echo(Kind::Resource, vec![Capability::Get], None)), type_exists: true },
        ];
        assert!(run(DOCUMENT, discovered, "nested-resources-unsupported").is_empty());
        let discovered = vec![
            ResolvedResource { found: Some(echo(Kind::Group, vec![Capability::Get, Capability::Delete], None)), type_exists: true },
            ResolvedResource { found: Some(echo(Kind::Resource, vec![Capability::Get], None)), type_exists: true },
        ];
        assert!(run(DOCUMENT, discovered, "exist-false-unsupported").is_empty());
    }
}
