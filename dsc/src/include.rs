// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Include {
    /// The path to the file to include.  Path is relative to the file containing the include
    /// and not allowed to reference parent directories.  If a configuration document is used
    /// instead of a file, then the path is relative to the current working directory.
    #[serde(rename = "configurationFile")]
    pub configuration_file: String,
    pub parameters_file: Option<String>,
}
