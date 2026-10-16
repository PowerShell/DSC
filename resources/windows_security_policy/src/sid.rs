// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Security identifiers: owned copies, string conversion and account name resolution.
//!
//! Everything here uses `advapi32` lookups that work without elevation.

use crate::types::SecurityPolicyError;
use rust_i18n::t;
use std::collections::HashSet;
use std::ffi::c_void;
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_NONE_MAPPED, HLOCAL, LocalFree, WIN32_ERROR};
use windows::Win32::Security::Authorization::{ConvertSidToStringSidW, ConvertStringSidToSidW};
use windows::Win32::Security::{GetLengthSid, LookupAccountNameW, LookupAccountSidW, PSID, SID_NAME_USE};
use windows::core::{PCWSTR, PWSTR};

/// An owned copy of a security identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sid(Vec<u8>);

impl Sid {
    /// Copy the SID that `psid` points to.
    ///
    /// # Safety
    ///
    /// `psid` must point to a valid SID returned by a Win32 API.
    pub(crate) unsafe fn from_psid(psid: PSID) -> Self {
        // SAFETY: the caller guarantees a valid SID; `GetLengthSid` reads its header to size the copy.
        let length = unsafe { GetLengthSid(psid) } as usize;
        // SAFETY: a valid SID occupies exactly `length` readable bytes.
        let bytes = unsafe { std::slice::from_raw_parts(psid.0.cast::<u8>(), length) };
        Self(bytes.to_vec())
    }

    /// A `PSID` view of this SID. The APIs this crate calls never write through it.
    pub(crate) fn as_psid(&self) -> PSID {
        PSID(self.0.as_ptr().cast_mut().cast())
    }

    #[cfg(test)]
    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

/// An account resolved from user input or from the LSA.
#[derive(Debug, Clone)]
pub struct ResolvedIdentity {
    pub sid: Sid,
    /// Canonical `DOMAIN\Name`, bare `Name` when the domain is empty, or the SID string when
    /// the SID does not resolve to an account.
    pub name: String,
}

/// Memory allocated with `LocalAlloc` by the `Convert*Sid*` APIs, released on drop.
struct LocalBuffer(*mut c_void);

impl Drop for LocalBuffer {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: the pointer came from an API documented to require `LocalFree` and is freed once.
            unsafe {
                let _ = LocalFree(Some(HLOCAL(self.0)));
            }
        }
    }
}

/// Encode a Rust string as a null-terminated UTF-16 buffer.
fn to_wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn is_win32(error: &windows::core::Error, code: WIN32_ERROR) -> bool {
    error.code() == code.to_hresult()
}

/// Parse an `S-1-...` string into a SID.
///
/// # Errors
///
/// Returns the Win32 error when the string is not a valid SID.
pub fn parse_string_sid(text: &str) -> Result<Sid, windows::core::Error> {
    let wide = to_wide(text);
    let mut psid = PSID::default();
    // SAFETY: `wide` is null-terminated and outlives the call; `psid` receives a `LocalAlloc`'d SID.
    unsafe { ConvertStringSidToSidW(PCWSTR(wide.as_ptr()), &mut psid) }?;
    let _buffer = LocalBuffer(psid.0);
    // SAFETY: the API returned a valid SID that stays alive until `_buffer` drops.
    Ok(unsafe { Sid::from_psid(psid) })
}

/// Format a SID as `S-1-...`.
///
/// # Errors
///
/// Returns the Win32 error when the conversion fails.
pub fn sid_to_string(sid: &Sid) -> Result<String, windows::core::Error> {
    let mut text = PWSTR::null();
    // SAFETY: `sid` is a valid SID and `text` receives a `LocalAlloc`'d null-terminated string.
    unsafe { ConvertSidToStringSidW(sid.as_psid(), &mut text) }?;
    let _buffer = LocalBuffer(text.as_ptr().cast());
    // SAFETY: the string is null-terminated and stays alive until `_buffer` drops.
    Ok(unsafe { String::from_utf16_lossy(text.as_wide()) })
}

/// Resolve an account name against the local machine (and its domain, when joined).
///
/// # Errors
///
/// Returns `ERROR_NONE_MAPPED` when the account does not exist, or another Win32 error.
pub fn lookup_account_name(name: &str) -> Result<Sid, windows::core::Error> {
    let wide = to_wide(name);
    let mut sid_length: u32 = 0;
    let mut domain_length: u32 = 0;
    let mut sid_type = SID_NAME_USE::default();
    // The first call sizes the buffers and is expected to fail with ERROR_INSUFFICIENT_BUFFER.
    // SAFETY: `wide` is null-terminated; the out pointers are valid; null buffers are allowed.
    let sizing = unsafe {
        LookupAccountNameW(PCWSTR::null(), PCWSTR(wide.as_ptr()), None, &mut sid_length, None, &mut domain_length, &mut sid_type)
    };
    if let Err(error) = sizing
        && !is_win32(&error, ERROR_INSUFFICIENT_BUFFER)
    {
        return Err(error);
    }
    let mut sid = vec![0u8; sid_length as usize];
    let mut domain = vec![0u16; domain_length as usize];
    // SAFETY: the buffers have the sizes the first call asked for.
    unsafe {
        LookupAccountNameW(
            PCWSTR::null(),
            PCWSTR(wide.as_ptr()),
            Some(PSID(sid.as_mut_ptr().cast())),
            &mut sid_length,
            Some(PWSTR(domain.as_mut_ptr())),
            &mut domain_length,
            &mut sid_type,
        )
    }?;
    sid.truncate(sid_length as usize);
    Ok(Sid(sid))
}

/// Resolve a SID to `(domain, name)`.
///
/// # Errors
///
/// Returns `ERROR_NONE_MAPPED` when no account has the SID, or another Win32 error.
pub fn lookup_account_sid(sid: &Sid) -> Result<(String, String), windows::core::Error> {
    let mut name_length: u32 = 0;
    let mut domain_length: u32 = 0;
    let mut sid_type = SID_NAME_USE::default();
    // SAFETY: `sid` is valid; the out pointers are valid; null buffers are allowed on the sizing call.
    let sizing = unsafe { LookupAccountSidW(PCWSTR::null(), sid.as_psid(), None, &mut name_length, None, &mut domain_length, &mut sid_type) };
    if let Err(error) = sizing
        && !is_win32(&error, ERROR_INSUFFICIENT_BUFFER)
    {
        return Err(error);
    }
    let mut name = vec![0u16; name_length as usize];
    let mut domain = vec![0u16; domain_length as usize];
    // SAFETY: the buffers have the sizes the first call asked for.
    unsafe {
        LookupAccountSidW(
            PCWSTR::null(),
            sid.as_psid(),
            Some(PWSTR(name.as_mut_ptr())),
            &mut name_length,
            Some(PWSTR(domain.as_mut_ptr())),
            &mut domain_length,
            &mut sid_type,
        )
    }?;
    // After the second call the lengths exclude the terminator.
    Ok((
        String::from_utf16_lossy(&domain[..domain_length as usize]),
        String::from_utf16_lossy(&name[..name_length as usize]),
    ))
}

/// The canonical display form of a SID: `DOMAIN\Name`, bare `Name` when the domain is empty
/// (e.g. `Everyone`), or the SID string when it does not resolve.
///
/// # Errors
///
/// Returns an error when the lookup fails for a reason other than an unknown SID.
pub fn display_name(sid: &Sid) -> Result<String, SecurityPolicyError> {
    match lookup_account_sid(sid) {
        Ok((domain, name)) if domain.is_empty() => Ok(name),
        Ok((domain, name)) => Ok(format!("{domain}\\{name}")),
        Err(error) if is_win32(&error, ERROR_NONE_MAPPED) => sid_to_string(sid).map_err(sid_to_string_error),
        Err(error) => Err(SecurityPolicyError::Policy(t!("sid.lookupSidFailed", error = error.to_string()).to_string())),
    }
}

fn sid_to_string_error(error: windows::core::Error) -> SecurityPolicyError {
    SecurityPolicyError::Policy(t!("sid.sidToStringFailed", error = error.to_string()).to_string())
}

/// Describe a SID obtained from the LSA.
///
/// # Errors
///
/// Returns an error when the SID cannot be converted to a string or looked up.
pub fn describe(sid: Sid) -> Result<ResolvedIdentity, SecurityPolicyError> {
    let name = display_name(&sid)?;
    Ok(ResolvedIdentity { sid, name })
}

/// Whether `text` has the shape of a SID string (`S-1-<digit>...`).
fn looks_like_sid(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() >= 5 && bytes[0].eq_ignore_ascii_case(&b'S') && &bytes[1..4] == b"-1-" && bytes[4].is_ascii_digit()
}

/// Resolve one `identity` entry: a name, a SID string, or the secedit `*S-1-...` form.
///
/// # Errors
///
/// Returns `InvalidInput` for an empty entry, a malformed SID string or an unknown account name.
pub fn resolve_identity(input: &str) -> Result<ResolvedIdentity, SecurityPolicyError> {
    let trimmed = input.trim();
    let text = trimmed.strip_prefix('*').unwrap_or(trimmed).trim();
    if text.is_empty() {
        return Err(SecurityPolicyError::InvalidInput(t!("sid.emptyIdentity").to_string()));
    }
    let sid = if looks_like_sid(text) {
        // A well-formed SID with no matching account is accepted: the LSA keeps rights for
        // orphaned SIDs and `display_name` reports them as SID strings.
        parse_string_sid(text).map_err(|_| SecurityPolicyError::InvalidInput(t!("sid.invalidSid", identity = input).to_string()))?
    } else {
        lookup_account_name(text).map_err(|error| {
            if is_win32(&error, ERROR_NONE_MAPPED) {
                SecurityPolicyError::InvalidInput(t!("sid.unknownAccount", identity = input).to_string())
            } else {
                SecurityPolicyError::Policy(t!("sid.lookupNameFailed", identity = input, error = error.to_string()).to_string())
            }
        })?
    };
    describe(sid)
}

/// Resolve every entry, dropping duplicates that resolve to the same SID (first occurrence wins).
///
/// # Errors
///
/// Returns the first resolution error.
pub fn resolve_identities(inputs: &[String]) -> Result<Vec<ResolvedIdentity>, SecurityPolicyError> {
    let mut seen = HashSet::new();
    let mut identities = Vec::with_capacity(inputs.len());
    for input in inputs {
        let identity = resolve_identity(input)?;
        if seen.insert(identity.sid.clone()) {
            identities.push(identity);
        }
    }
    Ok(identities)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADMINISTRATORS: &str = "S-1-5-32-544";

    #[test]
    fn sid_strings_round_trip() {
        let sid = parse_string_sid(ADMINISTRATORS).unwrap();
        assert_eq!(sid.0.len(), 16, "S-1-5-32-544 is 8 header bytes plus two sub-authorities");
        assert_eq!(sid_to_string(&sid).unwrap(), ADMINISTRATORS);
        assert!(parse_string_sid("S-1-X").is_err());
        assert!(parse_string_sid("Administrators").is_err());
    }

    #[test]
    fn names_resolve_to_the_expected_sids() {
        let administrators = parse_string_sid(ADMINISTRATORS).unwrap();
        assert_eq!(lookup_account_name("BUILTIN\\Administrators").unwrap(), administrators);
        assert_eq!(lookup_account_name("Administrators").unwrap(), administrators);
        assert_eq!(lookup_account_name("Everyone").unwrap(), parse_string_sid("S-1-1-0").unwrap());
        assert_eq!(lookup_account_name("NT AUTHORITY\\SYSTEM").unwrap(), parse_string_sid("S-1-5-18").unwrap());
        let missing = lookup_account_name("NoSuchAccount_zz_12345").unwrap_err();
        assert!(is_win32(&missing, ERROR_NONE_MAPPED));
    }

    #[test]
    fn display_names_are_canonical() {
        assert_eq!(display_name(&parse_string_sid(ADMINISTRATORS).unwrap()).unwrap(), "BUILTIN\\Administrators");
        assert_eq!(display_name(&parse_string_sid("S-1-1-0").unwrap()).unwrap(), "Everyone");
        assert_eq!(display_name(&parse_string_sid("S-1-5-18").unwrap()).unwrap(), "NT AUTHORITY\\SYSTEM");
        let orphan = "S-1-5-21-1-2-3-4567";
        assert_eq!(display_name(&parse_string_sid(orphan).unwrap()).unwrap(), orphan);
    }

    #[test]
    fn identities_resolve_from_every_accepted_form() {
        for input in ["Administrators", "BUILTIN\\Administrators", "builtin\\administrators", ADMINISTRATORS, "*S-1-5-32-544", "  s-1-5-32-544 "] {
            let resolved = resolve_identity(input).unwrap_or_else(|e| panic!("{input}: {e}"));
            assert_eq!(resolved.name, "BUILTIN\\Administrators", "{input}");
            assert_eq!(sid_to_string(&resolved.sid).unwrap(), ADMINISTRATORS, "{input}");
        }
        let orphan = resolve_identity("S-1-5-21-1-2-3-4567").unwrap();
        assert_eq!(orphan.name, "S-1-5-21-1-2-3-4567");
    }

    #[test]
    fn invalid_identities_are_invalid_input() {
        for input in ["", "   ", "*", "S-1-X", "NoSuchAccount_zz_12345"] {
            assert!(
                matches!(resolve_identity(input), Err(SecurityPolicyError::InvalidInput(_))),
                "{input:?} should be invalid input"
            );
        }
    }

    #[test]
    fn duplicate_identities_collapse_by_sid() {
        let inputs: Vec<String> = ["Administrators", "BUILTIN\\Administrators", ADMINISTRATORS, "Everyone"]
            .iter()
            .map(ToString::to_string)
            .collect();
        let resolved = resolve_identities(&inputs).unwrap();
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].name, "BUILTIN\\Administrators");
        assert_eq!(resolved[1].name, "Everyone");
    }

    #[test]
    fn sid_shape_detection() {
        assert!(looks_like_sid("S-1-5-32-544"));
        assert!(looks_like_sid("s-1-1-0"));
        assert!(!looks_like_sid("S-1-"));
        assert!(!looks_like_sid("S-2-5-32-544"));
        assert!(!looks_like_sid("Administrators"));
        assert!(!looks_like_sid("S-1-X"));
    }
}
