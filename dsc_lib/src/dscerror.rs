use thiserror::Error;

#[derive(Error, Debug)]
pub enum DscError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Manifest: {0}\nJSON: {1}")]
    Manifest(String, serde_json::Error),

    #[error("Command: [{0}] {1}")]
    Command(i32, String),

    #[error("Missing manifest: {0}")]
    MissingManifest(String),

    #[error("Not implemented")]
    NotImplemented,

    #[error("Operation: {0}")]
    Operation(String),

    #[error("Unknown: {code:?} {message:?}")]
    Unknown {
        code: i32,
        message: String,
    },
}
