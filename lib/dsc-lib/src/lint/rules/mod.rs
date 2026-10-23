// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The lint rules, grouped by the area of the document they inspect.

use super::rule::Rule;

pub mod dependency;
pub mod deprecation;
pub mod document;
pub mod environment;
pub mod expression;
pub mod parameter;
pub mod resource;

/// Every rule, grouped by category.
#[must_use]
pub fn all_rules() -> Vec<Rule> {
    let mut rules = Vec::new();
    rules.extend(document::rules());
    rules.extend(expression::rules());
    rules.extend(parameter::rules());
    rules.extend(dependency::rules());
    rules.extend(deprecation::rules());
    rules.extend(environment::rules());
    rules.extend(resource::rules());
    rules
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn rule_ids_are_unique_and_kebab_case() {
        let mut seen = HashSet::new();
        for rule in all_rules() {
            assert!(seen.insert(rule.id), "duplicate rule id {}", rule.id);
            assert!(
                rule.id.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "rule id {} must be kebab-case",
                rule.id
            );
            assert!(!(rule.description)().is_empty(), "rule {} has no description", rule.id);
        }
    }
}
