// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Local Security Authority policy access: enumerate, grant and revoke account rights.
//!
//! Every buffer handed out by an `Lsa*` API is owned by the LSA and must be released with
//! `LsaFreeMemory`; the RAII guards below do that on drop so early returns cannot leak.

use crate::sid::Sid;
use crate::types::SecurityPolicyError;
use rust_i18n::t;
use std::ffi::c_void;
use std::ptr;
use windows::Win32::Foundation::{
    NTSTATUS, STATUS_ACCESS_DENIED, STATUS_NO_MORE_ENTRIES, STATUS_NO_SUCH_PRIVILEGE, STATUS_OBJECT_NAME_NOT_FOUND,
    STATUS_SUCCESS, WIN32_ERROR,
};
use windows::Win32::Security::Authentication::Identity::{
    LSA_ENUMERATION_INFORMATION, LSA_HANDLE, LSA_OBJECT_ATTRIBUTES, LSA_UNICODE_STRING, LsaAddAccountRights, LsaClose,
    LsaEnumerateAccountsWithUserRight, LsaFreeMemory, LsaNtStatusToWinError, LsaOpenPolicy, LsaRemoveAccountRights,
    POLICY_CREATE_ACCOUNT, POLICY_LOOKUP_NAMES, POLICY_VIEW_LOCAL_INFORMATION,
};
use windows::core::PWSTR;

/// Access needed to enumerate account rights and look up names.
pub const POLICY_READ: u32 = POLICY_VIEW_LOCAL_INFORMATION.cast_unsigned() | POLICY_LOOKUP_NAMES.cast_unsigned();
/// Access needed to grant and revoke account rights (adding a right may create an LSA account object).
pub const POLICY_WRITE: u32 = POLICY_READ | POLICY_CREATE_ACCOUNT.cast_unsigned();

/// The Win32 error message for an `NTSTATUS`.
fn nt_error(status: NTSTATUS) -> String {
    // SAFETY: pure conversion with no preconditions.
    let win32 = unsafe { LsaNtStatusToWinError(status) };
    windows::core::Error::from_hresult(WIN32_ERROR(win32).to_hresult()).to_string()
}

/// Map an `NTSTATUS` to a resource error. Access denied means the process is not elevated.
fn check(status: NTSTATUS, context: impl FnOnce(String) -> String) -> Result<(), SecurityPolicyError> {
    if status == STATUS_SUCCESS {
        return Ok(());
    }
    if status == STATUS_ACCESS_DENIED {
        return Err(SecurityPolicyError::ElevationRequired);
    }
    Err(SecurityPolicyError::Policy(context(nt_error(status))))
}

/// An `LSA_UNICODE_STRING` together with the UTF-16 buffer it points into.
pub struct LsaString {
    _buffer: Vec<u16>,
    raw: LSA_UNICODE_STRING,
}

impl LsaString {
    /// Build an LSA string. `Length` counts bytes without the terminator, `MaximumLength` with it.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` when the value does not fit the 16-bit length fields.
    pub fn new(value: &str) -> Result<Self, SecurityPolicyError> {
        let mut buffer: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
        let too_long = || SecurityPolicyError::InvalidInput(t!("lsa.stringTooLong", value = value).to_string());
        let length = u16::try_from((buffer.len() - 1) * 2).map_err(|_| too_long())?;
        let maximum_length = u16::try_from(buffer.len() * 2).map_err(|_| too_long())?;
        let raw = LSA_UNICODE_STRING {
            Length: length,
            MaximumLength: maximum_length,
            Buffer: PWSTR(buffer.as_mut_ptr()),
        };
        // Moving `buffer` moves the Vec header only; the heap pointer `raw.Buffer` holds stays valid.
        Ok(Self { _buffer: buffer, raw })
    }

    pub fn as_raw(&self) -> &LSA_UNICODE_STRING {
        &self.raw
    }
}

/// An array allocated by an `Lsa*` API and released with `LsaFreeMemory`.
struct LsaBuffer<T> {
    ptr: *mut T,
    count: usize,
}

impl<T> LsaBuffer<T> {
    fn as_slice(&self) -> &[T] {
        if self.ptr.is_null() || self.count == 0 {
            &[]
        } else {
            // SAFETY: the LSA allocated `count` contiguous items at `ptr`; they stay valid until
            // `LsaFreeMemory` runs in `drop`.
            unsafe { std::slice::from_raw_parts(self.ptr, self.count) }
        }
    }
}

impl<T> Drop for LsaBuffer<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: `ptr` came from an `Lsa*` API and is released exactly once.
            unsafe {
                let _ = LsaFreeMemory(Some(self.ptr.cast()));
            }
        }
    }
}

/// An open handle to the local security policy, closed on drop.
pub struct LsaPolicy(LSA_HANDLE);

impl LsaPolicy {
    /// Open the local policy with the given access mask (`POLICY_READ` or `POLICY_WRITE`).
    ///
    /// # Errors
    ///
    /// Returns `ElevationRequired` when access is denied, or a policy error.
    pub fn open(access: u32) -> Result<Self, SecurityPolicyError> {
        let attributes = LSA_OBJECT_ATTRIBUTES::default();
        let mut handle = LSA_HANDLE::default();
        // SAFETY: an all-zero attributes block is what the API requires; `None` selects the local
        // system; `handle` is a valid out pointer.
        let status = unsafe { LsaOpenPolicy(None, &attributes, access, &mut handle) };
        check(status, |error| t!("lsa.openPolicyFailed", error = error).to_string())?;
        Ok(Self(handle))
    }

    /// The SIDs of every account that holds `right`. An unheld right yields an empty list.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` for a right the LSA does not know, `ElevationRequired` when the
    /// process is not elevated, or a policy error.
    pub fn holders(&self, right: &str) -> Result<Vec<Sid>, SecurityPolicyError> {
        let name = LsaString::new(right)?;
        let mut buffer: *mut c_void = ptr::null_mut();
        let mut count: u32 = 0;
        // SAFETY: `name` outlives the call; the out pointers are valid.
        let status = unsafe {
            LsaEnumerateAccountsWithUserRight(self.0, Some(ptr::from_ref(name.as_raw())), &mut buffer, &mut count)
        };
        if status == STATUS_NO_MORE_ENTRIES {
            return Ok(Vec::new());
        }
        if status == STATUS_NO_SUCH_PRIVILEGE {
            return Err(SecurityPolicyError::InvalidInput(t!("rights.unknownRight", right = right).to_string()));
        }
        check(status, |error| t!("lsa.enumerateFailed", right = right, error = error).to_string())?;
        let entries = LsaBuffer::<LSA_ENUMERATION_INFORMATION> {
            ptr: buffer.cast(),
            count: count as usize,
        };
        // Copy every SID out before `entries` drops.
        // SAFETY: each entry holds a valid SID that lives as long as `entries`.
        let sids = entries.as_slice().iter().map(|entry| unsafe { Sid::from_psid(entry.Sid) }).collect();
        Ok(sids)
    }

    /// Grant `right` to the account with `sid`.
    ///
    /// # Errors
    ///
    /// Returns `ElevationRequired` or a policy error.
    pub fn add_right(&self, sid: &Sid, right: &str) -> Result<(), SecurityPolicyError> {
        let name = LsaString::new(right)?;
        // SAFETY: `sid` and `name` are valid for the duration of the call.
        let status = unsafe { LsaAddAccountRights(self.0, sid.as_psid(), std::slice::from_ref(name.as_raw())) };
        check(status, |error| t!("lsa.addFailed", right = right, error = error).to_string())
    }

    /// Revoke `right` from the account with `sid`. Revoking from an account that holds no rights
    /// at all is treated as already done.
    ///
    /// # Errors
    ///
    /// Returns `ElevationRequired` or a policy error.
    pub fn remove_right(&self, sid: &Sid, right: &str) -> Result<(), SecurityPolicyError> {
        let name = LsaString::new(right)?;
        // `allrights` is always false: `true` would strip every right from the account.
        // SAFETY: `sid` and `name` are valid for the duration of the call.
        let status =
            unsafe { LsaRemoveAccountRights(self.0, sid.as_psid(), false, Some(std::slice::from_ref(name.as_raw()))) };
        if status == STATUS_OBJECT_NAME_NOT_FOUND {
            return Ok(());
        }
        check(status, |error| t!("lsa.removeFailed", right = right, error = error).to_string())
    }
}

impl Drop for LsaPolicy {
    fn drop(&mut self) {
        // SAFETY: the handle was returned by `LsaOpenPolicy` and is closed exactly once.
        unsafe {
            let _ = LsaClose(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sid::parse_string_sid;
    use dsc_lib_security_context::{SecurityContext, get_security_context};

    #[test]
    fn lsa_strings_count_bytes() {
        let value = LsaString::new("SeServiceLogonRight").unwrap();
        assert_eq!(value.as_raw().Length, 38);
        assert_eq!(value.as_raw().MaximumLength, 40);
        // SAFETY: the buffer is null-terminated and owned by `value`.
        let text = unsafe { value.as_raw().Buffer.to_string() }.unwrap();
        assert_eq!(text, "SeServiceLogonRight");
        let empty = LsaString::new("").unwrap();
        assert_eq!(empty.as_raw().Length, 0);
        assert_eq!(empty.as_raw().MaximumLength, 2);
    }

    #[test]
    fn read_policy_opens_in_any_context() {
        LsaPolicy::open(POLICY_READ).unwrap();
    }

    #[test]
    fn enumeration_follows_the_security_context() {
        let policy = LsaPolicy::open(POLICY_READ).unwrap();
        let holders = policy.holders("SeChangeNotifyPrivilege");
        if get_security_context() == SecurityContext::Admin {
            let holders = holders.unwrap();
            assert!(holders.contains(&parse_string_sid("S-1-1-0").unwrap()), "Everyone should hold SeChangeNotifyPrivilege");
            assert!(matches!(policy.holders("SeNotARight"), Err(SecurityPolicyError::InvalidInput(_))));
        } else {
            assert!(matches!(holders, Err(SecurityPolicyError::ElevationRequired)));
        }
    }
}
