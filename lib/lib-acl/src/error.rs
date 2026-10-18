// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use std::fmt::{Display, Formatter};

/// Errors that can occur while modeling or applying an access control list.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum AclError {
    /// An access control entry's shape is invalid for the section it belongs to, for example an
    /// `audit` entry in a discretionary list, or an `allow`/`deny` entry in a system list.
    InvalidEntry(String),

    /// The principal (SID string or account name) could not be resolved to a security
    /// identifier.
    PrincipalResolution { principal: String, reason: String },

    /// A value that was expected to be a SID string was not formatted correctly.
    InvalidSid(String),

    /// A Win32 API call failed.
    Win32 { operation: &'static str, code: u32 },

    /// The current process does not hold a privilege required to manage the system access
    /// control list (SACL).
    PrivilegeNotHeld(&'static str),

    /// An access control entry of an unsupported type (for example a conditional or
    /// object-specific ACE) was encountered while reading an existing ACL.
    UnsupportedAceType(u8),

    /// An access mask read from an existing access control entry contains one or more bits that
    /// don't correspond to any value in the resource's [`crate::model::AccessMask`] enum.
    /// Returned instead of silently discarding the unrecognized bits, which would otherwise let
    /// a subsequent merge-and-rewrite of the access control list narrow its effective access
    /// mask without any indication that it had done so.
    UnrepresentableAccessMask(u32),

    /// The discretionary access control list is NULL (absent), which grants unrestricted access.
    /// This is distinct from a present-but-empty DACL, which denies all access by default.
    NullAcl(&'static str),
}

impl Display for AclError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidEntry(reason) => {
                t!("error.invalidEntry", reason = reason).to_string()
            }
            Self::PrincipalResolution { principal, reason } => t!(
                "error.principalResolution",
                principal = principal,
                reason = reason
            )
            .to_string(),
            Self::InvalidSid(value) => t!("error.invalidSid", value = value).to_string(),
            Self::Win32 { operation, code } => {
                t!("error.win32", operation = operation, code = code).to_string()
            }
            Self::PrivilegeNotHeld(privilege) => {
                t!("error.privilegeNotHeld", privilege = privilege).to_string()
            }
            Self::UnsupportedAceType(ace_type) => {
                t!("error.unsupportedAceType", ace_type = ace_type).to_string()
            }
            Self::UnrepresentableAccessMask(mask) => {
                t!("error.unrepresentableAccessMask", mask = format!("{mask:#010x}")).to_string()
            }
            Self::NullAcl(section) => {
                let section = match *section {
                    "discretionary" => t!("section.discretionary"),
                    "system" => t!("section.system"),
                    _ => (*section).into(),
                };
                t!("error.nullAcl", section = section).to_string()
            }
        };
        formatter.write_str(&message)
    }
}

impl std::error::Error for AclError {}
