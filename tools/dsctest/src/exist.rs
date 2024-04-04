// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum State {
    Present,
    Absent,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Exist {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<State>,
    #[serde(rename = "_exist")]
    pub exist: bool,
}
