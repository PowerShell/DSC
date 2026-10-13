// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde_json::{Map, Value};

use crate::dism::DismSessionHandle;
use crate::feature_on_demand::types::{CapabilityState, FeatureOnDemandInfo, FeatureOnDemandList};
use crate::util::get_computer_name;

pub fn handle_set(input: &str) -> Result<String, String> {
    let capability_list: FeatureOnDemandList = serde_json::from_str(input)
        .map_err(|e| t!("fod_set.failedParseInput", err = e.to_string()).to_string())?;

    if capability_list.capabilities.is_empty() {
        return Err(t!("fod_set.capabilitiesArrayEmpty").to_string());
    }

    let session = DismSessionHandle::open()?;
    let mut results = Vec::new();
    let mut reboot_required = false;

    for cap_input in &capability_list.capabilities {
        let identity = cap_input
            .identity
            .as_ref()
            .ok_or_else(|| t!("fod_set.identityRequired").to_string())?;

        let desired_state = cap_input
            .state
            .as_ref()
            .ok_or_else(|| t!("fod_set.stateRequired").to_string())?;

        let current = session.get_capability_info(identity)?;

        if current.unknown {
            return Err(t!("fod_set.capabilityNotFound", identity = identity).to_string());
        }

        let current_state = CapabilityState::from_dism(current.state);

        let needs_reboot = match desired_state {
            CapabilityState::Installed => {
                match current_state {
                    Some(CapabilityState::Installed) => false,
                    _ => session.add_capability(identity)?,
                }
            }
            CapabilityState::NotPresent => {
                match current_state {
                    Some(CapabilityState::NotPresent) | Some(CapabilityState::Removed) => false,
                    _ => session.remove_capability(identity)?,
                }
            }
            _ => {
                return Err(t!(
                    "fod_set.unsupportedDesiredState",
                    state = desired_state.to_string()
                )
                .to_string());
            }
        };

        reboot_required = reboot_required || needs_reboot;

        let raw = session.get_capability_info(identity)?;
        let info = FeatureOnDemandInfo {
            identity: Some(raw.name),
            state: CapabilityState::from_dism(raw.state),
            display_name: Some(raw.display_name),
            description: Some(raw.description),
            download_size: Some(raw.download_size),
            install_size: Some(raw.install_size),
            ..FeatureOnDemandInfo::default()
        };
        results.push(info);
    }

    let restart_required_meta = if reboot_required {
        let mut entry = Map::new();
        entry.insert("system".to_string(), Value::String(get_computer_name()));
        Some(vec![entry])
    } else {
        None
    };

    let output = FeatureOnDemandList { restart_required_meta, capabilities: results };
    serde_json::to_string(&output)
        .map_err(|e| t!("fod_set.failedSerializeOutput", err = e.to_string()).to_string())
}
