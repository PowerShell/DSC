// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    #[serde(rename="_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SubMetadata>
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<String>>
}
