// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;

use crate::dism::DismSessionHandle;
use crate::feature_on_demand::types::{CapabilityState, FeatureOnDemandInfo, FeatureOnDemandList};
use crate::util::{matches_wildcard, WildcardFilterable};

pub fn handle_export(input: &str) -> Result<String, String> {
    let filters: Vec<FeatureOnDemandInfo> = if input.trim().is_empty() {
        vec![FeatureOnDemandInfo::default()]
    } else {
        let list: FeatureOnDemandList = serde_json::from_str(input)
            .map_err(|e| t!("fod_export.failedParseInput", err = e.to_string()).to_string())?;
        list.capabilities
    };

    let session = DismSessionHandle::open()?;
    let all_basics = session.get_all_capability_basics()?;

    // Check if any filter requires full info (displayName, description, downloadSize, installSize)
    let needs_full_info = filters.iter().any(|f| {
        f.display_name.is_some() || f.description.is_some()
            || f.download_size.is_some() || f.install_size.is_some()
    });

    let mut results = Vec::new();

    let (filters_with_identity, filters_without_identity): (Vec<&FeatureOnDemandInfo>, Vec<&FeatureOnDemandInfo>) =
        if needs_full_info {
            filters.iter().partition(|f| f.identity.is_some())
        } else {
            (Vec::new(), Vec::new())
        };

    for (name, state_val) in &all_basics {
        let state = CapabilityState::from_dism(*state_val);

        if needs_full_info {
            let mut should_get_full = !filters_without_identity.is_empty();
            if !should_get_full {
                for f in &filters_with_identity {
                    if let Some(ref filter_identity) = f.identity {
                        if matches_wildcard(name, filter_identity) {
                            should_get_full = true;
                            break;
                        }
                    }
                }
            }
            if !should_get_full {
                continue;
            }

            let info = match session.get_capability_info(name) {
                Ok(raw) if !raw.unknown => FeatureOnDemandInfo {
                    identity: Some(raw.name),
                    exist: None,
                    state: CapabilityState::from_dism(raw.state),
                    display_name: Some(raw.display_name),
                    description: Some(raw.description),
                    download_size: Some(raw.download_size),
                    install_size: Some(raw.install_size),
                },
                _ => FeatureOnDemandInfo {
                    identity: Some(name.clone()),
                    exist: None,
                    state,
                    display_name: None,
                    description: None,
                    download_size: None,
                    install_size: None,
                },
            };

            if info.matches_any_filter(&filters) {
                results.push(info);
            }
        } else {
            // Fast path: only need identity and state for filtering, skip expensive
            // per-capability DismGetCapabilityInfo calls to match dism /online /get-capabilities speed.
            let basic = FeatureOnDemandInfo {
                identity: Some(name.clone()),
                state: state.clone(),
                ..FeatureOnDemandInfo::default()
            };

            if basic.matches_any_filter(&filters) {
                results.push(basic);
            }
        }
    }

    let output = FeatureOnDemandList { restart_required_meta: None, capabilities: results };
    serde_json::to_string(&output)
        .map_err(|e| t!("fod_export.failedSerializeOutput", err = e.to_string()).to_string())
}
