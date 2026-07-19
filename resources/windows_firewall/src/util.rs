// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types::FirewallRule;

pub fn matches_wildcard(text: &str, pattern: &str) -> bool {
    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    if !pattern_lower.contains('*') {
        return text_lower == pattern_lower;
    }

    let parts: Vec<&str> = pattern_lower.split('*').collect();
    if !parts[0].is_empty() && !text_lower.starts_with(parts[0]) {
        return false;
    }

    let mut position = parts[0].len();
    let suffix = *parts.last().unwrap_or(&"");
    let end = if suffix.is_empty() {
        text_lower.len()
    } else {
        if !text_lower.ends_with(suffix) {
            return false;
        }
        text_lower.len() - suffix.len()
    };

    for part in &parts[1..parts.len().saturating_sub(1)] {
        if part.is_empty() {
            continue;
        }
        match text_lower.get(position..end).and_then(|s| s.find(part)) {
            Some(index) => position += index + part.len(),
            None => return false,
        }
    }

    position <= end
}

fn matches_optional_wildcard(actual: &Option<String>, filter: &Option<String>) -> bool {
    match filter {
        Some(pattern) => match actual {
            Some(value) => matches_wildcard(value, pattern),
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
    matches_optional_wildcard(&rule.name, &filter.name)
        && matches_optional_wildcard(&rule.description, &filter.description)
        && matches_optional_wildcard(&rule.application_name, &filter.application_name)
        && matches_optional_wildcard(&rule.service_name, &filter.service_name)
        && matches_optional_exact(&rule.protocol, &filter.protocol)
        && matches_optional_wildcard(&rule.local_ports, &filter.local_ports)
        && matches_optional_wildcard(&rule.remote_ports, &filter.remote_ports)
        && matches_optional_wildcard(&rule.local_addresses, &filter.local_addresses)
        && matches_optional_wildcard(&rule.remote_addresses, &filter.remote_addresses)
        && matches_optional_exact(&rule.direction, &filter.direction)
        && matches_optional_exact(&rule.action, &filter.action)
        && matches_optional_exact(&rule.enabled, &filter.enabled)
        && matches_optional_vec(&rule.profiles, &filter.profiles)
        && matches_optional_wildcard(&rule.grouping, &filter.grouping)
        && matches_optional_vec(&rule.interface_types, &filter.interface_types)
        && matches_optional_exact(&rule.edge_traversal, &filter.edge_traversal)
}

pub fn matches_any_filter(rule: &FirewallRule, filters: &[FirewallRule]) -> bool {
    filters.iter().any(|filter| rule_matches_filter(rule, filter))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_matching_is_case_insensitive() {
        assert!(matches_wildcard("Firewall-Rule", "firewall-*"));
        assert!(matches_wildcard("AllowTCP", "*tcp"));
        assert!(!matches_wildcard("AllowUDP", "*tcp"));
    }
}
