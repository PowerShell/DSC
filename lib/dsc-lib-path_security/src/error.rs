// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines the errors returned when a path fails security validation.

use rust_i18n::t;
use thiserror::Error;

/// Errors returned when a path fails security validation or when the security information for
/// a path cannot be read.
///
/// Every variant carries the path of the object that failed. When a file is validated, the
/// containing folder is validated first, so callers can tell from the path whether the folder
/// or the file caused the failure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum PathSecurityError {
    /// The path does not exist.
    #[error("{t}", t = t!("error.pathNotFound", path = path))]
    PathNotFound {
        /// The path that was not found.
        path: String,
    },
    /// The security information for the path could not be read.
    ///
    /// The check fails closed, so a path whose permissions cannot be inspected is treated as
    /// insecure.
    #[error("{t}", t = t!("error.securityInfoRead", path = path, reason = reason))]
    SecurityInfoRead {
        /// The path whose security information could not be read.
        path: String,
        /// The underlying failure, such as the failing operation and its error code.
        reason: String,
    },
    /// The path has no discretionary access control list, which grants everyone full access.
    #[error("{t}", t = t!("error.nullDacl", path = path))]
    NullDacl {
        /// The path without a discretionary access control list.
        path: String,
    },
    /// An access control entry grants write access to a principal that the requested scope does
    /// not trust.
    #[error("{t}", t = t!("error.untrustedWriter", path = path, principal = principal))]
    UntrustedWriter {
        /// The path that grants the write access.
        path: String,
        /// The principal that holds write access, as a SID string on Windows or as a user or
        /// group description on Unix.
        principal: String,
    },
    /// The path is owned by a principal that the requested scope does not trust.
    #[error("{t}", t = t!("error.untrustedOwner", path = path, principal = principal))]
    UntrustedOwner {
        /// The path with the untrusted owner.
        path: String,
        /// The owner, as a SID string on Windows or as a user description on Unix.
        principal: String,
    },
    /// The path is neither a file nor a folder.
    #[error("{t}", t = t!("error.unsupportedFileType", path = path))]
    UnsupportedFileType {
        /// The path with the unsupported type.
        path: String,
    },
}
