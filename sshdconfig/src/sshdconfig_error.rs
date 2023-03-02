use thiserror::Error;

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_UNSPECIFIED_ERR: i32 = 1;
pub const EXIT_INPUT_INVALID: i32 = 2;
pub const EXIT_INPUT_UNAVAILABLE: i32 = 3;
pub const EXIT_CONFIG_NOT_FOUND: i32 = 4;
pub const EXIT_NOT_IN_DESIRED_STATE: i32 = 5;

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
