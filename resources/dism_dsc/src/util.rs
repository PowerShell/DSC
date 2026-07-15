// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use std::fmt;

/// DISM package/feature state values shared by both Optional Features and Features on Demand.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DismState {
    NotPresent,
    UninstallPending,
    Staged,
    Removed,
    Installed,
    InstallPending,
    Superseded,
    PartiallyInstalled,
}

impl fmt::Display for DismState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DismState::NotPresent => write!(f, "NotPresent"),
            DismState::UninstallPending => write!(f, "UninstallPending"),
            DismState::Staged => write!(f, "Staged"),
            DismState::Removed => write!(f, "Removed"),
            DismState::Installed => write!(f, "Installed"),
            DismState::InstallPending => write!(f, "InstallPending"),
            DismState::Superseded => write!(f, "Superseded"),
            DismState::PartiallyInstalled => write!(f, "PartiallyInstalled"),
        }
    }
}

impl DismState {
    pub fn from_dism(state: i32) -> Option<Self> {
        match state {
            0 => Some(DismState::NotPresent),
            1 => Some(DismState::UninstallPending),
            2 => Some(DismState::Staged),
            3 => Some(DismState::Removed),
            4 => Some(DismState::Installed),
            5 => Some(DismState::InstallPending),
            6 => Some(DismState::Superseded),
            7 => Some(DismState::PartiallyInstalled),
            _ => None,
        }
    }
}

/// Check that an optional string field matches a case-insensitive exact filter value.
/// Returns true if the filter has no value (no constraint).
pub fn matches_optional_string(info_value: &Option<String>, filter_value: &Option<String>) -> bool {
    match filter_value {
        Some(pattern) => match info_value {
            Some(value) => value.eq_ignore_ascii_case(pattern),
            None => false,
        },
        None => true,
    }
}

/// Check whether an enumerated DISM name matches a populated filter identity.
pub fn matches_filter_name(name: &str, filter_name: Option<&str>) -> bool {
    filter_name.is_some_and(|filter_name| name.eq_ignore_ascii_case(filter_name))
}

/// Check that an optional field matches an exact filter value.
/// Returns true if the filter has no value (no constraint).
pub fn matches_optional_exact<T: PartialEq>(info_value: &Option<T>, filter_value: &Option<T>) -> bool {
    match filter_value {
        Some(expected) => match info_value {
            Some(actual) => actual == expected,
            None => false,
        },
        None => true,
    }
}

/// Trait for types that support export filtering.
pub trait Filterable {
    /// Returns true if this instance matches the given filter (AND logic within a single filter).
    fn matches_filter(&self, filter: &Self) -> bool;

    /// Returns true if this instance matches any of the given filters (OR logic between filters).
    fn matches_any_filter(&self, filters: &[Self]) -> bool
    where
        Self: Sized,
    {
        filters.iter().any(|filter| self.matches_filter(filter))
    }
}

/// Returns the computer name from the COMPUTERNAME environment variable, or "localhost" as fallback.
pub fn get_computer_name() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "localhost".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_string_matching_is_case_insensitive_and_exact() {
        let value = Some("Hello".to_string());

        assert!(matches_optional_string(&value, &Some("hello".to_string())));
        assert!(!matches_optional_string(&value, &Some("Hello*".to_string())));
        assert!(!matches_optional_string(&value, &Some("World".to_string())));
        assert!(matches_optional_string(&value, &None));
        assert!(!matches_optional_string(&None, &Some("Hello".to_string())));
    }

    #[test]
    fn filter_name_matching_requires_a_case_insensitive_exact_value() {
        assert!(matches_filter_name("Web-Server", Some("web-server")));
        assert!(!matches_filter_name("Web-Server", Some("Web-*")));
        assert!(!matches_filter_name("Web-Server", Some("TelnetClient")));
        assert!(!matches_filter_name("Web-Server", None));
    }

    #[test]
    fn optional_exact_and_any_filter_cover_all_outcomes() {
        assert!(matches_optional_exact(&Some(1), &None));
        assert!(matches_optional_exact(&Some(1), &Some(1)));
        assert!(!matches_optional_exact(&Some(1), &Some(2)));
        assert!(!matches_optional_exact(&None, &Some(1)));

        struct Number(i32);

        impl Filterable for Number {
            fn matches_filter(&self, filter: &Self) -> bool {
                self.0 == filter.0
            }
        }

        assert!(Number(2).matches_any_filter(&[Number(1), Number(2)]));
        assert!(!Number(2).matches_any_filter(&[Number(1), Number(3)]));
        assert!(!Number(2).matches_any_filter(&[]));
    }

    #[test]
    fn test_dism_state_from_dism() {
        assert_eq!(DismState::from_dism(0), Some(DismState::NotPresent));
        assert_eq!(DismState::from_dism(4), Some(DismState::Installed));
        assert_eq!(DismState::from_dism(7), Some(DismState::PartiallyInstalled));
        assert_eq!(DismState::from_dism(8), None);
        assert_eq!(DismState::from_dism(-1), None);
    }
}
