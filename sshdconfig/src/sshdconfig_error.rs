use thiserror::Error;

#[derive(Error, Debug)]
pub enum SshdConfigError {
    #[error("Not implemented")]
    NotImplemented,

    #[error("Unknown: {code:?} {message:?}")]
    Unknown {
        code: i32,
        message: String,
    },
}
