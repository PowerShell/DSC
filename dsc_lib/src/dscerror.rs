// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::str::Utf8Error;

use indicatif::style::TemplateError;
use reqwest::StatusCode;
use thiserror::Error;
use tracing::error;
use tree_sitter::LanguageError;

#[derive(Error, Debug)]
pub enum DscError {
    #[error("Function boolean argument conversion error: {0}")]
    BooleanConversion(#[from] std::str::ParseBoolError),

    #[error("Command: Resource '{0}' [Exit code {1}] {2}")]
    Command(String, i32, String),

    #[error("CommandOperation: {0} for executable '{1}'")]
    CommandOperation(String, String),

    #[error("Function '{0}' error: {1}")]
    Function(String, String),

    #[error("HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("HTTP status: {0}")]
    HttpStatus(StatusCode),

    #[error("Function integer argument conversion error: {0}")]
    IntegerConversion(#[from] std::num::ParseIntError),

    #[error("Invalid configuration:\n{0}")]
    InvalidConfiguration(String),

    #[error("Unsupported manifest version: {0}.  Must be: {1}")]
    InvalidManifestSchemaVersion(String, String),

    #[error("Invalid function parameter count for '{0}', expected {1}, got {2}")]
    InvalidFunctionParameterCount(String, usize, usize),

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Language: {0}")]
    Language(#[from] LanguageError),

    #[error("Manifest: {0}\nJSON: {1}")]
    Manifest(String, serde_json::Error),

    #[error("Manifest: {0}\nYAML: {1}")]
    ManifestYaml(String, serde_yaml::Error),

    #[error("Missing manifest: {0}")]
    MissingManifest(String),

    #[error("Adapter-based resource '{0}' missing 'requires' property for resource '{1}'")]
    MissingRequires(String, String),

    #[error("Schema missing from manifest: {0}")]
    MissingSchema(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Number conversion error: {0}")]
    NumberConversion(#[from] std::num::TryFromIntError),

    #[error("Operation: {0}")]
    Operation(String),

    #[error("Parser: {0}")]
    Parser(String),

    #[error("Progress: {0}")]
    Progress(#[from] TemplateError),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Schema: {0}")]
    Schema(String),

    #[error("No Schema: {0}")]
    SchemaNotAvailable(String),

    #[error("Security context: {0}")]
    SecurityContext(String),

    #[error("Utf-8 conversion error: {0}")]
    Utf8Conversion(#[from] Utf8Error),

    #[error("Unknown: {code:?} {message:?}")]
    Unknown {
        code: i32,
        message: String,
    },

    #[error("Validation: {0}")]
    Validation(String),
}
