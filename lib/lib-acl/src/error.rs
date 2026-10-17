// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use thiserror::Error;

/// Errors that can occur while modeling or applying an access control list.
#[derive(Debug, Error)]
#[allow(clippy::module_name_repetitions)]
pub enum AclError {
    /// An access control entry's shape is invalid for the section it belongs to, for example an
    /// `audit` entry in a discretionary list, or an `allow`/`deny` entry in a system list.
    #[error("Invalid access control entry: {0}")]
    InvalidEntry(String),

    /// The principal (SID string or account name) could not be resolved to a security
    /// identifier.
    #[error("Failed to resolve principal '{principal}' to a security identifier: {reason}")]
    PrincipalResolution { principal: String, reason: String },

    /// A value that was expected to be a SID string was not formatted correctly.
    #[error("'{0}' is not a valid security identifier (SID) string")]
    InvalidSid(String),

    /// A Win32 API call failed.
    #[error("{operation} failed with error code {code}")]
    Win32 { operation: &'static str, code: u32 },

    /// The current process does not hold a privilege required to manage the system access
    /// control list (SACL).
    #[error(
        "The '{0}' privilege is required to read or write the system access control list (SACL) \
         but is not held by the current process. Run with elevated (Administrator) rights and \
         ensure the account has the 'Manage auditing and security log' user right assigned."
    )]
    PrivilegeNotHeld(&'static str),

    /// An access control entry of an unsupported type (for example a conditional or
    /// object-specific ACE) was encountered while reading an existing ACL.
    #[error("Access control entry type {0} is not supported")]
    UnsupportedAceType(u8),
}
