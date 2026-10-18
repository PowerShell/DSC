// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Resolves access control entry principals (SID strings or account/group names) to security
//! identifiers, and converts security identifiers back to their normalized string form.

use crate::error::AclError;
use core::ffi::c_void;
use rust_i18n::t;
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
    if principal.contains('\0') {
        return Err(AclError::PrincipalResolution {
            principal: principal.to_string(),
            reason: t!("principal.nulCharacter").to_string(),
        });
    }
    if looks_like_sid(principal) {
        return parse_sid(principal);
    }
    lookup_account_name(principal)
}

/// Converts a security identifier to its normalized string form (for example
/// `S-1-5-32-544`).
///
/// # Safety
///
/// `sid` must point to a valid, well-formed SID structure (for example one returned by
/// [`resolve_principal`], embedded in an ACE that was read from a genuine Windows ACL, or
/// otherwise populated by a Win32 API), readable for the entire length implied by its own
/// header, for the duration of this call.
///
/// # Errors
///
/// Returns [`AclError::Win32`] when the conversion fails, or [`AclError::InvalidSid`] when the
/// converted string is not valid UTF-16.
pub unsafe fn sid_to_string(sid: PSID) -> Result<String, AclError> {
    let mut pwstr = PWSTR::null();
    // SAFETY: the caller guarantees `sid` is a valid SID pointer for the duration of this call.
    // The returned string is released via LocalFree below.
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

/// Returns whether `value` has the lexical shape of a SID string (for example `S-1-5-32-544`),
/// so that [`resolve_principal`] attempts to parse it as one instead of looking it up as an
/// account or group name. Only strings that actually match the SID grammar -- a case-insensitive
/// `S`, then a hyphen-separated revision number, identifier authority, and zero or more
/// subauthorities, each a decimal number (the identifier authority may alternatively be a
/// `0x`-prefixed hexadecimal value) -- are treated as candidates. This intentionally excludes
/// hyphenated account or group names that merely start with `S` (for example `Sales-Team` or
/// `S-Corp-Admins`), which must go through account lookup instead.
fn looks_like_sid(value: &str) -> bool {
    let Some(rest) = value.strip_prefix(['S', 's']) else {
        return false;
    };

    let mut parts = rest.split('-');
    // Every SID string has a hyphen immediately after the leading `S`; a hyphenated name
    // instead has more letters there (for example "ales-Team" from "Sales-Team"), which fails
    // this check.
    if parts.next() != Some("") {
        return false;
    }

    let is_decimal = |segment: &str| !segment.is_empty() && segment.bytes().all(|b| b.is_ascii_digit());
    // The identifier authority may instead be a `0x`-prefixed hexadecimal value (up to 48 bits).
    let is_authority = |segment: &str| {
        is_decimal(segment)
            || segment
                .strip_prefix("0x")
                .or_else(|| segment.strip_prefix("0X"))
                .is_some_and(|hex| !hex.is_empty() && hex.bytes().all(|b| b.is_ascii_hexdigit()))
    };

    let Some(revision) = parts.next() else {
        return false;
    };
    if !is_decimal(revision) {
        return false;
    }
    let Some(authority) = parts.next() else {
        return false;
    };
    if !is_authority(authority) {
        return false;
    }

    // Zero or more subauthorities; bail out on the first remaining segment that isn't a plain
    // decimal number (for example a trailing hyphenated word) rather than accepting it as a SID.
    parts.all(is_decimal)
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
            reason: t!("principal.notFound").to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_well_known_sid_strings() {
        assert!(looks_like_sid("S-1-1-0"));
        assert!(looks_like_sid("S-1-5-7"));
        assert!(looks_like_sid("S-1-5-32-544"));
        assert!(looks_like_sid("S-1-5-21-3623811015-3361044348-30300820-1013"));
        // Case-insensitive leading `S`.
        assert!(looks_like_sid("s-1-5-32-544"));
        // Hexadecimal identifier authority.
        assert!(looks_like_sid("S-1-0x206-1-2-3"));
    }

    #[test]
    fn rejects_hyphenated_names_that_merely_start_with_s() {
        // Account/group names that happen to start with `S` and contain hyphens must be
        // resolved via account lookup, not parsed as if they were SID strings.
        assert!(!looks_like_sid("Sales-Team"));
        assert!(!looks_like_sid("S-Corp-Admins"));
        assert!(!looks_like_sid("S-ales-Team"));
        assert!(!looks_like_sid("Seattle-Users"));
    }

    #[test]
    fn rejects_malformed_sid_like_strings() {
        assert!(!looks_like_sid(""));
        assert!(!looks_like_sid("S"));
        assert!(!looks_like_sid("S-"));
        assert!(!looks_like_sid("S-1"));
        // Non-numeric revision/authority segments.
        assert!(!looks_like_sid("S-a-5"));
        assert!(!looks_like_sid("S-1-a"));
        // A non-numeric trailing segment is not a valid subauthority.
        assert!(!looks_like_sid("S-1-5-Team"));
    }

    #[test]
    fn rejects_names_that_do_not_start_with_s() {
        assert!(!looks_like_sid("DOMAIN\\User"));
        assert!(!looks_like_sid("Administrators"));
    }

    #[test]
    fn rejects_principal_names_with_embedded_nul() {
        let error = match resolve_principal("Everyone\0DSC-Invalid") {
            Ok(_) => panic!("an embedded NUL must be rejected"),
            Err(error) => error,
        };

        assert!(matches!(error, AclError::PrincipalResolution { .. }));
        assert!(error.to_string().contains("must not contain NUL"));
    }
}
