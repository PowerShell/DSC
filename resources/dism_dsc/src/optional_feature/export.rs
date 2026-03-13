// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;

use crate::optional_feature::dism::DismSessionHandle;
use crate::optional_feature::types::{FeatureState, OptionalFeatureInfo, OptionalFeatureList};

pub fn handle_export(input: &str) -> Result<String, String> {
    let filters: Vec<OptionalFeatureInfo> = if input.trim().is_empty() {
        vec![OptionalFeatureInfo::default()]
    } else {
        let list: OptionalFeatureList = serde_json::from_str(input)
            .map_err(|e| t!("export.failedParseInput", err = e.to_string()).to_string())?;
        list.features
    };

    let session = DismSessionHandle::open()?;
    let all_basics = session.get_all_feature_basics()?;

    // Check if any filter requires full info (displayName or description)
    let needs_full_info = filters
        .iter()
        .any(|f| f.display_name.is_some() || f.description.is_some());

    let mut results = Vec::new();

    for (name, state_val) in &all_basics {
        let state = FeatureState::from_dism(*state_val);

        if needs_full_info {
            // Get full info first so we can filter on displayName/description
            let info = match session.get_feature_info(name) {
                Ok(info) => info,
                Err(_) => OptionalFeatureInfo {
                    feature_name: Some(name.clone()),
                    state,
                    display_name: None,
                    description: None,
                    restart_required: None,
                },
            };

            if matches_any_filter(&info, &filters) {
                results.push(info);
            }
        } else {
            // Fast path: only need name and state for filtering
            let basic = OptionalFeatureInfo {
                feature_name: Some(name.clone()),
                state: state.clone(),
                ..OptionalFeatureInfo::default()
            };

            if matches_any_filter(&basic, &filters) {
                match session.get_feature_info(name) {
                    Ok(info) => results.push(info),
                    Err(_) => {
                        results.push(OptionalFeatureInfo {
                            feature_name: Some(name.clone()),
                            state,
                            display_name: None,
                            description: None,
                            restart_required: None,
                        });
                    }
                }
            }
        }
    }

    let output = OptionalFeatureList { features: results };
    serde_json::to_string(&output)
        .map_err(|e| t!("export.failedSerializeOutput", err = e.to_string()).to_string())
}

/// Check if the feature matches any filter (OR between filters, AND within each).
fn matches_any_filter(info: &OptionalFeatureInfo, filters: &[OptionalFeatureInfo]) -> bool {
    filters.iter().any(|filter| matches_filter(info, filter))
}

/// Apply AND logic within a single filter: all specified criteria must match.
fn matches_filter(info: &OptionalFeatureInfo, filter: &OptionalFeatureInfo) -> bool {
    // If filter has featureName, check with wildcard support
    if let Some(filter_name) = &filter.feature_name {
        match &info.feature_name {
            Some(name) if matches_wildcard(name, filter_name) => {}
            _ => return false,
        }
    }

    // If filter has state, check exact match
    if let Some(filter_state) = &filter.state {
        match &info.state {
            Some(s) if s == filter_state => {}
            _ => return false,
        }
    }

    // If filter has displayName, check with wildcard support
    if let Some(filter_display) = &filter.display_name {
        match &info.display_name {
            Some(display) if matches_wildcard(display, filter_display) => {}
            _ => return false,
        }
    }

    // If filter has description, check with wildcard support
    if let Some(filter_desc) = &filter.description {
        match &info.description {
            Some(desc) if matches_wildcard(desc, filter_desc) => {}
            _ => return false,
        }
    }

    true
}

/// Match a string against a pattern that supports `*` wildcards (case-insensitive).
fn matches_wildcard(text: &str, pattern: &str) -> bool {
    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    if !pattern_lower.contains('*') {
        return text_lower == pattern_lower;
    }

    let parts: Vec<&str> = pattern_lower.split('*').collect();

    // Check prefix (part before first *)
    if !parts[0].is_empty() && !text_lower.starts_with(parts[0]) {
        return false;
    }

    // Track position through the text, starting after the prefix
    let mut pos = parts[0].len();

    // The suffix is the last part; we need to reserve space for it
    let suffix = *parts.last().unwrap_or(&"");
    let end = if suffix.is_empty() {
        text_lower.len()
    } else {
        // Verify suffix matches and compute the boundary
        if !text_lower.ends_with(suffix) {
            return false;
        }
        text_lower.len() - suffix.len()
    };

    // Check middle parts appear in order within [pos..end]
    for part in &parts[1..parts.len().saturating_sub(1)] {
        if part.is_empty() {
            continue;
        }
        match text_lower[pos..end].find(part) {
            Some(idx) => pos += idx + part.len(),
            None => return false,
        }
    }

    pos <= end
}
