// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::dscresources::resource_manifest::ArgKind;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct DiscoverMethod {
    /// The command to run to get the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Get.
    pub args: Option<Vec<ArgKind>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct DiscoverResult {
    /// The path to the resource manifest, must be absolute.
    #[serde(rename = "manifestPath")]
    pub manifest_path: String,
}
