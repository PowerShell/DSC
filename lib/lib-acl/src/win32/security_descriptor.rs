// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Builds a self-relative security descriptor byte buffer from the portable [`crate::model`]
//! types, and reads the DACL/SACL and protection state back out of an existing (self-relative)
//! security descriptor buffer such as one returned by `RegGetKeySecurity` or
//! `GetFileSecurityW`.

use crate::error::AclError;
use crate::model::{AccessControlEntry, AccessControlList, AccessMask};
use crate::win32::acl::{AclBuffer, parse_acl};
use crate::win32::sid::{resolve_principal, sid_to_string};
use core::ffi::c_void;
use windows::Win32::Security::{
    DACL_SECURITY_INFORMATION, GetSecurityDescriptorControl, GetSecurityDescriptorDacl,
    GetSecurityDescriptorSacl, InitializeSecurityDescriptor, MakeSelfRelativeSD,
    OBJECT_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION,
    PROTECTED_SACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SACL_SECURITY_INFORMATION,
    SE_DACL_PROTECTED, SE_SACL_PROTECTED, SECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_CONTROL,
    SetSecurityDescriptorControl, SetSecurityDescriptorDacl, SetSecurityDescriptorSacl,
    UNPROTECTED_DACL_SECURITY_INFORMATION, UNPROTECTED_SACL_SECURITY_INFORMATION,
};

// `SECURITY_DESCRIPTOR_REVISION` lives in `windows::Win32::System::SystemServices` upstream,
// which would require pulling in an additional `windows` crate feature just for this one
// well-known, stable constant; it is reproduced here instead.
const SECURITY_DESCRIPTOR_REVISION: u32 = 1;

/// The `SECURITY_INFORMATION` bits identifying which sections a read request wants.
#[must_use]
pub fn read_security_information(
    want_discretionary: bool,
    want_system: bool,
) -> OBJECT_SECURITY_INFORMATION {
    let mut information = OBJECT_SECURITY_INFORMATION(0);
    if want_discretionary {
        information |= DACL_SECURITY_INFORMATION;
    }
    if want_system {
        information |= SACL_SECURITY_INFORMATION;
    }
    information
}

/// Reads the requested sections of an existing, self-relative security descriptor buffer into
/// the portable model.
///
/// # Errors
///
/// Returns [`AclError`] when the descriptor is malformed or contains an unsupported ACE type.
pub fn read_access_control_list<P: AccessMask>(
    descriptor_bytes: &mut [u8],
    want_discretionary: bool,
    want_system: bool,
) -> Result<AccessControlList<P>, AclError> {
    let descriptor = PSECURITY_DESCRIPTOR(descriptor_bytes.as_mut_ptr().cast::<c_void>());
    let mut control = 0_u16;
    let mut revision = 0_u32;
    // SAFETY: descriptor points at a valid, self-relative security descriptor buffer.
    unsafe { GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) }.map_err(
        |error| AclError::Win32 {
            operation: "GetSecurityDescriptorControl",
            code: error.code().0 as u32,
        },
    )?;

    let mut result = AccessControlList {
        // A raw read always reflects the actual, current state of the sections requested; report
        // `purge: true` (the default) for consistency, since there is no partial/merge request to
        // reflect here.
        purge: Some(true),
        ..Default::default()
    };

    if want_discretionary {
        let (entries, protected) = read_acl_section(descriptor, control, false)?;
        result.discretionary = Some(entries);
        result.discretionary_protected = Some(protected);
    }
    if want_system {
        let (entries, protected) = read_acl_section(descriptor, control, true)?;
        result.system = Some(entries);
        result.system_protected = Some(protected);
    }

    Ok(result)
}

fn read_acl_section<P: AccessMask>(
    descriptor: PSECURITY_DESCRIPTOR,
    control: u16,
    sacl: bool,
) -> Result<(Vec<AccessControlEntry<P>>, bool), AclError> {
    use windows::core::BOOL;
    let mut present = BOOL::default();
    let mut defaulted = BOOL::default();
    let mut acl_ptr = core::ptr::null_mut::<windows::Win32::Security::ACL>();

    // SAFETY: descriptor is a valid security descriptor; the output pointers are writable
    // locals.
    if sacl {
        unsafe { GetSecurityDescriptorSacl(descriptor, &mut present, &mut acl_ptr, &mut defaulted) }
    } else {
        unsafe { GetSecurityDescriptorDacl(descriptor, &mut present, &mut acl_ptr, &mut defaulted) }
    }
    .map_err(|error| AclError::Win32 {
        operation: if sacl {
            "GetSecurityDescriptorSacl"
        } else {
            "GetSecurityDescriptorDacl"
        },
        code: error.code().0 as u32,
    })?;

    let entries = if present.as_bool() && !acl_ptr.is_null() {
        parse_acl::<P>(acl_ptr)?
    } else {
        Vec::new()
    };

    let protected_bit = if sacl {
        SE_SACL_PROTECTED.0
    } else {
        SE_DACL_PROTECTED.0
    };
    let protected = control & protected_bit != 0;

    Ok((entries, protected))
}

/// Builds a self-relative security descriptor buffer for the requested sections of `acl`, along
/// with the `SECURITY_INFORMATION` bits identifying which sections (and protection state) to
/// write. Sections left as `None` in `acl` are omitted entirely so that callers can update the
/// DACL and/or SACL independently.
///
/// # Errors
///
/// Returns [`AclError`] when a principal cannot be resolved or a lower-level Win32 API call
/// fails.
pub fn build_self_relative_security_descriptor<P: AccessMask>(
    acl: &AccessControlList<P>,
) -> Result<(Vec<u8>, OBJECT_SECURITY_INFORMATION), AclError> {
    let mut descriptor = SECURITY_DESCRIPTOR::default();
    let descriptor_ptr = PSECURITY_DESCRIPTOR((&raw mut descriptor).cast::<c_void>());
    // SAFETY: descriptor is a local, writable SECURITY_DESCRIPTOR.
    unsafe { InitializeSecurityDescriptor(descriptor_ptr, SECURITY_DESCRIPTOR_REVISION) }.map_err(
        |error| AclError::Win32 {
            operation: "InitializeSecurityDescriptor",
            code: error.code().0 as u32,
        },
    )?;

    let mut information = OBJECT_SECURITY_INFORMATION(0);
    // Keep the built ACL buffers alive until the descriptor has been converted to self-relative
    // form, since the absolute descriptor holds raw pointers into them.
    let mut dacl_buffer = None;
    let mut sacl_buffer = None;

    if let Some(entries) = &acl.discretionary {
        let mut buffer = AclBuffer::build(entries.as_slice())?;
        // SAFETY: descriptor_ptr is the descriptor initialized above, and buffer.as_ptr() is
        // kept alive for as long as descriptor_ptr is used (stored in dacl_buffer below).
        unsafe {
            SetSecurityDescriptorDacl(
                descriptor_ptr,
                true,
                Some(buffer.as_ptr().cast_const()),
                false,
            )
        }
        .map_err(|error| AclError::Win32 {
            operation: "SetSecurityDescriptorDacl",
            code: error.code().0 as u32,
        })?;
        information |= DACL_SECURITY_INFORMATION;
        information |= if acl.discretionary_protected.unwrap_or(false) {
            PROTECTED_DACL_SECURITY_INFORMATION
        } else {
            UNPROTECTED_DACL_SECURITY_INFORMATION
        };
        dacl_buffer = Some(buffer);
    }

    if let Some(entries) = &acl.system {
        let mut buffer = AclBuffer::build(entries.as_slice())?;
        // SAFETY: see above.
        unsafe {
            SetSecurityDescriptorSacl(
                descriptor_ptr,
                true,
                Some(buffer.as_ptr().cast_const()),
                false,
            )
        }
        .map_err(|error| AclError::Win32 {
            operation: "SetSecurityDescriptorSacl",
            code: error.code().0 as u32,
        })?;
        information |= SACL_SECURITY_INFORMATION;
        information |= if acl.system_protected.unwrap_or(false) {
            PROTECTED_SACL_SECURITY_INFORMATION
        } else {
            UNPROTECTED_SACL_SECURITY_INFORMATION
        };
        sacl_buffer = Some(buffer);
    }

    // Also stamp the protection state directly into the descriptor's own control bits. The
    // `information` bits above are the primary mechanism (honored by, for example,
    // `RegSetKeySecurity`), but some legacy, path-based APIs (`SetFileSecurityW`) are documented
    // as obsolete and, in practice, silently ignore the `PROTECTED_*`/`UNPROTECTED_*` bits in
    // their `SecurityInformation` parameter while still persisting whatever the descriptor's own
    // control word says; setting both makes the protection state apply consistently regardless
    // of which Set*Security API a caller ultimately uses.
    let mut control_mask = SECURITY_DESCRIPTOR_CONTROL(0);
    let mut control_bits = SECURITY_DESCRIPTOR_CONTROL(0);
    if acl.discretionary.is_some() {
        control_mask |= SE_DACL_PROTECTED;
        if acl.discretionary_protected.unwrap_or(false) {
            control_bits |= SE_DACL_PROTECTED;
        }
    }
    if acl.system.is_some() {
        control_mask |= SE_SACL_PROTECTED;
        if acl.system_protected.unwrap_or(false) {
            control_bits |= SE_SACL_PROTECTED;
        }
    }
    if control_mask.0 != 0 {
        // SAFETY: descriptor_ptr is the descriptor initialized above.
        unsafe { SetSecurityDescriptorControl(descriptor_ptr, control_mask, control_bits) }
            .map_err(|error| AclError::Win32 {
                operation: "SetSecurityDescriptorControl",
                code: error.code().0 as u32,
            })?;
    }

    let self_relative = to_self_relative(descriptor_ptr);
    drop(dacl_buffer);
    drop(sacl_buffer);
    Ok((self_relative?, information))
}

fn to_self_relative(descriptor: PSECURITY_DESCRIPTOR) -> Result<Vec<u8>, AclError> {
    let mut size = 0_u32;
    // SAFETY: descriptor is a valid, initialized (absolute-format) security descriptor. The
    // first call intentionally supplies no buffer to obtain the required size, and is expected
    // to fail with ERROR_INSUFFICIENT_BUFFER; the error is inspected only if size was not
    // populated.
    let first =
        unsafe { MakeSelfRelativeSD(descriptor, Some(PSECURITY_DESCRIPTOR::default()), &mut size) };
    if size == 0 {
        let code = first.err().map_or(0, |error| error.code().0 as u32);
        return Err(AclError::Win32 {
            operation: "MakeSelfRelativeSD",
            code,
        });
    }

    let mut buffer = vec![0_u8; size as usize];
    let target = PSECURITY_DESCRIPTOR(buffer.as_mut_ptr().cast::<c_void>());
    // SAFETY: buffer is sized according to the size returned above.
    unsafe { MakeSelfRelativeSD(descriptor, Some(target), &mut size) }.map_err(|error| {
        AclError::Win32 {
            operation: "MakeSelfRelativeSD",
            code: error.code().0 as u32,
        }
    })?;
    Ok(buffer)
}

/// Returns a copy of `acl` with every entry's `principal` normalized to its resolved SID string,
/// so that a caller-supplied access control list (which may name principals by SID or by
/// account/group name) can be compared directly, field-for-field, against the output of
/// [`read_access_control_list`] (which always reports resolved SID strings), and so that
/// entries which name the same principal differently are recognized as duplicates when merging.
///
/// # Errors
///
/// Returns [`AclError`] when a principal cannot be resolved to a SID.
pub fn normalize<P: AccessMask>(acl: &AccessControlList<P>) -> Result<AccessControlList<P>, AclError> {
    Ok(AccessControlList {
        discretionary: acl
            .discretionary
            .as_deref()
            .map(normalize_entries)
            .transpose()?,
        discretionary_protected: acl.discretionary_protected,
        system: acl.system.as_deref().map(normalize_entries).transpose()?,
        system_protected: acl.system_protected,
        purge: acl.purge,
    })
}

fn normalize_entries<P: AccessMask>(
    entries: &[AccessControlEntry<P>],
) -> Result<Vec<AccessControlEntry<P>>, AclError> {
    entries
        .iter()
        .map(|entry| {
            let sid = resolve_principal(&entry.principal)?;
            let principal = sid_to_string(sid.as_psid())?;
            Ok(AccessControlEntry {
                principal,
                ..entry.clone()
            })
        })
        .collect()
}
