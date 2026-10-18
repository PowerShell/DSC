// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! A portable typed model for a Windows access control list (ACL), expressed as arrays of
//! access control entry (ACE) objects, along with Win32-backed operations to read and write it.
//!
//! The [`model`] module (DACL/SACL entries, access mask traits, flags) is available on every
//! platform so that resources can share the JSON shape and schema regardless of the host OS.
//! The [`win32`] module, which actually talks to Win32 APIs, is only compiled on Windows.

pub mod error;
pub mod model;

#[cfg(windows)]
pub mod win32;

pub use error::AclError;
pub use model::{
    AccessControlEntry, AccessControlList, AccessMask, AceFlag, AceType, AuditFlag, bits_to_mask,
    is_satisfied, mask_to_bits, merge_access_control_list, merge_entries,
};
