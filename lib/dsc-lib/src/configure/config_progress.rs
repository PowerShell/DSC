// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
use dsc_lib_jsonschema::transforms::idiomaticize_string_enum;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationResourceStartedEvent {
    pub resource: String,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(transform = idiomaticize_string_enum)]
pub enum ConfigurationResourceCompletionStatus {
    Success,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationResourceCompletedEvent {
    pub resource: String,
    pub parent: Option<String>,
    pub status: ConfigurationResourceCompletionStatus,
    pub errors: Option<Vec<String>>,
}
