// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;

use crate::optional_feature::dism::DismSessionHandle;
use crate::optional_feature::types::OptionalFeatureList;

pub fn handle_get(input: &str) -> Result<String, String> {
    let feature_list: OptionalFeatureList = serde_json::from_str(input)
        .map_err(|e| t!("get.failedParseInput", err = e.to_string()).to_string())?;

    if feature_list.features.is_empty() {
        return Err(t!("get.featuresArrayEmpty").to_string());
    }

    let session = DismSessionHandle::open()?;
    let mut results = Vec::new();

    for feature_input in &feature_list.features {
        let feature_name = feature_input
            .feature_name
            .as_ref()
            .ok_or_else(|| t!("get.featureNameRequired").to_string())?;

        let info = session.get_feature_info(feature_name)?;
        results.push(info);
    }

    let output = OptionalFeatureList { restart_required_meta: None, features: results };
    serde_json::to_string(&output)
        .map_err(|e| t!("get.failedSerializeOutput", err = e.to_string()).to_string())
}
