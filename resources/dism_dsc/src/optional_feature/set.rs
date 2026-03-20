// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde_json::{Map, Value};

use crate::optional_feature::dism::DismSessionHandle;
use crate::optional_feature::types::{FeatureState, OptionalFeatureList};

fn get_computer_name() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "localhost".to_string())
}

pub fn handle_set(input: &str) -> Result<String, String> {
    let feature_list: OptionalFeatureList = serde_json::from_str(input)
        .map_err(|e| t!("set.failedParseInput", err = e.to_string()).to_string())?;

    if feature_list.features.is_empty() {
        return Err(t!("set.featuresArrayEmpty").to_string());
    }

    let session = DismSessionHandle::open()?;
    let mut results = Vec::new();
    let mut reboot_required = false;

    for feature_input in &feature_list.features {
        let feature_name = feature_input
            .feature_name
            .as_ref()
            .ok_or_else(|| t!("set.featureNameRequired").to_string())?;

        let desired_state = feature_input
            .state
            .as_ref()
            .ok_or_else(|| t!("set.stateRequired").to_string())?;

        let needs_reboot = match desired_state {
            FeatureState::Installed => {
                session.enable_feature(feature_name)?
            }
            FeatureState::NotPresent => {
                session.disable_feature(feature_name, false)?
            }
            FeatureState::Removed => {
                session.disable_feature(feature_name, true)?
            }
            _ => {
                return Err(t!(
                    "set.unsupportedDesiredState",
                    state = desired_state.to_string()
                )
                .to_string());
            }
        };

        reboot_required = reboot_required || needs_reboot;

        let info = session.get_feature_info(feature_name)?;
        results.push(info);
    }

    let restart_required_meta = if reboot_required {
        let mut entry = Map::new();
        entry.insert("system".to_string(), Value::String(get_computer_name()));
        Some(vec![entry])
    } else {
        None
    };

    let output = OptionalFeatureList { restart_required_meta, features: results };
    serde_json::to_string(&output)
        .map_err(|e| t!("set.failedSerializeOutput", err = e.to_string()).to_string())
}
