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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{RuleAction, RuleDirection};

    fn firewall_rule() -> FirewallRule {
        FirewallRule {
            name: Some("Allow HTTPS".to_string()),
            description: Some("Allows inbound HTTPS".to_string()),
            application_name: Some("server.exe".to_string()),
            service_name: Some("W3SVC".to_string()),
            protocol: Some(6),
            local_ports: Some("443".to_string()),
            remote_ports: Some("*".to_string()),
            local_addresses: Some("Any".to_string()),
            remote_addresses: Some("Any".to_string()),
            direction: Some(RuleDirection::Inbound),
            action: Some(RuleAction::Allow),
            enabled: Some(true),
            profiles: Some(vec!["Private".to_string(), "Domain".to_string()]),
            grouping: Some("Web Server".to_string()),
            interface_types: Some(vec!["Wireless".to_string(), "Lan".to_string()]),
            edge_traversal: Some(false),
            ..Default::default()
        }
    }

    #[test]
    fn optional_matchers_handle_unset_missing_matching_and_mismatching_values() {
        let actual_string = Some("Value".to_string());
        assert!(matches_optional_string(&actual_string, &None));
        assert!(matches_optional_string(
            &actual_string,
            &Some("value".to_string())
        ));
        assert!(!matches_optional_string(
            &actual_string,
            &Some("other".to_string())
        ));
        assert!(!matches_optional_string(&None, &Some("value".to_string())));

        assert!(matches_optional_exact(&Some(1), &None));
        assert!(matches_optional_exact(&Some(1), &Some(1)));
        assert!(!matches_optional_exact(&Some(1), &Some(2)));
        assert!(!matches_optional_exact(&None, &Some(1)));
    }

    #[test]
    fn vector_matcher_is_case_insensitive_and_order_independent() {
        let actual = Some(vec!["Domain".to_string(), "Private".to_string()]);
        let reordered = Some(vec!["private".to_string(), "DOMAIN".to_string()]);

        assert!(matches_optional_vec(&actual, &None));
        assert!(matches_optional_vec(&actual, &reordered));
        assert!(!matches_optional_vec(
            &actual,
            &Some(vec!["Public".to_string()])
        ));
        assert!(!matches_optional_vec(&None, &reordered));
    }

    #[test]
    fn rule_filter_matches_every_supported_field() {
        let filter = FirewallRule {
            name: Some("allow https".to_string()),
            description: Some("allows inbound https".to_string()),
            application_name: Some("SERVER.EXE".to_string()),
            service_name: Some("w3svc".to_string()),
            protocol: Some(6),
            local_ports: Some("443".to_string()),
            remote_ports: Some("*".to_string()),
            local_addresses: Some("any".to_string()),
            remote_addresses: Some("ANY".to_string()),
            direction: Some(RuleDirection::Inbound),
            action: Some(RuleAction::Allow),
            enabled: Some(true),
            profiles: Some(vec!["domain".to_string(), "private".to_string()]),
            grouping: Some("web server".to_string()),
            interface_types: Some(vec!["lan".to_string(), "wireless".to_string()]),
            edge_traversal: Some(false),
            ..Default::default()
        };

        assert!(rule_matches_filter(&firewall_rule(), &filter));
    }

    #[test]
    fn rule_filter_rejects_wildcards_and_mismatched_values() {
        let rule = firewall_rule();

        assert!(!rule_matches_filter(
            &rule,
            &FirewallRule {
                name: Some("Allow*".to_string()),
                ..Default::default()
            }
        ));
        assert!(!rule_matches_filter(
            &rule,
            &FirewallRule {
                protocol: Some(17),
                ..Default::default()
            }
        ));
        assert!(!rule_matches_filter(
            &rule,
            &FirewallRule {
                profiles: Some(vec!["Public".to_string()]),
                ..Default::default()
            }
        ));
    }

    #[test]
    fn any_filter_uses_or_semantics() {
        let filters = vec![
            FirewallRule {
                name: Some("missing".to_string()),
                ..Default::default()
            },
            FirewallRule {
                protocol: Some(6),
                ..Default::default()
            },
        ];

        assert!(matches_any_filter(&firewall_rule(), &filters));
        assert!(!matches_any_filter(&firewall_rule(), &[]));
    }
}

