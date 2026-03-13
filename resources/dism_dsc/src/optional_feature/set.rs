// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;

use crate::optional_feature::dism::DismSessionHandle;
use crate::optional_feature::types::{FeatureState, OptionalFeatureList};

pub fn handle_set(input: &str) -> Result<String, String> {
    let feature_list: OptionalFeatureList = serde_json::from_str(input)
        .map_err(|e| t!("set.failedParseInput", err = e.to_string()).to_string())?;

    if feature_list.features.is_empty() {
        return Err(t!("set.featuresArrayEmpty").to_string());
    }

    let session = DismSessionHandle::open()?;
    let mut results = Vec::new();

    for feature_input in &feature_list.features {
        let feature_name = feature_input
            .feature_name
            .as_ref()
            .ok_or_else(|| t!("set.featureNameRequired").to_string())?;

        let desired_state = feature_input
            .state
            .as_ref()
            .ok_or_else(|| t!("set.stateRequired").to_string())?;

        match desired_state {
            FeatureState::Installed => {
                session.enable_feature(feature_name)?;
            }
            FeatureState::NotPresent | FeatureState::Removed => {
                session.disable_feature(feature_name, true)?;
            }
            _ => {
                return Err(t!(
                    "set.unsupportedDesiredState",
                    state = format!("{desired_state:?}")
                )
                .to_string());
            }
        }

        let info = session.get_feature_info(feature_name)?;
        results.push(info);
    }

    let output = OptionalFeatureList { features: results };
    serde_json::to_string(&output)
        .map_err(|e| t!("set.failedSerializeOutput", err = e.to_string()).to_string())
}
