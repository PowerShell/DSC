// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use thiserror::Error;

/// A specific validation failure in an access control entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum InvalidEntryError {
    /// A discretionary entry specifies audit flags.
    #[error("{t}", t = t!("validation.discretionaryAuditFlags"))]
    DiscretionaryAuditFlags,

    /// A discretionary entry has a type other than `allow` or `deny`.
    #[error("{t}", t = t!("validation.discretionaryType"))]
    DiscretionaryType,

    /// An entry has no access mask values.
    #[error("{t}", t = t!("validation.accessMaskRequired"))]
    MissingAccessMask,

    /// A system entry has no audit flags.
    #[error("{t}", t = t!("validation.systemAuditFlags"))]
    SystemAuditFlags,

    /// A system entry has a type other than `audit`.
    #[error("{t}", t = t!("validation.systemType"))]
    SystemType,
}

fn format_validation_errors(errors: &[InvalidEntryError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

fn localized_section(section: &str) -> String {
    match section {
        "discretionary" => t!("section.discretionary").to_string(),
        "system" => t!("section.system").to_string(),
        _ => section.to_string(),
    }
}

/// Errors that can occur while modeling or applying an access control list.
#[derive(Debug, Error)]
#[allow(clippy::module_name_repetitions)]
pub enum AclError {
    /// One or more access control entry validation rules failed.
    #[error(
        "{t}",
        t = t!("error.invalidEntry", reason = format_validation_errors(errors))
    )]
    InvalidEntry { errors: Vec<InvalidEntryError> },

    /// The principal (SID string or account name) could not be resolved to a security
    /// identifier.
    #[error(
        "{t}",
        t = t!(
            "error.unresolvablePrincipal",
            principal = principal,
            reason = reason
        )
    )]
    UnresolvablePrincipal { principal: String, reason: String },

    /// A value that was expected to be a SID string was not formatted correctly.
    #[error("{t}", t = t!("error.invalidSid", value = .0))]
    InvalidSid(String),

    /// A Win32 API call failed.
    #[error("{t}", t = t!("error.win32", operation = operation, code = code))]
    Win32 { operation: &'static str, code: u32 },

    /// The current process does not hold a privilege required to manage the system access
    /// control list (SACL).
    #[error("{t}", t = t!("error.privilegeNotHeld", privilege = .0))]
    PrivilegeNotHeld(&'static str),

    /// An access control entry of an unsupported type (for example a conditional or
    /// object-specific ACE) was encountered while reading an existing ACL.
    #[error("{t}", t = t!("error.unsupportedAceType", ace_type = .0))]
    UnsupportedAceType(u8),

    /// An access mask read from an existing access control entry contains one or more bits that
    /// don't correspond to any value in the resource's [`crate::model::AccessMask`] enum.
    /// Returned instead of silently discarding the unrecognized bits, which would otherwise let
    /// a subsequent merge-and-rewrite of the access control list narrow its effective access
    /// mask without any indication that it had done so.
    #[error(
        "{t}",
        t = t!(
            "error.unrepresentableAccessMask",
            mask = format!("{:#010x}", .0)
        )
    )]
    UnrepresentableAccessMask(u32),

    /// The discretionary access control list is NULL (absent), which grants unrestricted access.
    /// This is distinct from a present-but-empty DACL, which denies all access by default.
    #[error(
        "{t}",
        t = t!("error.nullAcl", section = localized_section(.0))
    )]
    NullAcl(&'static str),
}
