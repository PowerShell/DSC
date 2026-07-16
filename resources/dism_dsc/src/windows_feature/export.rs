// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;

use crate::dism::DismSessionHandle;
use crate::windows_feature::types::{FeatureState, WindowsFeatureInfo, WindowsFeatureList};

pub fn handle_export(_input: &str) -> Result<String, String> {
    let session = DismSessionHandle::open()?;
    let all_basics = session.get_all_feature_basics()?;

    let features = all_basics
        .iter()
        .map(|(name, state_val)| WindowsFeatureInfo {
            feature_name: Some(name.clone()),
            state: FeatureState::from_dism(*state_val),
            ..WindowsFeatureInfo::default()
        })
        .collect();

    let output = WindowsFeatureList {
        restart_required_meta: None,
        features,
    };
    serde_json::to_string(&output)
        .map_err(|e| t!("export.failedSerializeOutput", err = e.to_string()).to_string())
}
