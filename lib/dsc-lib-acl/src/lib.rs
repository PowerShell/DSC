// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! A typed model for a Windows access control list (ACL), expressed as arrays of access control
//! entry (ACE) objects, along with Win32-backed operations to read and write it. Principal names
//! and SIDs are validated and canonicalized during construction on Windows.

rust_i18n::i18n!("locales", fallback = "en-us");

pub mod error;
pub mod model;
pub mod principal;

#[cfg(windows)]
pub mod win32;

pub use error::{AclError, InvalidEntryError};
pub use model::{
    AccessControlEntry, AccessControlList, AccessMask, AceFlag, AceType, AuditFlag, bits_to_mask,
    is_satisfied, mask_to_bits, merge_access_control_list, merge_entries,
};
pub use principal::Principal;
