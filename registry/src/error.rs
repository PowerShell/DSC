// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use indicatif::style::TemplateError;
use reqwest::StatusCode;
use std::str::Utf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
}
