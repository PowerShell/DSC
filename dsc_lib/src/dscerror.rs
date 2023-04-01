use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DscError {
    #[error("Command: [{0}] {1}")]
    Command(i32, String),

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
}
