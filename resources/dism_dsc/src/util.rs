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

/// Match a string against a pattern that supports `*` wildcards (case-insensitive).
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

    let mut pos = parts[0].len();

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
        match text_lower.get(pos..end).and_then(|s| s.find(part)) {
            Some(idx) => pos += idx + part.len(),
            None => return false,
        }
    }

    pos <= end
}

/// Check that an optional string field matches a wildcard filter pattern.
/// Returns true if the filter has no value (no constraint).
pub fn matches_optional_wildcard(info_value: &Option<String>, filter_value: &Option<String>) -> bool {
    match filter_value {
        Some(pattern) => match info_value {
            Some(value) => matches_wildcard(value, pattern),
            None => false,
        },
        None => true,
    }
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

/// Trait for types that support wildcard-based filter matching in export operations.
pub trait WildcardFilterable {
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
    fn test_exact_match() {
        assert!(matches_wildcard("Hello", "Hello"));
        assert!(matches_wildcard("Hello", "hello"));
        assert!(!matches_wildcard("Hello", "World"));
    }

    #[test]
    fn test_star_only() {
        assert!(matches_wildcard("anything", "*"));
        assert!(matches_wildcard("", "*"));
    }

    #[test]
    fn test_prefix_wildcard() {
        assert!(matches_wildcard("HelloWorld", "Hello*"));
        assert!(matches_wildcard("Hello", "Hello*"));
        assert!(!matches_wildcard("World", "Hello*"));
    }

    #[test]
    fn test_suffix_wildcard() {
        assert!(matches_wildcard("HelloWorld", "*World"));
        assert!(matches_wildcard("World", "*World"));
        assert!(!matches_wildcard("Hello", "*World"));
    }

    #[test]
    fn test_middle_wildcard() {
        assert!(matches_wildcard("HelloWorld", "Hello*World"));
        assert!(matches_wildcard("HelloBeautifulWorld", "Hello*World"));
        assert!(!matches_wildcard("HelloBeautiful", "Hello*World"));
    }

    #[test]
    fn test_multiple_wildcards() {
        assert!(matches_wildcard("abcdef", "*b*d*"));
        assert!(matches_wildcard("abcdef", "a*c*f"));
        assert!(!matches_wildcard("abcdef", "a*z*f"));
    }

    #[test]
    fn test_double_star() {
        assert!(matches_wildcard("abc", "**"));
        assert!(matches_wildcard("abc", "a**c"));
        assert!(matches_wildcard("", "**"));
    }

    #[test]
    fn test_empty_pattern() {
        assert!(matches_wildcard("", ""));
        assert!(!matches_wildcard("abc", ""));
    }

    #[test]
    fn test_case_insensitive() {
        assert!(matches_wildcard("HELLO", "hello"));
        assert!(matches_wildcard("HelloWorld", "hello*world"));
        assert!(matches_wildcard("Microsoft.Windows.Feature", "*windows*"));
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
