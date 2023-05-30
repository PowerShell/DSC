// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use ntapi::winapi::shared::ntdef::{NTSTATUS};
use ntapi::winapi::shared::ntstatus::{STATUS_OBJECT_NAME_NOT_FOUND, STATUS_OBJECT_PATH_NOT_FOUND, STATUS_OBJECT_PATH_SYNTAX_BAD, STATUS_ACCESS_DENIED, STATUS_KEY_DELETED, STATUS_CANNOT_DELETE};
use std::fmt;
use thiserror::Error;

/// Struct for returning NTSTATUS errors
#[derive(Debug, PartialEq, Eq)]
pub struct NtStatusError {
    pub status: NtStatusErrorKind,
    pub message: String,
}

impl NtStatusError {
    /// Create a new `NtStatusError` from an NTSTATUS error code and a message.
    /// 
    /// # Arguments
    ///
    /// * `status` - The NTSTATUS error code
    /// * `message` - The message to return such as the current action
    ///
    /// # Example
    ///
    /// ```
    /// # use ntstatuserror::NtStatusError;
    /// # use ntapi::winapi::shared::ntstatus::{STATUS_OBJECT_NAME_NOT_FOUND};
    /// let error = NtStatusError::new(STATUS_OBJECT_NAME_NOT_FOUND, "Could not find object");
    /// assert_eq!(error.status, ntstatuserror::NtStatusErrorKind::ObjectNameNotFound);
    /// assert_eq!(error.message, "Could not find object".to_string());
    /// ```
    #[must_use]
    pub fn new(status: NTSTATUS, message: &str) -> Self {
        NtStatusError {
            status: NtStatusErrorKind::from(status),
            message: message.to_string(),
        }
    }
}

/// Error codes returned by the NT API.
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum NtStatusErrorKind {
    /// The registry key or value name does not exist.
    #[error("The registry key or value name does not exist.")]
    ObjectNameNotFound,
    /// The registry key or value path is invalid.
    #[error("The registry key or value path is invalid.")]
    ObjectPathNotFound,
    /// The registry key or value path is invalid.
    #[error("The registry key or value path is invalid.")]
    ObjectPathSyntaxBad,
    /// Insufficient access rights to perform the operation.
    #[error("Insufficient access rights to perform the operation.")]
    AccessDenied,
    /// The registry key or value has been deleted.
    #[error("The registry key or value has been deleted.")]
    KeyDeleted,
    /// The registry key or value cannot be deleted.
    #[error("The registry key or value cannot be deleted.")]
    CannotDelete,
    /// Unknown error.
    #[error("Unknown error: {0:#x}")]
    Unknown(i32),
}

impl From<NTSTATUS> for NtStatusErrorKind {
    fn from(status: NTSTATUS) -> Self {
        match status {
            STATUS_OBJECT_NAME_NOT_FOUND => NtStatusErrorKind::ObjectNameNotFound,
            STATUS_OBJECT_PATH_NOT_FOUND => NtStatusErrorKind::ObjectPathNotFound,
            STATUS_OBJECT_PATH_SYNTAX_BAD => NtStatusErrorKind::ObjectPathSyntaxBad,
            STATUS_ACCESS_DENIED => NtStatusErrorKind::AccessDenied,
            STATUS_KEY_DELETED => NtStatusErrorKind::KeyDeleted,
            STATUS_CANNOT_DELETE => NtStatusErrorKind::CannotDelete,
            _ => NtStatusErrorKind::Unknown(status),
        }
    }
}

impl fmt::Display for NtStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.status, self.message)
    }
}
