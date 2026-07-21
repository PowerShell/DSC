// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde_json::{Map, Value};
use tracing::debug;

/// Apply an export filter to a list of exported instances, retaining only matching instances.
///
/// When a filter property is an array of objects and the matching instance property is an
/// array, the instance is retained and that array property is rewritten to only the elements
/// matching any of the filter objects. This supports resources that export a single instance
/// holding a list of items (e.g. `{"features": [...]}`).
///
/// # Arguments
///
/// * `instances` - The exported instances to filter.
/// * `filters` - The filter objects from the `exportFilter` directive.
pub(super) fn apply_export_filter(instances: &mut Vec<Value>, filters: &[Map<String, Value>]) {
    if filters.is_empty() {
        // an empty filter list means no filtering is applied
        return;
    }

    let original_count = instances.len();
    instances.retain_mut(|instance| {
        if !instance_matches_filters(instance, filters) {
            return false;
        }
        let matching: Vec<&Map<String, Value>> = instance.as_object()
            .map(|instance_obj| filters.iter().filter(|filter| instance_matches_filter(instance_obj, filter)).collect())
            .unwrap_or_default();
        filter_array_elements(instance, &matching);
        true
    });
    debug!("{}", t!("configure.export_filter.filteredInstances", original = original_count, retained = instances.len()));
}

/// Rewrite the instance's array properties targeted by element filters, retaining only the
/// elements that match any of the element filter objects (logical OR).
fn filter_array_elements(instance: &mut Value, filters: &[&Map<String, Value>]) {
    let Some(instance_obj) = instance.as_object_mut() else {
        return;
    };

    for filter in filters {
        for (name, expected) in *filter {
            let Some(element_filters) = as_element_filters(expected) else {
                continue;
            };
            if let Some(elements) = instance_obj.get_mut(name).and_then(Value::as_array_mut) {
                elements.retain(|element| {
                    element.as_object().is_some_and(|element_obj| {
                        element_filters.iter().any(|element_filter| instance_matches_filter(element_obj, element_filter))
                    })
                });
            }
        }
    }
}

/// Returns the filter objects when the value is a non-empty array consisting solely of objects,
/// which is treated as a set of element filters for an array property.
fn as_element_filters(value: &Value) -> Option<Vec<&Map<String, Value>>> {
    let array = value.as_array()?;
    if array.is_empty() {
        return None;
    }
    array.iter().map(Value::as_object).collect()
}

/// Check if an instance matches any of the filter objects (logical OR).
#[must_use]
fn instance_matches_filters(instance: &Value, filters: &[Map<String, Value>]) -> bool {
    let Some(instance) = instance.as_object() else {
        // non-object instances can't be matched by property filters
        return false;
    };

    filters.iter().any(|filter| instance_matches_filter(instance, filter))
}

/// Check if an instance matches all properties of a single filter object (logical AND).
fn instance_matches_filter(instance: &Map<String, Value>, filter: &Map<String, Value>) -> bool {
    filter.iter().all(|(name, expected)| {
        instance.get(name).is_some_and(|actual| value_matches(actual, expected))
    })
}

/// Check if an actual value matches an expected filter value.
fn value_matches(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        // strings are compared case-insensitively with `*` wildcard support
        (Value::String(actual_str), Value::String(pattern)) => wildcard_match(pattern, actual_str),
        // nested objects match recursively as a partial match
        (Value::Object(actual_obj), Value::Object(expected_obj)) => instance_matches_filter(actual_obj, expected_obj),
        (Value::Array(actual_arr), Value::Array(expected_arr)) => {
            if as_element_filters(expected).is_some() {
                true
            } else {
                expected_arr.iter().all(|e| actual_arr.iter().any(|a| value_matches(a, e)))
            }
        },
        // everything else requires equality
        _ => actual == expected,
    }
}

/// Match `text` against `pattern` where `*` matches zero or more characters.
/// The comparison is case-insensitive.
fn wildcard_match(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.to_lowercase().chars().collect();
    let text: Vec<char> = text.to_lowercase().chars().collect();

    // iterative greedy matching with backtracking on the last `*`
    let (mut p, mut t) = (0usize, 0usize);
    let mut star: Option<usize> = None;
    let mut star_text = 0usize;

    while t < text.len() {
        if p < pattern.len() && pattern[p] == '*' {
            star = Some(p);
            star_text = t;
            p += 1;
        } else if p < pattern.len() && pattern[p] == text[t] {
            p += 1;
            t += 1;
        } else if let Some(star_pos) = star {
            // backtrack: let the last `*` consume one more character
            p = star_pos + 1;
            star_text += 1;
            t = star_text;
        } else {
            return false;
        }
    }

    // remaining pattern must be all `*`
    pattern[p..].iter().all(|c| *c == '*')
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn to_filters(value: Value) -> Vec<Map<String, Value>> {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn wildcard_match_exact() {
        assert!(wildcard_match("sshd", "sshd"));
        assert!(!wildcard_match("sshd", "sshd2"));
        assert!(!wildcard_match("sshd2", "sshd"));
    }

    #[test]
    fn wildcard_match_case_insensitive() {
        assert!(wildcard_match("SSHD", "sshd"));
        assert!(wildcard_match("*Ssh*", "OpenSSH Server"));
    }

    #[test]
    fn wildcard_match_star() {
        assert!(wildcard_match("*ssh*", "ssh"));
        assert!(wildcard_match("*ssh*", "openssh-server"));
        assert!(wildcard_match("ssh*", "sshd"));
        assert!(wildcard_match("*shd", "sshd"));
        assert!(wildcard_match("*", ""));
        assert!(wildcard_match("*", "anything"));
        assert!(wildcard_match("s*h*d", "sshd"));
        assert!(!wildcard_match("*ssh*", "no match"));
        assert!(!wildcard_match("ssh*", "openssh"));
    }

    #[test]
    fn empty_filter_list_matches_nothing_but_apply_is_noop() {
        let mut instances = vec![json!({"name": "one"}), json!({"name": "two"})];
        apply_export_filter(&mut instances, &[]);
        assert_eq!(instances.len(), 2);
    }

    #[test]
    fn filters_are_logical_or() {
        let filters = to_filters(json!([
            { "name": "*ssh*" },
            { "startType": "automatic" }
        ]));
        // matches first filter
        assert!(instance_matches_filters(&json!({"name": "sshd", "startType": "manual"}), &filters));
        // matches second filter
        assert!(instance_matches_filters(&json!({"name": "spooler", "startType": "automatic"}), &filters));
        // matches neither
        assert!(!instance_matches_filters(&json!({"name": "spooler", "startType": "manual"}), &filters));
    }

    #[test]
    fn properties_within_filter_are_logical_and() {
        let filters = to_filters(json!([
            { "name": "*ssh*", "startType": "automatic" }
        ]));
        assert!(instance_matches_filters(&json!({"name": "sshd", "startType": "automatic"}), &filters));
        assert!(!instance_matches_filters(&json!({"name": "sshd", "startType": "manual"}), &filters));
        assert!(!instance_matches_filters(&json!({"name": "spooler", "startType": "automatic"}), &filters));
    }

    #[test]
    fn missing_property_does_not_match() {
        let filters = to_filters(json!([{ "name": "*ssh*" }]));
        assert!(!instance_matches_filters(&json!({"startType": "automatic"}), &filters));
    }

    #[test]
    fn non_string_values_use_equality() {
        let filters = to_filters(json!([{ "count": 2, "enabled": true }]));
        assert!(instance_matches_filters(&json!({"count": 2, "enabled": true}), &filters));
        assert!(!instance_matches_filters(&json!({"count": 3, "enabled": true}), &filters));
        assert!(!instance_matches_filters(&json!({"count": 2, "enabled": false}), &filters));
        // a string pattern does not match a non-string value
        let filters = to_filters(json!([{ "count": "*" }]));
        assert!(!instance_matches_filters(&json!({"count": 2}), &filters));
    }

    #[test]
    fn nested_objects_match_recursively() {
        let filters = to_filters(json!([
            { "properties": { "name": "b*r" } }
        ]));
        assert!(instance_matches_filters(&json!({"properties": {"name": "bar", "other": 1}}), &filters));
        assert!(!instance_matches_filters(&json!({"properties": {"name": "baz"}}), &filters));
    }

    #[test]
    fn empty_filter_object_matches_everything() {
        let filters = to_filters(json!([{}]));
        assert!(instance_matches_filters(&json!({"name": "anything"}), &filters));
    }

    #[test]
    fn apply_export_filter_retains_matching() {
        let mut instances = vec![
            json!({"name": "sshd", "startType": "automatic"}),
            json!({"name": "spooler", "startType": "automatic"}),
            json!({"name": "ssh-agent", "startType": "manual"}),
        ];
        let filters = to_filters(json!([{ "name": "*ssh*" }]));
        apply_export_filter(&mut instances, &filters);
        assert_eq!(instances.len(), 2);
        assert_eq!(instances[0]["name"], "sshd");
        assert_eq!(instances[1]["name"], "ssh-agent");
    }

    #[test]
    fn non_object_instances_do_not_match() {
        let filters = to_filters(json!([{ "name": "*" }]));
        assert!(!instance_matches_filters(&json!("just a string"), &filters));
        assert!(!instance_matches_filters(&json!(42), &filters));
    }

    #[test]
    fn element_filters_rewrite_array_properties() {
        let mut instances = vec![json!({"features": [
            {"featureName": "TelnetClient", "state": "Installed"},
            {"featureName": "Printing-Foundation", "state": "Installed"},
            {"featureName": "SMB1", "state": "NotPresent"}
        ]})];
        let filters = to_filters(json!([{ "features": [{ "featureName": "printing-*" }] }]));
        apply_export_filter(&mut instances, &filters);
        assert_eq!(instances.len(), 1);
        let features = instances[0]["features"].as_array().unwrap();
        assert_eq!(features.len(), 1);
        assert_eq!(features[0]["featureName"], "Printing-Foundation");
    }

    #[test]
    fn element_filters_are_logical_or() {
        let mut instances = vec![json!({"features": [
            {"featureName": "One", "state": "Installed"},
            {"featureName": "Two", "state": "NotPresent"},
            {"featureName": "Three", "state": "NotPresent"}
        ]})];
        let filters = to_filters(json!([{ "features": [{ "featureName": "One" }, { "state": "notpresent" }] }]));
        apply_export_filter(&mut instances, &filters);
        let features = instances[0]["features"].as_array().unwrap();
        assert_eq!(features.len(), 3);
    }

    #[test]
    fn instance_retained_when_no_elements_match() {
        let mut instances = vec![json!({"rules": [{"name": "AllowTCP"}]})];
        let filters = to_filters(json!([{ "rules": [{ "name": "NoSuchRule" }] }]));
        apply_export_filter(&mut instances, &filters);
        assert_eq!(instances.len(), 1);
        assert!(instances[0]["rules"].as_array().unwrap().is_empty());
    }

    #[test]
    fn element_filters_require_array_property() {
        let filters = to_filters(json!([{ "features": [{ "featureName": "x" }] }]));
        assert!(!instance_matches_filters(&json!({"features": "not an array"}), &filters));
        assert!(!instance_matches_filters(&json!({"other": []}), &filters));
    }

    #[test]
    fn scalar_array_filter_is_contains_all() {
        let filters = to_filters(json!([{ "dependencies": ["rpcss"] }]));
        // case-insensitive element matching, superset allowed
        assert!(instance_matches_filters(&json!({"dependencies": ["RpcSs", "http"]}), &filters));
        assert!(!instance_matches_filters(&json!({"dependencies": ["http"]}), &filters));
        let filters = to_filters(json!([{ "dependencies": ["rpcss", "http"] }]));
        assert!(instance_matches_filters(&json!({"dependencies": ["http", "RpcSs", "other"]}), &filters));
        assert!(!instance_matches_filters(&json!({"dependencies": ["RpcSs"]}), &filters));
    }

    #[test]
    fn empty_array_filter_matches_any_array() {
        let filters = to_filters(json!([{ "dependencies": [] }]));
        assert!(instance_matches_filters(&json!({"dependencies": ["anything"]}), &filters));
        assert!(instance_matches_filters(&json!({"dependencies": []}), &filters));
        assert!(!instance_matches_filters(&json!({"dependencies": "not an array"}), &filters));
    }
}
