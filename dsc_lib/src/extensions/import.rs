// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct ImportMethod {
    /// The extensions to import.
    pub extensions: Vec<String>,
    /// The command to run to get the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Get.
    pub args: Option<Vec<ImportArgKind>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum ImportArgKind {
    /// The argument is a string.
    String(String),
    /// The argument accepts the file path.
    File {
        /// The argument that accepts the file path.
        #[serde(rename = "fileArg")]
        file_arg: String,
    },
}
