// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib_registry::config::Registry;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegistryList {
    pub registry_entries: Vec<Registry>,
}
