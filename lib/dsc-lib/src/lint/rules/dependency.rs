// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Rules about the dependencies between resources.

use std::collections::{HashMap, HashSet};

use rust_i18n::t;
use serde_json::Value;

use crate::{
    lint::{
        context::LintContext,
        diagnostic::LintCategory,
        expression_index::ExpressionScope,
        options::RuleLevel,
        pointer::JsonPointer,
        rule::{DiagnosticSink, Rule, RuleTier},
    },
    parser::{expressions::Expression, functions::FunctionArg, ParsedStatement},
};

/// The rules in this module.
#[must_use]
pub fn rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "unresolved-dependency",
            category: LintCategory::Dependency,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.dependency.unresolvedDependencyDescription").to_string(),
            check: unresolved_dependency,
        },
        Rule {
            id: "circular-dependency",
            category: LintCategory::Dependency,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.dependency.circularDependencyDescription").to_string(),
            check: circular_dependency,
        },
        Rule {
            id: "reference-without-depends-on",
            category: LintCategory::Dependency,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.dependency.referenceWithoutDependsOnDescription").to_string(),
            check: reference_without_depends_on,
        },
        Rule {
            id: "duplicate-depends-on",
            category: LintCategory::Dependency,
            tier: RuleTier::Static,
            default_level: RuleLevel::Info,
            description: || t!("lint.rules.dependency.duplicateDependsOnDescription").to_string(),
            check: duplicate_depends_on,
        },
    ]
}

/// Identifies a resource by type and name, as `resourceId()` does.
#[derive(Clone, Debug)]
struct ResourceKey {
    /// The resource type as written; compared case-insensitively.
    type_name: String,
    /// The resource name; compared exactly.
    name: String,
}

impl ResourceKey {
    /// Whether this key identifies the same resource as another.
    ///
    /// # Arguments
    ///
    /// * `other` - The key to compare with.
    fn same_as(&self, other: &ResourceKey) -> bool {
        self.name == other.name && self.type_name.eq_ignore_ascii_case(&other.type_name)
    }

    /// The key as `type/name`, for messages.
    fn display(&self) -> String {
        format!("{} named '{}'", self.type_name, self.name)
    }
}

/// How a dependency key resolved against the sibling resources.
enum Resolution {
    /// The resource at the given index matches by name and type.
    Found(usize),
    /// A resource at the given index has the name but a different type.
    TypeMismatch(usize),
    /// No sibling has the name.
    NotFound,
}

/// The key of a `resourceId('type', 'name')` expression with literal arguments and no accessors.
///
/// # Arguments
///
/// * `expression` - The expression.
fn resource_id_key(expression: &Expression) -> Option<ResourceKey> {
    if !expression.accessors().is_empty() || expression.function().name() != "resourceId" {
        return None;
    }
    let args = expression.function().args()?;
    match args {
        [FunctionArg::Value(Value::String(type_name)), FunctionArg::Value(Value::String(name))] => Some(ResourceKey {
            type_name: type_name.clone(),
            name: name.clone(),
        }),
        _ => None,
    }
}

/// The literal `dependsOn` entries of a resource with their pointers.
///
/// Entries that aren't a literal `resourceId()` call are skipped; the schema already rejects
/// entries that aren't `resourceId()` expressions.
///
/// # Arguments
///
/// * `context` - The lint context.
/// * `index` - The index of the resource in the context's resource list.
fn literal_dependencies(context: &LintContext, index: usize) -> Vec<(JsonPointer, ResourceKey)> {
    let site = &context.resources[index];
    let Some(depends_on) = &site.resource.depends_on else {
        return Vec::new();
    };
    let base = site.pointer.push("dependsOn");
    (0..depends_on.len()).filter_map(|position| {
        let pointer = base.push_index(position);
        let parsed = context.expressions.site_at(&pointer)?.parsed.as_ref().ok()?;
        let ParsedStatement::Expression(expression) = parsed else {
            return None;
        };
        resource_id_key(expression).map(|key| (pointer, key))
    }).collect()
}

/// The resources a resource references through `reference(resourceId('type', 'name'))`, with
/// the pointer of each referencing expression.
///
/// # Arguments
///
/// * `context` - The lint context.
/// * `index` - The index of the resource in the context's resource list.
fn reference_targets(context: &LintContext, index: usize) -> Vec<(JsonPointer, ResourceKey)> {
    context.expressions.calls_named("reference").into_iter()
        .filter(|call| call.site.scope == ExpressionScope::Resource(index))
        .filter_map(|call| {
            let Some([FunctionArg::Expression(argument), ..]) = call.function.args() else {
                return None;
            };
            resource_id_key(argument).map(|key| (call.site.pointer.clone(), key))
        })
        .collect()
}

/// Resolve a dependency key against a group of sibling resources.
///
/// # Arguments
///
/// * `context` - The lint context.
/// * `group` - The indexes of the sibling resources.
/// * `key` - The dependency key.
fn resolve(context: &LintContext, group: &[usize], key: &ResourceKey) -> Resolution {
    let by_name: Vec<usize> = group.iter().copied().filter(|&index| context.resources[index].resource.name == key.name).collect();
    if let Some(&index) = by_name.iter().find(|&&index| context.resources[index].resource.resource_type.as_ref().eq_ignore_ascii_case(&key.type_name)) {
        return Resolution::Found(index);
    }
    match by_name.first() {
        Some(&index) => Resolution::TypeMismatch(index),
        None => Resolution::NotFound,
    }
}

/// Report `dependsOn` entries and `reference()` calls whose target isn't a sibling resource.
///
/// Resources in copy loops are skipped, and a missing target isn't reported when a sibling is
/// a copy loop, because copy loops produce names that are only known at runtime.
fn unresolved_dependency(context: &LintContext, sink: &mut DiagnosticSink) {
    for group in context.sibling_groups() {
        let has_copy = group.iter().any(|&index| context.resources[index].resource.copy.is_some());
        for &index in &group {
            let site = &context.resources[index];
            if site.resource.copy.is_some() {
                continue;
            }
            for (pointer, key) in literal_dependencies(context, index).into_iter().chain(reference_targets(context, index)) {
                match resolve(context, &group, &key) {
                    Resolution::Found(_) => {},
                    Resolution::TypeMismatch(actual) => sink.report(
                        &pointer,
                        t!(
                            "lint.rules.dependency.dependencyTypeMismatch",
                            name = site.resource.name,
                            dependency_type = key.type_name,
                            dependency_name = key.name,
                            actual_type = context.resources[actual].resource.resource_type
                        ).to_string(),
                    ),
                    Resolution::NotFound => {
                        if !has_copy {
                            sink.report(
                                &pointer,
                                t!(
                                    "lint.rules.dependency.unresolvedDependency",
                                    name = site.resource.name,
                                    dependency_type = key.type_name,
                                    dependency_name = key.name
                                ).to_string(),
                            );
                        }
                    },
                }
            }
        }
    }
}

/// Report resources whose `dependsOn` entries form a cycle.
fn circular_dependency(context: &LintContext, sink: &mut DiagnosticSink) {
    for group in context.sibling_groups() {
        let mut edges: HashMap<usize, Vec<usize>> = HashMap::new();
        for &index in &group {
            if context.resources[index].resource.copy.is_some() {
                continue;
            }
            let targets = literal_dependencies(context, index).into_iter().filter_map(|(_, key)| match resolve(context, &group, &key) {
                Resolution::Found(target) => Some(target),
                _ => None,
            }).collect();
            edges.insert(index, targets);
        }

        let mut reported: HashSet<Vec<usize>> = HashSet::new();
        for &start in &group {
            let Some(cycle) = find_cycle(start, &edges) else {
                continue;
            };
            let mut members = cycle.clone();
            members.sort_unstable();
            if !reported.insert(members) {
                continue;
            }
            let site = &context.resources[start];
            let path: Vec<&str> = cycle.iter().chain(std::iter::once(&start)).map(|&index| context.resources[index].resource.name.as_str()).collect();
            sink.report(
                &site.pointer.push("dependsOn"),
                t!("lint.rules.dependency.circularDependency", name = site.resource.name, cycle = path.join(" -> ")).to_string(),
            );
        }
    }
}

/// Find a dependency path that leads from a resource back to itself.
///
/// # Arguments
///
/// * `start` - The resource to start from.
/// * `edges` - The resolved dependencies of each resource.
///
/// # Returns
///
/// The resources on the cycle, starting with `start`, or `None` when there is no cycle.
fn find_cycle(start: usize, edges: &HashMap<usize, Vec<usize>>) -> Option<Vec<usize>> {
    let mut path = Vec::new();
    let mut visited = HashSet::new();
    if walk(start, start, edges, &mut path, &mut visited) { Some(path) } else { None }
}

/// Depth-first search for a path from `node` back to `start`.
///
/// # Arguments
///
/// * `node` - The current resource.
/// * `start` - The resource the search is looking for.
/// * `edges` - The resolved dependencies of each resource.
/// * `path` - The resources on the current path; holds the cycle when the search succeeds.
/// * `visited` - The resources already explored.
fn walk(node: usize, start: usize, edges: &HashMap<usize, Vec<usize>>, path: &mut Vec<usize>, visited: &mut HashSet<usize>) -> bool {
    path.push(node);
    visited.insert(node);
    for &next in edges.get(&node).map(Vec::as_slice).unwrap_or(&[]) {
        if next == start {
            return true;
        }
        if !visited.contains(&next) && walk(next, start, edges, path, visited) {
            return true;
        }
    }
    path.pop();
    false
}

/// Report `reference()` calls to resources that aren't listed in `dependsOn`.
fn reference_without_depends_on(context: &LintContext, sink: &mut DiagnosticSink) {
    for index in 0..context.resources.len() {
        let site = &context.resources[index];
        let dependencies = literal_dependencies(context, index);
        for (pointer, target) in reference_targets(context, index) {
            if !dependencies.iter().any(|(_, dependency)| dependency.same_as(&target)) {
                sink.report(
                    &pointer,
                    t!("lint.rules.dependency.referenceWithoutDependsOn", name = site.resource.name, target = target.display()).to_string(),
                );
            }
        }
    }
}

/// Report `dependsOn` entries that repeat an earlier entry.
fn duplicate_depends_on(context: &LintContext, sink: &mut DiagnosticSink) {
    for index in 0..context.resources.len() {
        let site = &context.resources[index];
        let mut seen: Vec<ResourceKey> = Vec::new();
        for (pointer, key) in literal_dependencies(context, index) {
            if seen.iter().any(|earlier| earlier.same_as(&key)) {
                sink.report(
                    &pointer,
                    t!("lint.rules.dependency.duplicateDependsOn", name = site.resource.name, target = key.display()).to_string(),
                );
            } else {
                seen.push(key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lint::test_util::{lint_yaml, rule_messages, rule_paths};

    #[test]
    fn reports_unresolved_and_mismatched_dependencies() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: first
  type: Microsoft.DSC.Debug/Echo
- name: second
  type: Microsoft.DSC.Debug/Echo
  dependsOn:
  - "[resourceId('microsoft.dsc.debug/echo', 'first')]"
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'missing')]"
  - "[resourceId('Microsoft/OSInfo', 'first')]"
  properties:
    output: "[reference(resourceId('Microsoft.DSC.Debug/Echo', 'nobody'))]"
"#);
        assert_eq!(
            rule_paths(&result, "unresolved-dependency"),
            vec!["/resources/1/dependsOn/1", "/resources/1/dependsOn/2", "/resources/1/properties/output"]
        );
        assert!(rule_messages(&result, "unresolved-dependency")[1].contains("Microsoft.DSC.Debug/Echo"));
    }

    #[test]
    fn copy_loops_suppress_unresolved_dependencies() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[concat('item', copyIndex())]"
  type: Microsoft.DSC.Debug/Echo
  copy:
    name: items
    count: 2
- name: consumer
  type: Microsoft.DSC.Debug/Echo
  dependsOn:
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'item0')]"
"#);
        assert!(rule_paths(&result, "unresolved-dependency").is_empty());
    }

    #[test]
    fn reports_cycles_once() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  dependsOn:
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'b')]"
- name: b
  type: Microsoft.DSC.Debug/Echo
  dependsOn:
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'a')]"
- name: self
  type: Microsoft.DSC.Debug/Echo
  dependsOn:
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'self')]"
- name: fine
  type: Microsoft.DSC.Debug/Echo
  dependsOn:
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'a')]"
"#);
        assert_eq!(rule_paths(&result, "circular-dependency"), vec!["/resources/0/dependsOn", "/resources/2/dependsOn"]);
        assert!(rule_messages(&result, "circular-dependency")[0].contains("a -> b -> a"));
    }

    #[test]
    fn reports_references_without_depends_on() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: source
  type: Microsoft.DSC.Debug/Echo
- name: listed
  type: Microsoft.DSC.Debug/Echo
  dependsOn:
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'source')]"
  properties:
    output: "[reference(resourceId('Microsoft.DSC.Debug/Echo', 'source')).output]"
- name: unlisted
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[reference(resourceId('Microsoft.DSC.Debug/Echo', 'source')).output]"
"#);
        assert_eq!(rule_paths(&result, "reference-without-depends-on"), vec!["/resources/2/properties/output"]);
    }

    #[test]
    fn reports_duplicate_depends_on_entries() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: source
  type: Microsoft.DSC.Debug/Echo
- name: twice
  type: Microsoft.DSC.Debug/Echo
  dependsOn:
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'source')]"
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'source')]"
"#);
        assert_eq!(rule_paths(&result, "duplicate-depends-on"), vec!["/resources/1/dependsOn/1"]);
    }
}
