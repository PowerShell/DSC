// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib_registry::config::Registry;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegistryList {
    pub registry_entries: Vec<Registry>,
    /// Optional path to an offline registry hive file. When specified, operations
    /// are performed against the offline hive instead of the live system registry.
    #[serde(rename = "registryFilePath", skip_serializing_if = "Option::is_none")]
    pub registry_file_path: Option<String>,
    /// Whether every entry in `registryEntries` is in the desired state. Only returned by a
    /// `test` operation.
    #[serde(rename = "_inDesiredState", skip_serializing_if = "Option::is_none")]
    pub in_desired_state: Option<bool>,
}
