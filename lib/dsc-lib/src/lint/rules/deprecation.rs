// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Rules about features DSC has deprecated or doesn't support.

use rust_i18n::t;

use crate::{
    configure::config_doc::ValueOrCopy,
    lint::{
        context::LintContext,
        diagnostic::LintCategory,
        options::RuleLevel,
        pointer::JsonPointer,
        rule::{DiagnosticSink, Rule, RuleTier},
    },
};

/// The rules in this module.
#[must_use]
pub fn rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "copy-loop-deprecated",
            category: LintCategory::Deprecation,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.deprecation.copyLoopDeprecatedDescription").to_string(),
            check: copy_loop_deprecated,
        },
        Rule {
            id: "copy-options-unsupported",
            category: LintCategory::Deprecation,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.deprecation.copyOptionsUnsupportedDescription").to_string(),
            check: copy_options_unsupported,
        },
        Rule {
            id: "security-context-in-metadata",
            category: LintCategory::Deprecation,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.deprecation.securityContextInMetadataDescription").to_string(),
            check: security_context_in_metadata,
        },
        Rule {
            id: "security-context-conflict",
            category: LintCategory::Deprecation,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.deprecation.securityContextConflictDescription").to_string(),
            check: security_context_conflict,
        },
        Rule {
            id: "dsc-version-unsatisfied",
            category: LintCategory::Deprecation,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.deprecation.dscVersionUnsatisfiedDescription").to_string(),
            check: dsc_version_unsatisfied,
        },
    ]
}

/// Report copy loops on resources and outputs.
fn copy_loop_deprecated(context: &LintContext, sink: &mut DiagnosticSink) {
    for site in &context.resources {
        if let Some(copy) = &site.resource.copy {
            sink.report(&site.pointer.push("copy"), t!("lint.rules.deprecation.copyLoopDeprecated", name = copy.name).to_string());
        }
    }
    if let Some(outputs) = &context.config.outputs {
        for (name, output) in outputs {
            if let ValueOrCopy::Copy(copy) = &output.value_or_copy {
                sink.report(
                    &JsonPointer::root().push("outputs").push(name).push("copy"),
                    t!("lint.rules.deprecation.copyLoopDeprecated", name = copy.name).to_string(),
                );
            }
        }
    }
}

/// Report copy loop options that DSC rejects.
fn copy_options_unsupported(context: &LintContext, sink: &mut DiagnosticSink) {
    for site in &context.resources {
        let Some(copy) = &site.resource.copy else {
            continue;
        };
        let base = site.pointer.push("copy");
        if copy.mode.is_some() {
            sink.report(&base.push("mode"), t!("lint.rules.deprecation.copyOptionsUnsupported", option = "mode").to_string());
        }
        if copy.batch_size.is_some() {
            sink.report(&base.push("batchSize"), t!("lint.rules.deprecation.copyOptionsUnsupported", option = "batchSize").to_string());
        }
    }
}

/// Report a security context set through `Microsoft.DSC` metadata.
fn security_context_in_metadata(context: &LintContext, sink: &mut DiagnosticSink) {
    if metadata_security_context(context).is_some() {
        sink.report(
            &JsonPointer::root().push("metadata").push("Microsoft.DSC").push("securityContext"),
            t!("lint.rules.deprecation.securityContextInMetadata").to_string(),
        );
    }
}

/// Report a security context in metadata that contradicts the directive.
fn security_context_conflict(context: &LintContext, sink: &mut DiagnosticSink) {
    let directive = context.config.directives.as_ref().and_then(|directives| directives.security_context.as_ref());
    if let (Some(metadata), Some(directive)) = (metadata_security_context(context), directive)
        && metadata != directive {
            sink.report(
                &JsonPointer::root().push("directives").push("securityContext"),
                t!("lint.rules.deprecation.securityContextConflict", metadata = metadata, directive = directive).to_string(),
            );
        }
}

/// Report a `directives.version` requirement the linting DSC doesn't satisfy.
fn dsc_version_unsatisfied(context: &LintContext, sink: &mut DiagnosticSink) {
    let Some(current) = &context.options.dsc_version else {
        return;
    };
    let Some(required) = context.config.directives.as_ref().and_then(|directives| directives.version.as_ref()) else {
        return;
    };
    if !required.matches(current) {
        sink.report(
            &JsonPointer::root().push("directives").push("version"),
            t!("lint.rules.deprecation.dscVersionUnsatisfied", required = required, current = current).to_string(),
        );
    }
}

/// The security context set through `Microsoft.DSC` metadata, if any.
///
/// # Arguments
///
/// * `context` - The lint context.
fn metadata_security_context<'a>(context: &'a LintContext<'_>) -> Option<&'a crate::configure::config_doc::SecurityContextKind> {
    context.config.metadata.as_ref()?.microsoft.as_ref()?.security_context.as_ref()
}

#[cfg(test)]
mod tests {
    use crate::{lint::{options::LintOptions, test_util::{lint_with, lint_yaml, rule_paths}}, types::SemanticVersion};

    #[test]
    fn reports_copy_loops_and_their_options() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
outputs:
  items:
    type: array
    copy:
      name: outer
      count: 2
resources:
- name: "[concat('a', copyIndex())]"
  type: Microsoft.DSC.Debug/Echo
  copy:
    name: loop
    count: 2
    mode: serial
    batchSize: 1
"#);
        assert_eq!(rule_paths(&result, "copy-loop-deprecated"), vec!["/outputs/items/copy", "/resources/0/copy"]);
        assert_eq!(rule_paths(&result, "copy-options-unsupported"), vec!["/resources/0/copy/batchSize", "/resources/0/copy/mode"]);
    }

    #[test]
    fn reports_security_context_in_metadata_and_conflicts() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
directives:
  securityContext: restricted
resources: []
"#);
        assert_eq!(rule_paths(&result, "security-context-in-metadata"), vec!["/metadata/Microsoft.DSC/securityContext"]);
        assert_eq!(rule_paths(&result, "security-context-conflict"), vec!["/directives/securityContext"]);
    }

    #[test]
    fn reports_unsatisfied_dsc_version() {
        let options = LintOptions {
            discovery: false,
            use_settings: false,
            dsc_version: Some(SemanticVersion::parse("3.1.0").unwrap()),
            ..Default::default()
        };
        let result = lint_with(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
directives:
  version: ">=4.0.0"
resources: []
"#, options);
        assert_eq!(rule_paths(&result, "dsc-version-unsatisfied"), vec!["/directives/version"]);
    }
}
