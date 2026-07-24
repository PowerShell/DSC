// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;

use crate::dism::DismSessionHandle;
use crate::optional_feature::types::{FeatureState, OptionalFeatureInfo, OptionalFeatureList};

pub fn handle_export(_input: &str) -> Result<String, String> {
    let session = DismSessionHandle::open()?;
    let all_basics = session.get_all_feature_basics()?;

    let features = all_basics
        .iter()
        .map(|(name, state_val)| {
            // Return full info so engine-side filtering can match on properties like
            // displayName; fall back to the basic info if the feature can't be queried.
            session.get_feature_info(name).unwrap_or_else(|_| OptionalFeatureInfo {
                feature_name: Some(name.clone()),
                state: FeatureState::from_dism(*state_val),
                ..OptionalFeatureInfo::default()
            })
        })
        .collect();

    let output = OptionalFeatureList { restart_required_meta: None, features };
    serde_json::to_string(&output)
        .map_err(|e| t!("export.failedSerializeOutput", err = e.to_string()).to_string())
}
