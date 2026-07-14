// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;

use crate::dism::DismSessionHandle;
use crate::util::Filterable;
use crate::windows_feature::types::{FeatureState, WindowsFeatureInfo, WindowsFeatureList};

pub fn handle_export(input: &str) -> Result<String, String> {
    let filters: Vec<WindowsFeatureInfo> = if input.trim().is_empty() {
        vec![WindowsFeatureInfo::default()]
    } else {
        let list: WindowsFeatureList = serde_json::from_str(input)
            .map_err(|e| t!("export.failedParseInput", err = e.to_string()).to_string())?;
        if list.features.is_empty() {
            vec![WindowsFeatureInfo::default()]
        } else {
            list.features
        }
    };

    let session = DismSessionHandle::open()?;
    let all_basics = session.get_all_feature_basics()?;

    let needs_full_info = filters
        .iter()
        .any(|filter| filter.display_name.is_some() || filter.description.is_some());

    let mut results = Vec::new();

    let (filters_with_name, filters_without_name): (
        Vec<&WindowsFeatureInfo>,
        Vec<&WindowsFeatureInfo>,
    ) = if needs_full_info {
        filters
            .iter()
            .partition(|filter| filter.feature_name.is_some())
    } else {
        (Vec::new(), Vec::new())
    };

    for (name, state_val) in &all_basics {
        let state = FeatureState::from_dism(*state_val);

        if needs_full_info {
            let mut should_get_full = !filters_without_name.is_empty();
            if !should_get_full {
                for filter in &filters_with_name {
                    if let Some(ref filter_name) = filter.feature_name
                        && name.eq_ignore_ascii_case(filter_name)
                    {
                        should_get_full = true;
                        break;
                    }
                }
            }
            if !should_get_full {
                continue;
            }

            let info = match session.get_windows_feature_info(name) {
                Ok(info) => info,
                Err(_) => WindowsFeatureInfo {
                    feature_name: Some(name.clone()),
                    exist: None,
                    state,
                    display_name: None,
                    description: None,
                    restart_required: None,
                    enable_all: None,
                    source_paths: None,
                    limit_access: None,
                    ..Default::default()
                },
            };

            if info.matches_any_filter(&filters) {
                results.push(info);
            }
        } else {
            let basic = WindowsFeatureInfo {
                feature_name: Some(name.clone()),
                state: state.clone(),
                ..WindowsFeatureInfo::default()
            };

            if basic.matches_any_filter(&filters) {
                results.push(basic);
            }
        }
    }

    let output = WindowsFeatureList {
        restart_required_meta: None,
        features: results,
    };
    serde_json::to_string(&output)
        .map_err(|e| t!("export.failedSerializeOutput", err = e.to_string()).to_string())
}
