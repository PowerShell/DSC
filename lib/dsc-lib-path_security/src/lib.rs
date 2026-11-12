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
//! 1. For a file, the containing folder satisfies the same rules.
//!
//! Every failure to read security information is treated as insecure, so the check fails
//! closed.
//!
//! # Links
//!
//! Symbolic links and, on Windows, junctions are resolved one link at a time. The rules apply to
//! the file or folder at the end of the chain, and every link on the way is validated too:
//!
//! 1. On Windows, the link itself must satisfy the owner and access control list rules, because
//!    a principal with write access to a link can point it somewhere else. Unix has no
//!    permissions on a link, only on the folder that contains it.
//! 1. When a file is validated, the folder containing each link must satisfy the rules, because
//!    a principal with write access to that folder can replace the link.
//!
//! # Limits
//!
//! Only the folder that directly contains a file or a link to a file is validated. Folders
//! further up the path are not inspected, and a folder is validated without its containing
//! folder.
//!
//! [`Machine`]: PathSecurityScope::Machine
//! [`User`]: PathSecurityScope::User

use std::path::{Path, PathBuf};

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

/// The number of links followed before a path is rejected.
///
/// The operating systems stop resolving a path earlier, Linux after 40 links and Windows after
/// 63, so this limit is only reached when a path changes while it is being validated.
const MAX_LINK_HOPS: usize = 64;

/// The kind of object a path resolves to once every link is followed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjectKind {
    /// A regular file, which is validated together with the folder that contains it.
    File,
    /// A folder, which is validated on its own.
    Folder,
}

/// Validates that a file or folder is writable only by the principals a scope trusts.
///
/// A folder is validated on its own. A file is validated together with the folder that contains
/// it. Folders are validated before the file so that a problem with the folder is reported
/// first.
///
/// Symbolic links and, on Windows, junctions are followed one link at a time. Every link on the
/// way is validated as described in the [crate documentation](crate#links), so a chain of links
/// is only secure when no untrusted principal can change any link in it.
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
/// cannot be inspected, or grants ownership or write access to an untrusted principal. The path
/// in the error is the link, folder, or file that failed.
pub fn validate_path(path: &Path, scope: PathSecurityScope) -> Result<(), PathSecurityError> {
    let absolute = std::path::absolute(path).map_err(|error| io_error(path, &error))?;
    let metadata = std::fs::metadata(&absolute).map_err(|error| io_error(&absolute, &error))?;

    let kind = if metadata.is_dir() {
        ObjectKind::Folder
    } else if metadata.is_file() {
        ObjectKind::File
    } else {
        return Err(PathSecurityError::UnsupportedFileType {
            path: path_string(&absolute),
        });
    };

    validate_resolved(&absolute, scope, kind)
}

/// Follows the links at the end of a path one at a time and validates every object on the way.
///
/// For each link the function validates the link itself and then continues with the link
/// target. The object at the end of the chain is validated last. When a file is validated, the
/// folder containing each link and the folder containing the file are validated first.
///
/// # Arguments
///
/// 1. `path`: the absolute path of the file or folder.
/// 1. `scope`: the scope whose trust rules apply.
/// 1. `kind`: whether the path resolves to a file or a folder.
///
/// # Errors
///
/// Returns a [`PathSecurityError`] when a link, a containing folder, or the final object fails
/// validation, when a link cannot be read, when the chain has more than [`MAX_LINK_HOPS`] links,
/// or when the chain ends at a different object than the operating system resolves the path to.
fn validate_resolved(path: &Path, scope: PathSecurityScope, kind: ObjectKind) -> Result<(), PathSecurityError> {
    let mut current = path.to_path_buf();
    for _ in 0..=MAX_LINK_HOPS {
        if kind == ObjectKind::File {
            validate_containing_folder(&current, scope)?;
        }
        if !is_link(&current)? {
            ensure_same_object(path, &current)?;
            return platform::validate_object(&current, scope);
        }
        // The platform check never follows a link at the end of a path, so this validates the
        // link itself. Unix has no permissions on a link, so only Windows needs it.
        #[cfg(target_os = "windows")]
        platform::validate_object(&current, scope)?;
        current = link_target(&current)?;
    }

    Err(PathSecurityError::SecurityInfoRead {
        path: path_string(path),
        reason: t!("error.tooManyLinks", limit = MAX_LINK_HOPS).to_string(),
    })
}

/// Validates the folder that contains a file or a link to a file.
///
/// # Arguments
///
/// 1. `file`: the absolute path of the file or link.
/// 1. `scope`: the scope whose trust rules apply.
///
/// # Errors
///
/// Returns a [`PathSecurityError`] when the path has no containing folder or when the folder
/// fails validation.
fn validate_containing_folder(file: &Path, scope: PathSecurityScope) -> Result<(), PathSecurityError> {
    let Some(parent) = file.parent() else {
        return Err(PathSecurityError::SecurityInfoRead {
            path: path_string(file),
            reason: t!("error.noParentFolder").to_string(),
        });
    };
    validate_resolved(parent, scope, ObjectKind::Folder)
}

/// Indicates whether a path names a link without following it.
///
/// # Arguments
///
/// 1. `path`: the absolute path to inspect.
///
/// # Returns
///
/// `true` for a symbolic link and, on Windows, for a junction.
///
/// # Errors
///
/// Returns a [`PathSecurityError`] when the path cannot be inspected.
fn is_link(path: &Path) -> Result<bool, PathSecurityError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| io_error(path, &error))?;
    Ok(metadata.file_type().is_symlink())
}

/// Reads where a link points to.
///
/// # Arguments
///
/// 1. `link`: the absolute path of the link.
///
/// # Returns
///
/// The link target. A relative target is joined to the folder that contains the link.
///
/// # Errors
///
/// Returns a [`PathSecurityError`] when the link cannot be read.
fn link_target(link: &Path) -> Result<PathBuf, PathSecurityError> {
    let target = std::fs::read_link(link).map_err(|error| io_error(link, &error))?;
    Ok(match link.parent() {
        Some(parent) => parent.join(target),
        None => target,
    })
}

/// Verifies that following the links by hand ended at the object the operating system resolves
/// the path to.
///
/// The links are followed by joining each target to the folder of its link, which can differ
/// from how the operating system resolves a relative target behind another link. Validating a
/// different object than the one that is later read would make the result meaningless, so a
/// difference is treated as insecure.
///
/// # Arguments
///
/// 1. `original`: the path that was passed in for validation.
/// 1. `resolved`: the path the chain of links ended at.
///
/// # Errors
///
/// Returns a [`PathSecurityError`] when either path cannot be resolved or when they resolve to
/// different objects.
fn ensure_same_object(original: &Path, resolved: &Path) -> Result<(), PathSecurityError> {
    if original == resolved {
        return Ok(());
    }
    let expected = std::fs::canonicalize(original).map_err(|error| io_error(original, &error))?;
    let actual = std::fs::canonicalize(resolved).map_err(|error| io_error(resolved, &error))?;
    if expected == actual {
        return Ok(());
    }

    Err(PathSecurityError::SecurityInfoRead {
        path: path_string(original),
        reason: t!("error.linkTargetMismatch", resolved = path_string(resolved)).to_string(),
    })
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
