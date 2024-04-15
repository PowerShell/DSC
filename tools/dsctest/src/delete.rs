// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Delete {
    #[serde(rename = "deleteCalled", skip_serializing_if = "Option::is_none")]
    pub delete_called: Option<bool>,
    #[serde(rename = "_exist")]
    pub exist: bool,
}
