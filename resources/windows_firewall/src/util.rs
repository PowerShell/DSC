// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types::FirewallRule;

fn matches_optional_string(actual: &Option<String>, filter: &Option<String>) -> bool {
    match filter {
        Some(pattern) => match actual {
            Some(value) => value.eq_ignore_ascii_case(pattern),
            None => false,
        },
        None => true,
    }
}

fn matches_optional_exact<T: PartialEq>(actual: &Option<T>, filter: &Option<T>) -> bool {
    match filter {
        Some(expected) => match actual {
            Some(value) => value == expected,
            None => false,
        },
        None => true,
    }
}

fn normalize_string_vec(values: &[String]) -> Vec<String> {
    let mut normalized: Vec<String> = values.iter().map(|value| value.to_lowercase()).collect();
    normalized.sort_unstable();
    normalized
}

fn matches_optional_vec(actual: &Option<Vec<String>>, filter: &Option<Vec<String>>) -> bool {
    match filter {
        Some(expected) => match actual {
            Some(value) => normalize_string_vec(value) == normalize_string_vec(expected),
            None => false,
        },
        None => true,
    }
}

pub fn rule_matches_filter(rule: &FirewallRule, filter: &FirewallRule) -> bool {
    matches_optional_string(&rule.name, &filter.name)
        && matches_optional_string(&rule.description, &filter.description)
        && matches_optional_string(&rule.application_name, &filter.application_name)
        && matches_optional_string(&rule.service_name, &filter.service_name)
        && matches_optional_exact(&rule.protocol, &filter.protocol)
        && matches_optional_string(&rule.local_ports, &filter.local_ports)
        && matches_optional_string(&rule.remote_ports, &filter.remote_ports)
        && matches_optional_string(&rule.local_addresses, &filter.local_addresses)
        && matches_optional_string(&rule.remote_addresses, &filter.remote_addresses)
        && matches_optional_exact(&rule.direction, &filter.direction)
        && matches_optional_exact(&rule.action, &filter.action)
        && matches_optional_exact(&rule.enabled, &filter.enabled)
        && matches_optional_vec(&rule.profiles, &filter.profiles)
        && matches_optional_string(&rule.grouping, &filter.grouping)
        && matches_optional_vec(&rule.interface_types, &filter.interface_types)
        && matches_optional_exact(&rule.edge_traversal, &filter.edge_traversal)
}

pub fn matches_any_filter(rule: &FirewallRule, filters: &[FirewallRule]) -> bool {
    filters.iter().any(|filter| rule_matches_filter(rule, filter))
}

