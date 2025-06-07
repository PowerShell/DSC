// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum RegistryError {
    #[error("{t}: {0}", t = t!("error.invalidHive"))]
    InvalidHive(String),

    #[error("{t}: {0}", t = t!("error.json"))]
    Json(#[from] serde_json::Error),

    #[error("{t}: {0}", t = t!("error.registry"))]
    Registry(#[from] registry::Error),

    #[error("{t}: {0}", t = t!("error.registryKey"))]
    RegistryKey(#[from] registry::key::Error),

    #[error("{t}: {0}", t = t!("error.registryKeyNotFound"))]
    RegistryKeyNotFound(String),

    #[error("{t}: {0}", t = t!("error.registryValue"))]
    RegistryValue(#[from] registry::value::Error),

    #[error("{t}: {0}", t = t!("error.utf16Conversion"))]
    Utf16Conversion(String),

    #[error("{t}", t = t!("error.unsupportedValueDataType"))]
    UnsupportedValueDataType,
}
