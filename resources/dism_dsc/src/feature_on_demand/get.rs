// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;

use crate::dism::DismSessionHandle;
use crate::feature_on_demand::types::{CapabilityState, FeatureOnDemandInfo, FeatureOnDemandList};

pub fn handle_get(input: &str) -> Result<String, String> {
    let capability_list: FeatureOnDemandList = serde_json::from_str(input)
        .map_err(|e| t!("fod_get.failedParseInput", err = e.to_string()).to_string())?;

    if capability_list.capabilities.is_empty() {
        return Err(t!("fod_get.capabilitiesArrayEmpty").to_string());
    }

    let session = DismSessionHandle::open()?;
    let mut results = Vec::new();

    for cap_input in &capability_list.capabilities {
        let identity = cap_input
            .identity
            .as_ref()
            .ok_or_else(|| t!("fod_get.identityRequired").to_string())?;

        let raw = session.get_capability_info(identity)?;
        let info = if raw.unknown {
            FeatureOnDemandInfo {
                identity: Some(identity.clone()),
                exist: Some(false),
                ..FeatureOnDemandInfo::default()
            }
        } else {
            FeatureOnDemandInfo {
                identity: Some(raw.name),
                state: CapabilityState::from_dism(raw.state),
                display_name: Some(raw.display_name),
                description: Some(raw.description),
                download_size: Some(raw.download_size),
                install_size: Some(raw.install_size),
                ..FeatureOnDemandInfo::default()
            }
        };
        results.push(info);
    }

    let output = FeatureOnDemandList { restart_required_meta: None, capabilities: results };
    serde_json::to_string(&output)
        .map_err(|e| t!("fod_get.failedSerializeOutput", err = e.to_string()).to_string())
}
