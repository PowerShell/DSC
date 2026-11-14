// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Rules about the environment a configuration document targets.

use std::sync::LazyLock;

use regex::Regex;
use rust_i18n::t;

use crate::lint::{
    context::LintContext,
    diagnostic::LintCategory,
    options::RuleLevel,
    pointer::{for_each_string, JsonPointer},
    rule::{DiagnosticSink, Rule, RuleTier},
};

/// Absolute paths rooted in an operating system drive or system directory.
static SYSTEM_PATH_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:[A-Za-z]:[\\/]|/(?:etc|usr|var|opt|bin|sbin|lib|home)/)")
        .expect("the system path pattern is a valid regex")
});

/// The rules in this module.
#[must_use]
pub fn rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "system-root-unused",
            category: LintCategory::Environment,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.environment.systemRootUnusedDescription").to_string(),
            check: system_root_unused,
        },
        Rule {
            id: "hardcoded-system-path",
            category: LintCategory::Environment,
            tier: RuleTier::Static,
            default_level: RuleLevel::Off,
            description: || t!("lint.rules.environment.hardcodedSystemPathDescription").to_string(),
            check: hardcoded_system_path,
        },
    ]
}

/// Report a document that doesn't call `systemRoot()` although a system root was specified.
fn system_root_unused(context: &LintContext, sink: &mut DiagnosticSink) {
    let Some(system_root) = &context.options.system_root else {
        return;
    };
    if context.expressions.calls_named("systemRoot").is_empty() {
        sink.report(
            &JsonPointer::root(),
            t!("lint.rules.environment.systemRootUnused", path = system_root.display()).to_string(),
        );
    }
}

/// Report literal absolute operating system paths in resource properties.
fn hardcoded_system_path(context: &LintContext, sink: &mut DiagnosticSink) {
    for (index, site) in context.resources.iter().enumerate() {
        let Some(properties) = context.raw_resource(index).and_then(|resource| resource.get("properties")) else {
            continue;
        };
        for_each_string(properties, &site.pointer.push("properties"), &mut |pointer, text| {
            if !text.starts_with('[') && SYSTEM_PATH_PATTERN.is_match(text) {
                sink.report(pointer, t!("lint.rules.environment.hardcodedSystemPath", value = text).to_string());
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::lint::{options::{LintOptions, RuleLevel, RuleLevelOverride}, test_util::{lint_with, rule_paths}};

    fn options_with_root() -> LintOptions {
        LintOptions {
            discovery: false,
            use_settings: false,
            system_root: Some(PathBuf::from("/mnt/image")),
            rule_levels: vec![RuleLevelOverride { id: "hardcoded-system-path".to_string(), level: RuleLevel::Info }],
            ..Default::default()
        }
    }

    #[test]
    fn reports_unused_system_root_and_hardcoded_paths() {
        let result = lint_with(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      windows: 'C:\Windows\System32'
      unix: /etc/ssh/sshd_config
      relative: config/sshd_config
      escaped: "[[C:\\not-an-expression]"
"#, options_with_root());
        assert_eq!(rule_paths(&result, "system-root-unused"), vec![""]);
        assert_eq!(
            rule_paths(&result, "hardcoded-system-path"),
            vec!["/resources/0/properties/output/unix", "/resources/0/properties/output/windows"]
        );
    }

    #[test]
    fn system_root_call_satisfies_the_rule() {
        let result = lint_with(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[concat(systemRoot(), 'etc')]"
"#, options_with_root());
        assert!(rule_paths(&result, "system-root-unused").is_empty());
    }
}
