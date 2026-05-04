// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistryResourceError {
    #[error("Failed to parse adapter input: {0}")]
    AdapterInputParseError(String),
    #[error("Registry error: {0}")]
    RegistryError(#[from] dsc_lib_registry::error::RegistryError),
}
