// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Validates that a file or folder can only be changed by trusted principals.
//!
//! DSC reads security sensitive files such as the policy settings file. Before trusting such a
//! file, DSC verifies that nobody other than administrators (and, for user scoped files, the
//! current user) can write to it or to the folder that contains it. This crate implements that
//! check for Windows and Unix platforms behind one API.
//!
//! # Scopes
//!
//! The [`PathSecurityScope`] decides which principals may hold write access:
//!
//! 1. [`Machine`]: only the local system account and the built in Administrators group on
//!    Windows, and only the root user on Unix.
//! 1. [`User`]: the principals trusted by the machine scope plus the account the current process
//!    runs as.
//!
//! # Rules
//!
//! A path is secure for a scope when all of the following hold:
//!
//! 1. The path exists and is a file or a folder.
//! 1. The owner of the path is a trusted principal.
//! 1. On Windows, the path has a discretionary access control list and no allow entry that
//!    applies to the path itself grants a write right to an untrusted principal. Inherit only
//!    entries and deny entries are ignored.
//! 1. On Unix, neither the group write bit nor the other write bit is set.
//! 1. For a file, the containing folder satisfies the same rules. When the file is a symbolic
//!    link, the folder containing the link target is validated as well.
//!
//! Every failure to read security information is treated as insecure, so the check fails
//! closed.
//!
//! [`Machine`]: PathSecurityScope::Machine
//! [`User`]: PathSecurityScope::User

use std::path::Path;

use rust_i18n::t;

rust_i18n::i18n!("locales", fallback = "en-us");

mod error;
mod model;
#[cfg(not(target_os = "windows"))]
mod unix;
#[cfg(target_os = "windows")]
mod win32;

#[cfg(not(target_os = "windows"))]
use unix as platform;
#[cfg(target_os = "windows")]
use win32 as platform;

pub use error::PathSecurityError;

/// Identifies which principals may hold write access to a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathSecurityScope {
    /// Only administrators may write. On Windows these are the local system account and the
    /// built in Administrators group. On Unix this is the root user.
    Machine,
    /// Administrators and the account the current process runs as may write.
    User,
}

/// Validates that a file or folder is writable only by the principals a scope trusts.
///
/// A folder is validated on its own. A file is validated together with the folder that contains
/// it and, when the file is a symbolic link, together with the folder that contains the link
/// target. Folders are validated before the file so that a problem with the folder is reported
/// first.
///
/// # Arguments
///
/// 1. `path`: the file or folder to validate. Relative paths are resolved against the current
///    working directory.
/// 1. `scope`: the scope whose trust rules apply.
///
/// # Errors
///
/// Returns a [`PathSecurityError`] when the path does not exist, is neither a file nor a folder,
/// cannot be inspected, or grants ownership or write access to an untrusted principal.
pub fn validate_path(path: &Path, scope: PathSecurityScope) -> Result<(), PathSecurityError> {
    let absolute = std::path::absolute(path).map_err(|error| io_error(path, &error))?;
    let metadata = std::fs::metadata(&absolute).map_err(|error| io_error(&absolute, &error))?;

    if metadata.is_dir() {
        return platform::validate_object(&absolute, scope);
    }
    if !metadata.is_file() {
        return Err(PathSecurityError::UnsupportedFileType {
            path: path_string(&absolute),
        });
    }

    validate_containing_folders(&absolute, scope)?;
    platform::validate_object(&absolute, scope)
}

/// Validates the folder that contains a file and, for a symbolic link, the folder that contains
/// the link target.
///
/// # Arguments
///
/// 1. `file`: the absolute path of the file.
/// 1. `scope`: the scope whose trust rules apply.
///
/// # Errors
///
/// Returns a [`PathSecurityError`] when the file has no containing folder, when the link target
/// cannot be resolved, or when either folder fails validation.
fn validate_containing_folders(file: &Path, scope: PathSecurityScope) -> Result<(), PathSecurityError> {
    let Some(parent) = file.parent() else {
        return Err(PathSecurityError::SecurityInfoRead {
            path: path_string(file),
            reason: t!("error.noParentFolder").to_string(),
        });
    };
    platform::validate_object(parent, scope)?;

    let link_metadata = std::fs::symlink_metadata(file).map_err(|error| io_error(file, &error))?;
    if link_metadata.file_type().is_symlink() {
        let target = std::fs::canonicalize(file).map_err(|error| io_error(file, &error))?;
        if let Some(target_parent) = target.parent() {
            platform::validate_object(target_parent, scope)?;
        }
    }

    Ok(())
}

/// Converts a path to the string form used in error messages.
///
/// # Arguments
///
/// 1. `path`: the path to convert.
///
/// # Returns
///
/// The display form of the path.
pub(crate) fn path_string(path: &Path) -> String {
    path.display().to_string()
}

/// Converts an I/O error for a path into a [`PathSecurityError`].
///
/// # Arguments
///
/// 1. `path`: the path the operation failed on.
/// 1. `error`: the I/O error.
///
/// # Returns
///
/// [`PathSecurityError::PathNotFound`] when the path does not exist and
/// [`PathSecurityError::SecurityInfoRead`] for every other failure.
pub(crate) fn io_error(path: &Path, error: &std::io::Error) -> PathSecurityError {
    if error.kind() == std::io::ErrorKind::NotFound {
        PathSecurityError::PathNotFound {
            path: path_string(path),
        }
    } else {
        PathSecurityError::SecurityInfoRead {
            path: path_string(path),
            reason: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_relative_path_is_reported_with_an_absolute_path() {
        let relative = Path::new("dsc-lib-path_security-missing-folder");
        match validate_path(relative, PathSecurityScope::User) {
            Err(PathSecurityError::PathNotFound { path }) => {
                assert!(Path::new(&path).is_absolute(), "expected an absolute path, got {path}");
                assert!(path.ends_with("dsc-lib-path_security-missing-folder"), "unexpected path {path}");
            }
            other => panic!("expected a missing path error, got {other:?}"),
        }
    }

    #[test]
    fn errors_render_with_their_path() {
        let error = PathSecurityError::UntrustedWriter {
            path: "example".into(),
            principal: "S-1-1-0".into(),
        };
        let message = error.to_string();
        assert!(message.contains("example"), "unexpected message {message}");
        assert!(message.contains("S-1-1-0"), "unexpected message {message}");
    }
}
