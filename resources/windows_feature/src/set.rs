// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde_json::{Map, Value};

use crate::dism::DismSessionHandle;
use crate::types::{FeatureState, Metadata, WindowsFeatureInfo, WindowsFeatureList};
use crate::util::get_computer_name;

pub fn handle_set(input: &WindowsFeatureList, what_if: bool) -> Result<WindowsFeatureList, String> {
    if input.features.is_empty() {
        return Err(t!("set.featuresArrayEmpty").to_string());
    }

    let session = DismSessionHandle::open()?;
    let mut results: Vec<WindowsFeatureInfo> = Vec::new();
    let mut reboot_required = false;

    for feature_input in &input.features {
        let feature_name = feature_input
            .feature_name
            .as_ref()
            .ok_or_else(|| t!("set.featureNameRequired").to_string())?;

        let desired_state = feature_input
            .state
            .as_ref()
            .ok_or_else(|| t!("set.stateRequired").to_string())?;

        let mut what_if_metadata: Vec<String> = Vec::new();

        let needs_reboot = match desired_state {
            FeatureState::Installed => {
                let source_paths = feature_input
                    .source_paths
                    .as_deref()
                    .unwrap_or(&[]);
                let limit_access = feature_input.limit_access.unwrap_or(false);
                let enable_all = feature_input.enable_all.unwrap_or(false);
                if what_if {
                    what_if_metadata.push(t!("windows_feature_helper.whatIfEnable", name = feature_name.as_str()).to_string());
                    false
                } else {
                    session.enable_feature(feature_name, source_paths, limit_access, enable_all)?
                }
            }
            FeatureState::NotPresent => {
                if what_if {
                    what_if_metadata.push(t!("windows_feature_helper.whatIfDisable", name = feature_name.as_str()).to_string());
                    false
                } else {
                    session.disable_feature(feature_name, false)?
                }
            }
            FeatureState::Removed => {
                if what_if {
                    what_if_metadata.push(t!("windows_feature_helper.whatIfRemove", name = feature_name.as_str()).to_string());
                    false
                } else {
                    session.disable_feature(feature_name, true)?
                }
            }
            _ => {
                return Err(t!(
                    "set.unsupportedDesiredState",
                    state = desired_state.to_string()
                )
                .to_string());
            }
        };

        if what_if {
            results.push(WindowsFeatureInfo {
                feature_name: feature_input.feature_name.clone(),
                state: feature_input.state.clone(),
                enable_all: feature_input.enable_all,
                source_paths: feature_input.source_paths.clone(),
                limit_access: feature_input.limit_access,
                metadata: if what_if_metadata.is_empty() {
                    None
                } else {
                    Some(Metadata { what_if: Some(what_if_metadata) })
                },
                ..Default::default()
            });
        } else {
            reboot_required = reboot_required || needs_reboot;
            let info = session.get_feature_info(feature_name)?;
            results.push(info);
        }
    }

    let restart_required_meta = if !what_if && reboot_required {
        let mut entry = Map::new();
        entry.insert("system".to_string(), Value::String(get_computer_name()));
        Some(vec![entry])
    } else {
        None
    };

    Ok(WindowsFeatureList {
        restart_required_meta,
        features: results,
    })
}
