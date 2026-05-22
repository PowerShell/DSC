// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;

use crate::dism::DismSessionHandle;
use crate::types::{WindowsFeatureInfo, WindowsFeatureList};

pub fn handle_get(input: &WindowsFeatureList) -> Result<WindowsFeatureList, String> {
    if input.features.is_empty() {
        return Err(t!("get.featuresArrayEmpty").to_string());
    }

    let session = DismSessionHandle::open()?;
    let mut results: Vec<WindowsFeatureInfo> = Vec::new();

    for feature_input in &input.features {
        let feature_name = feature_input
            .feature_name
            .as_ref()
            .ok_or_else(|| t!("get.featureNameRequired").to_string())?;

        let info = session.get_feature_info(feature_name)?;
        results.push(info);
    }

    Ok(WindowsFeatureList {
        restart_required_meta: None,
        features: results,
    })
}
