// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_field_names)]
pub struct RestartRequired {
    #[serde(rename="_restartRequired", skip_serializing_if = "Option::is_none")]
    pub restart_required: Option<Vec<Map<String, Value>>>,
}
