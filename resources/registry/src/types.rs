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
}
