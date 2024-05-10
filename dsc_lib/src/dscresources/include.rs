// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Include {
    /// The path to the file to include.  Relative paths are relative to the file containing the include
    /// and not allowed to reference parent directories.
    #[serde(rename = "configurationFile")]
    pub configuration_file: String,
}
