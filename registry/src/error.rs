// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum RegistryError {
    #[error("Invalid hive: {0}.")]
    InvalidHive(String),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Registry: {0}")]
    Registry(#[from] registry::Error),

    #[error("Registry key: {0}")]
    RegistryKey(#[from] registry::key::Error),

    #[error("Registry key not found: {0}")]
    RegistryKeyNotFound(String),

    #[error("Registry value: {0}")]
    RegistryValue(#[from] registry::value::Error),

    #[error("UTF-16 conversion of {0} failed due to interior NULL values")]
    Utf16Conversion(String),

    #[error("Unsupported registry value data type")]
    UnsupportedValueDataType,
}
