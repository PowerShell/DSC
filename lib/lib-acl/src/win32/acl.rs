// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Builds and parses Windows ACL structures (arrays of ACE objects) from and to the portable
//! [`crate::model`] types.

use crate::error::AclError;
use crate::model::{
    AccessControlEntry, AccessMask, AceFlag, AceType, AuditFlag, bits_to_mask, mask_to_bits,
};
use crate::win32::sid::{OwnedSid, resolve_principal, sid_to_string};
use core::ffi::c_void;
use core::mem::size_of;
use windows::Win32::Security::{
    ACCESS_ALLOWED_ACE, ACCESS_DENIED_ACE, ACE_FLAGS, ACL, ACL_REVISION, ACL_SIZE_INFORMATION,
    AclSizeInformation, AddAccessAllowedAceEx, AddAccessDeniedAceEx, AddAuditAccessAceEx,
    CONTAINER_INHERIT_ACE, GetAce, GetAclInformation, INHERIT_ONLY_ACE, INHERITED_ACE,
    InitializeAcl, NO_PROPAGATE_INHERIT_ACE, OBJECT_INHERIT_ACE, SYSTEM_AUDIT_ACE,
};

// These ACE type identifiers live in `windows::Win32::System::SystemServices` upstream, which
// would require pulling in an additional `windows` crate feature just for three well-known,
// stable constants; they are reproduced here instead.
const ACCESS_ALLOWED_ACE_TYPE: u8 = 0;
const ACCESS_DENIED_ACE_TYPE: u8 = 1;
const SYSTEM_AUDIT_ACE_TYPE: u8 = 2;

/// An owned, ready-to-attach ACL buffer built from a list of access control entries.
pub struct AclBuffer {
    bytes: Vec<u8>,
}

impl AclBuffer {
    /// Returns a pointer to the built ACL, valid for as long as this buffer is alive.
    pub fn as_ptr(&mut self) -> *mut ACL {
        self.bytes.as_mut_ptr().cast::<ACL>()
    }

    /// Builds an ACL from a list of access control entries.
    ///
    /// # Errors
    ///
    /// Returns [`AclError`] when a principal cannot be resolved, or when a lower-level Win32 API
    /// call fails.
    pub fn build<P: AccessMask>(entries: &[AccessControlEntry<P>]) -> Result<Self, AclError> {
        let mut resolved: Vec<(&AccessControlEntry<P>, OwnedSid)> =
            Vec::with_capacity(entries.len());
        // ACE layout for every ACE type this crate emits is header (4 bytes) + mask (4 bytes) +
        // SID bytes, which are always DWORD-aligned, so no additional padding is required.
        let mut total_size = size_of::<ACL>();
        for entry in entries {
            let sid = resolve_principal(&entry.principal)?;
            total_size += 8 + sid_byte_len(&sid);
            resolved.push((entry, sid));
        }

        let mut bytes = vec![0_u8; total_size];
        let acl_ptr = bytes.as_mut_ptr().cast::<ACL>();
        // SAFETY: bytes is sized to hold the ACL header plus every ACE that will be added below.
        unsafe { InitializeAcl(acl_ptr, total_size as u32, ACL_REVISION) }.map_err(|error| {
            AclError::Win32 {
                operation: "InitializeAcl",
                code: error.code().0 as u32,
            }
        })?;

        for (entry, sid) in &resolved {
            let ace_flags = ACE_FLAGS(inheritance_flags_to_bits(&entry.flags));
            let mask = mask_to_bits(&entry.access_mask);
            let psid = sid.as_psid();

            let result = match entry.ace_type {
                // SAFETY: acl_ptr was initialized above and has enough reserved space for this
                // ACE (accounted for in total_size); psid is valid for the duration of this call.
                AceType::Allow => unsafe {
                    AddAccessAllowedAceEx(acl_ptr, ACL_REVISION, ace_flags, mask, psid)
                },
                AceType::Deny => unsafe {
                    AddAccessDeniedAceEx(acl_ptr, ACL_REVISION, ace_flags, mask, psid)
                },
                AceType::Audit => {
                    let success = entry.audit_flags.contains(&AuditFlag::Success);
                    let failure = entry.audit_flags.contains(&AuditFlag::Failure);
                    unsafe {
                        AddAuditAccessAceEx(
                            acl_ptr,
                            ACL_REVISION,
                            ace_flags,
                            mask,
                            psid,
                            success,
                            failure,
                        )
                    }
                }
            };
            result.map_err(|error| AclError::Win32 {
                operation: "AddAce",
                code: error.code().0 as u32,
            })?;
        }

        Ok(Self { bytes })
    }
}

fn sid_byte_len(sid: &OwnedSid) -> usize {
    // SAFETY: sid owns a valid SID buffer; GetLengthSid only reads the SID header.
    unsafe { windows::Win32::Security::GetLengthSid(sid.as_psid()) as usize }
}

fn inheritance_flags_to_bits(flags: &[AceFlag]) -> u32 {
    flags.iter().fold(0_u32, |bits, flag| {
        bits | match flag {
            AceFlag::ContainerInherit => CONTAINER_INHERIT_ACE.0,
            AceFlag::ObjectInherit => OBJECT_INHERIT_ACE.0,
            AceFlag::NoPropagateInherit => NO_PROPAGATE_INHERIT_ACE.0,
            AceFlag::InheritOnly => INHERIT_ONLY_ACE.0,
            AceFlag::Inherited => INHERITED_ACE.0,
        }
    })
}

/// Decodes the subset of `AceFlags` bits this crate models back into [`AceFlag`] values.
fn inheritance_flags_from_bits(bits: u32) -> Vec<AceFlag> {
    let mut flags = Vec::new();
    if bits & CONTAINER_INHERIT_ACE.0 != 0 {
        flags.push(AceFlag::ContainerInherit);
    }
    if bits & OBJECT_INHERIT_ACE.0 != 0 {
        flags.push(AceFlag::ObjectInherit);
    }
    if bits & NO_PROPAGATE_INHERIT_ACE.0 != 0 {
        flags.push(AceFlag::NoPropagateInherit);
    }
    if bits & INHERIT_ONLY_ACE.0 != 0 {
        flags.push(AceFlag::InheritOnly);
    }
    if bits & INHERITED_ACE.0 != 0 {
        flags.push(AceFlag::Inherited);
    }
    flags
}

fn audit_flags_from_bits(bits: u32) -> Vec<AuditFlag> {
    const SUCCESSFUL_ACCESS_ACE_FLAG: u32 = 0x40;
    const FAILED_ACCESS_ACE_FLAG: u32 = 0x80;

    let mut flags = Vec::new();
    if bits & SUCCESSFUL_ACCESS_ACE_FLAG != 0 {
        flags.push(AuditFlag::Success);
    }
    if bits & FAILED_ACCESS_ACE_FLAG != 0 {
        flags.push(AuditFlag::Failure);
    }
    flags
}

/// Parses an existing ACL into a list of access control entries.
///
/// # Safety
///
/// `acl_ptr` must point to a single, valid, initialized `ACL` structure (for example one
/// obtained from `GetSecurityDescriptorDacl`/`GetSecurityDescriptorSacl` on a valid security
/// descriptor), valid for the duration of this call.
///
/// # Errors
///
/// Returns [`AclError::UnsupportedAceType`] when the ACL contains an entry type this crate does
/// not model (for example an object-specific or conditional ACE), or [`AclError`] when a
/// lower-level Win32 API call fails.
pub(crate) unsafe fn parse_acl<P: AccessMask>(
    acl_ptr: *mut ACL,
) -> Result<Vec<AccessControlEntry<P>>, AclError> {
    let mut size_info = ACL_SIZE_INFORMATION::default();
    // SAFETY: the caller guarantees acl_ptr is a valid, initialized ACL pointer; size_info is a
    // writable local variable of the correct size for AclSizeInformation.
    unsafe {
        GetAclInformation(
            acl_ptr,
            (&raw mut size_info).cast::<c_void>(),
            size_of::<ACL_SIZE_INFORMATION>() as u32,
            AclSizeInformation,
        )
    }
    .map_err(|error| AclError::Win32 {
        operation: "GetAclInformation",
        code: error.code().0 as u32,
    })?;

    let mut entries = Vec::with_capacity(size_info.AceCount as usize);
    for index in 0..size_info.AceCount {
        let mut ace_ptr: *mut c_void = core::ptr::null_mut();
        // SAFETY: acl_ptr is valid and index is within [0, AceCount).
        unsafe { GetAce(acl_ptr, index, &mut ace_ptr) }.map_err(|error| AclError::Win32 {
            operation: "GetAce",
            code: error.code().0 as u32,
        })?;

        // SAFETY: every ACE begins with an ACE_HEADER; GetAce returned a valid pointer.
        let header = unsafe { &*ace_ptr.cast::<windows::Win32::Security::ACE_HEADER>() };
        let ace_flags_bits = u32::from(header.AceFlags);

        let (mask, sid_ptr, ace_type) = if header.AceType == ACCESS_ALLOWED_ACE_TYPE {
            // SAFETY: header.AceType identifies this ACE as an ACCESS_ALLOWED_ACE.
            let ace = unsafe { &*ace_ptr.cast::<ACCESS_ALLOWED_ACE>() };
            (
                ace.Mask,
                core::ptr::addr_of!(ace.SidStart).cast::<c_void>(),
                AceType::Allow,
            )
        } else if header.AceType == ACCESS_DENIED_ACE_TYPE {
            // SAFETY: header.AceType identifies this ACE as an ACCESS_DENIED_ACE.
            let ace = unsafe { &*ace_ptr.cast::<ACCESS_DENIED_ACE>() };
            (
                ace.Mask,
                core::ptr::addr_of!(ace.SidStart).cast::<c_void>(),
                AceType::Deny,
            )
        } else if header.AceType == SYSTEM_AUDIT_ACE_TYPE {
            // SAFETY: header.AceType identifies this ACE as a SYSTEM_AUDIT_ACE.
            let ace = unsafe { &*ace_ptr.cast::<SYSTEM_AUDIT_ACE>() };
            (
                ace.Mask,
                core::ptr::addr_of!(ace.SidStart).cast::<c_void>(),
                AceType::Audit,
            )
        } else {
            return Err(AclError::UnsupportedAceType(header.AceType));
        };

        // SAFETY: sid_ptr points at the SID embedded in the ACE that GetAce just returned,
        // which is valid for as long as acl_ptr (borrowed by this function's caller) is valid;
        // it is a real, Windows-populated SID, not externally supplied bytes.
        let principal = unsafe { sid_to_string(windows::Win32::Security::PSID(sid_ptr.cast_mut())) }?;
        let flags = inheritance_flags_from_bits(ace_flags_bits);
        let audit_flags = if ace_type == AceType::Audit {
            audit_flags_from_bits(ace_flags_bits)
        } else {
            Vec::new()
        };
        let access_mask = bits_to_mask::<P>(mask)?;

        entries.push(AccessControlEntry {
            principal,
            ace_type,
            access_mask,
            flags,
            audit_flags,
        });
    }

    Ok(entries)
}
