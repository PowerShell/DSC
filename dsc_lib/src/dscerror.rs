use thiserror::Error;

#[derive(Error, Debug)]
pub enum DscError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Error: {0}")]
    Operation(String),

    #[error("Not implemented")]
    NotImplemented,

    #[error("Unknown: {code:?} {message:?}")]
    Unknown {
        code: i32,
        message: String,
    },
}
