// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum SecretArgKind {
    /// The argument is a string.
    String(String),
    /// The argument accepts the secret name.
    Name {
        /// The argument that accepts the secret name.
        #[serde(rename = "nameArg")]
        name_arg: String,
    },
    /// The argument accepts the vault name.
    Vault {
        /// The argument that accepts the vault name.
        #[serde(rename = "vaultArg")]
        vault_arg: String,
    },
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct SecretMethod {
    /// The command to run to get the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Get.
    pub args: Option<Vec<SecretArgKind>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct SecretResult {
    pub secret: Option<String>,
}
