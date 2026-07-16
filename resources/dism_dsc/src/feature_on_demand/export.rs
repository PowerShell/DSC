// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;

use crate::dism::DismSessionHandle;
use crate::feature_on_demand::types::{CapabilityState, FeatureOnDemandInfo, FeatureOnDemandList};

pub fn handle_export(_input: &str) -> Result<String, String> {
    let session = DismSessionHandle::open()?;
    let all_basics = session.get_all_capability_basics()?;

    let capabilities = all_basics
        .iter()
        .map(|(name, state_val)| FeatureOnDemandInfo {
            identity: Some(name.clone()),
            state: CapabilityState::from_dism(*state_val),
            ..FeatureOnDemandInfo::default()
        })
        .collect();

    let output = FeatureOnDemandList { restart_required_meta: None, capabilities };
    serde_json::to_string(&output)
        .map_err(|e| t!("fod_export.failedSerializeOutput", err = e.to_string()).to_string())
}
