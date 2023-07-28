// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DscError {
    #[error("Command: Resource '{0}' [Exit code {1}] {2}")]
    Command(String, i32, String),

    #[error("CommandOperation: {0} for executable '{1}'")]
    CommandOperation(String, String),

    #[error("HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("HTTP status: {0}")]
    HttpStatus(StatusCode),

    #[error("Invalid configuration:\n{0}")]
    InvalidConfiguration(String),

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Manifest: {0}\nJSON: {1}")]
    Manifest(String, serde_json::Error),

    #[error("Missing manifest: {0}")]
    MissingManifest(String),

    #[error("Provider source '{0}' missing 'requires' property for resource '{1}'")]
    MissingRequires(String, String),

    #[error("Schema missing from manifest: {0}")]
    MissingSchema(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Operation: {0}")]
    Operation(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Schema: {0}")]
    Schema(String),

    #[error("No Schema: {0}")]
    SchemaNotAvailable(String),

    #[error("Unknown: {code:?} {message:?}")]
    Unknown {
        code: i32,
        message: String,
    },

    #[error("Validation: {0}")]
    Validation(String),
}
