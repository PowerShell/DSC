// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::num::NonZeroU64;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::settings::DscSettingsResolvedField;

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(inline)]
pub struct ResourceTimeoutsFileData {
    #[serde(rename = "initializeTimeoutMS", skip_serializing_if = "Option::is_none")]
    pub initialize_timeout_ms: Option<NonZeroU64>,
    #[serde(rename = "requestTimeoutMS", skip_serializing_if = "Option::is_none")]
    pub request_timeout_ms: Option<u64>,
    #[serde(rename = "shutdownTimeoutMS", skip_serializing_if = "Option::is_none")]
    pub shutdown_timeout_ms: Option<NonZeroU64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(inline)]
pub struct ResourceTimeoutsResolvedSettings {
    #[serde(rename = "initializeTimeoutMS")]
    pub initialize_timeout_ms: DscSettingsResolvedField<NonZeroU64>,
    #[serde(rename = "requestTimeoutMS")]
    pub request_timeout_ms: DscSettingsResolvedField<u64>,
    #[serde(rename = "shutdownTimeoutMS")]
    pub shutdown_timeout_ms: DscSettingsResolvedField<NonZeroU64>,
}

impl Default for ResourceTimeoutsResolvedSettings {
    fn default() -> Self {
        Self {
            initialize_timeout_ms: DscSettingsResolvedField::for_code_default(NonZeroU64::new(250).unwrap()),
            request_timeout_ms: DscSettingsResolvedField::for_code_default(0),
            shutdown_timeout_ms: DscSettingsResolvedField::for_code_default(NonZeroU64::new(250).unwrap()),
        }
    }
}
