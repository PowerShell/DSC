// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;

use crate::dism::DismSessionHandle;
use crate::optional_feature::types::{FeatureState, OptionalFeatureInfo, OptionalFeatureList};
use crate::util::{matches_wildcard, WildcardFilterable};

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

    // When full info is needed, pre-partition filters by whether they specify a feature_name.
    // This lets us skip get_feature_info() for features that cannot match any name-constrained filter.
    let (filters_with_name, filters_without_name): (Vec<&OptionalFeatureInfo>, Vec<&OptionalFeatureInfo>) =
        if needs_full_info {
            filters.iter().partition(|f| f.feature_name.is_some())
        } else {
            (Vec::new(), Vec::new())
        };
        
    for (name, state_val) in &all_basics {
        let state = FeatureState::from_dism(*state_val);

        if needs_full_info {
            // Decide whether this feature could possibly match any filter based on its name.
            // If any filter does not constrain feature_name, we must consider every feature,
            // since such filters may match on displayName/description alone.
            let mut should_get_full = !filters_without_name.is_empty();
            if !should_get_full {
                for f in &filters_with_name {
                    if let Some(ref filter_name) = f.feature_name {
                        if matches_wildcard(name, filter_name) {
                            should_get_full = true;
                            break;
                        }
                    }
                }
            }
            if !should_get_full {
                // This feature cannot satisfy any name-constrained filter, and there are
                // no filters without a feature_name, so skip the expensive get_feature_info().
                continue;
            }
            // Get full info so we can filter on displayName/description and other fields.
            let info = match session.get_feature_info(name) {
                Ok(info) => info,
                Err(_) => OptionalFeatureInfo {
                    feature_name: Some(name.clone()),
                    exist: None,
                    state,
                    display_name: None,
                    description: None,
                    restart_required: None,
                },
            };

            if info.matches_any_filter(&filters) {
                results.push(info);
            }
        } else {
            // Fast path: only need name and state for filtering, skip expensive
            // per-feature DismGetFeatureInfo calls to match dism /online /get-features speed.
            let basic = OptionalFeatureInfo {
                feature_name: Some(name.clone()),
                state: state.clone(),
                ..OptionalFeatureInfo::default()
            };

            if basic.matches_any_filter(&filters) {
                results.push(basic);
            }
        }
    }

    let output = OptionalFeatureList { restart_required_meta: None, features: results };
    serde_json::to_string(&output)
        .map_err(|e| t!("export.failedSerializeOutput", err = e.to_string()).to_string())
}
