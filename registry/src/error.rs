// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum RegistryError {
    #[error("{t}: {0}", t = t!("errors.InvalidHive"))]
    InvalidHive(String),

    #[error("{t}: {0}", t = t!("errors.Json"))]
    Json(#[from] serde_json::Error),

    #[error("{t}: {0}", t = t!("errors.Registry"))]
    Registry(#[from] registry::Error),

    #[error("{t}: {0}", t = t!("errors.RegistryKey"))]
    RegistryKey(#[from] registry::key::Error),

    #[error("{t}: {0}", t = t!("errors.RegistryKeyNotFound"))]
    RegistryKeyNotFound(String),

    #[error("{t}: {0}", t = t!("errors.RegistryValue"))]
    RegistryValue(#[from] registry::value::Error),

    #[error("{t}: {0}", t = t!("errors.Utf16Conversion"))]
    Utf16Conversion(String),

    #[error("{t}", t = t!("errors.UnsupportedValueDataType"))]
    UnsupportedValueDataType,
}
