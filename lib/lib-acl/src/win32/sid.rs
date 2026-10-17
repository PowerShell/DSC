// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Resolves access control entry principals (SID strings or account/group names) to security
//! identifiers, and converts security identifiers back to their normalized string form.

use crate::error::AclError;
use core::ffi::c_void;
use windows::Win32::Foundation::{HLOCAL, LocalFree};
use windows::Win32::Security::Authorization::{ConvertSidToStringSidW, ConvertStringSidToSidW};
use windows::Win32::Security::{GetLengthSid, LookupAccountNameW, PSID, SID_NAME_USE};
use windows::core::{HSTRING, PCWSTR, PWSTR};

/// Owned bytes for a security identifier (SID). Safe to pass as a [`PSID`] for the lifetime of
/// this value.
pub struct OwnedSid(Vec<u8>);

impl OwnedSid {
    /// Returns a [`PSID`] pointing at this SID's bytes.
    #[must_use]
    pub fn as_psid(&self) -> PSID {
        PSID(self.0.as_ptr().cast_mut().cast::<c_void>())
    }
}

/// Resolves a principal string to a security identifier. Accepts either a SID string (for
/// example `S-1-5-32-544`) or an account/group name (for example `DOMAIN\User` or
/// `Administrators`), which is resolved via [`LookupAccountNameW`].
///
/// # Errors
///
/// Returns [`AclError::InvalidSid`] when `principal` looks like a SID string but is not valid,
/// or [`AclError::PrincipalResolution`] when the account/group name cannot be resolved.
pub fn resolve_principal(principal: &str) -> Result<OwnedSid, AclError> {
    if looks_like_sid(principal) {
        return parse_sid(principal);
    }
    lookup_account_name(principal)
}

/// Converts a security identifier to its normalized string form (for example
/// `S-1-5-32-544`).
///
/// # Errors
///
/// Returns [`AclError::Win32`] when the conversion fails, or [`AclError::InvalidSid`] when the
/// converted string is not valid UTF-16.
pub fn sid_to_string(sid: PSID) -> Result<String, AclError> {
    let mut pwstr = PWSTR::null();
    // SAFETY: sid is a valid SID pointer for the duration of this call. The returned string is
    // released via LocalFree below.
    unsafe { ConvertSidToStringSidW(sid, &mut pwstr) }.map_err(|error| AclError::Win32 {
        operation: "ConvertSidToStringSidW",
        code: error.code().0 as u32,
    })?;

    // SAFETY: pwstr was populated by ConvertSidToStringSidW and is a null-terminated UTF-16
    // string allocated with LocalAlloc.
    let result =
        unsafe { pwstr.to_string() }.map_err(|error| AclError::InvalidSid(error.to_string()));
    // SAFETY: pwstr was allocated by ConvertSidToStringSidW with LocalAlloc.
    unsafe {
        let _ = LocalFree(Some(HLOCAL(pwstr.0.cast())));
    }
    result
}

fn looks_like_sid(value: &str) -> bool {
    value.len() > 1 && value.as_bytes()[0].eq_ignore_ascii_case(&b'S') && value.contains('-')
}

fn parse_sid(value: &str) -> Result<OwnedSid, AclError> {
    let wide = HSTRING::from(value);
    let mut psid = PSID::default();
    // SAFETY: wide is a valid null-terminated string for the duration of this call. psid is
    // allocated by this API with LocalAlloc and copied into an owned buffer below.
    unsafe { ConvertStringSidToSidW(PCWSTR(wide.as_ptr()), &mut psid) }
        .map_err(|_| AclError::InvalidSid(value.to_string()))?;

    // SAFETY: psid was populated by ConvertStringSidToSidW and is valid until freed below.
    let length = unsafe { GetLengthSid(psid) } as usize;
    let mut bytes = vec![0_u8; length];
    // SAFETY: psid points to at least `length` valid bytes, and bytes has the same length.
    unsafe {
        core::ptr::copy_nonoverlapping(psid.0.cast::<u8>(), bytes.as_mut_ptr(), length);
        let _ = LocalFree(Some(HLOCAL(psid.0)));
    }
    Ok(OwnedSid(bytes))
}

fn lookup_account_name(name: &str) -> Result<OwnedSid, AclError> {
    let wide = HSTRING::from(name);
    let mut sid_size = 0_u32;
    let mut domain_size = 0_u32;
    let mut use_: SID_NAME_USE = SID_NAME_USE(0);

    // SAFETY: This first call intentionally supplies no SID/domain buffer to obtain the
    // required sizes.
    let _ = unsafe {
        LookupAccountNameW(
            PCWSTR::null(),
            PCWSTR(wide.as_ptr()),
            None,
            &mut sid_size,
            Some(PWSTR::null()),
            &mut domain_size,
            &mut use_,
        )
    };
    if sid_size == 0 {
        return Err(AclError::PrincipalResolution {
            principal: name.to_string(),
            reason: "the account or group name could not be found".to_string(),
        });
    }

    let mut sid_buffer = vec![0_u8; sid_size as usize];
    let mut domain_buffer = vec![0_u16; domain_size as usize];
    let psid = PSID(sid_buffer.as_mut_ptr().cast::<c_void>());
    // SAFETY: sid_buffer and domain_buffer are sized according to the sizes returned above and
    // are writable for the duration of this call.
    unsafe {
        LookupAccountNameW(
            PCWSTR::null(),
            PCWSTR(wide.as_ptr()),
            Some(psid),
            &mut sid_size,
            Some(PWSTR(domain_buffer.as_mut_ptr())),
            &mut domain_size,
            &mut use_,
        )
    }
    .map_err(|error| AclError::PrincipalResolution {
        principal: name.to_string(),
        reason: error.to_string(),
    })?;

    Ok(OwnedSid(sid_buffer))
}
