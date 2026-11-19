// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Enables the `SeSecurityPrivilege` privilege ("Manage auditing and security log") on the
//! current process's token, which is required before reading or writing a system access control
//! list (SACL).

use crate::error::AclError;
use windows::Win32::Foundation::{CloseHandle, ERROR_NOT_ALL_ASSIGNED, GetLastError, HANDLE, LUID};
use windows::Win32::Security::{
    AdjustTokenPrivileges, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED,
    TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::{HSTRING, PCWSTR};

/// The name of the privilege required to read or write a system access control list (SACL).
pub const SE_SECURITY_NAME: &str = "SeSecurityPrivilege";

/// Enables [`SE_SECURITY_NAME`] on the current process's token.
///
/// # Errors
///
/// Returns [`AclError::PrivilegeNotHeld`] when the current process's token does not hold the
/// privilege (for example because the account lacks the "Manage auditing and security log" user
/// right), or [`AclError::Win32`] when a lower-level API call fails.
pub fn enable_security_privilege() -> Result<(), AclError> {
    // SAFETY: GetCurrentProcess returns a pseudo-handle that does not need to be closed.
    let process = unsafe { GetCurrentProcess() };
    let mut token = HANDLE::default();
    // SAFETY: process is a valid pseudo-handle; token is a writable local variable that
    // receives ownership of the returned handle, closed below.
    unsafe { OpenProcessToken(process, TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token) }
        .map_err(|error| AclError::Win32 {
            operation: "OpenProcessToken",
            code: error.code().0 as u32,
        })?;

    let result = enable_privilege_on_token(token, SE_SECURITY_NAME);
    // SAFETY: token was opened above and is owned by this function.
    unsafe {
        let _ = CloseHandle(token);
    }
    result
}

fn enable_privilege_on_token(token: HANDLE, name: &'static str) -> Result<(), AclError> {
    let wide = HSTRING::from(name);
    let mut luid = LUID::default();
    // SAFETY: wide is a valid null-terminated string for the duration of this call.
    unsafe { LookupPrivilegeValueW(PCWSTR::null(), PCWSTR(wide.as_ptr()), &mut luid) }.map_err(
        |error| AclError::Win32 {
            operation: "LookupPrivilegeValueW",
            code: error.code().0 as u32,
        },
    )?;

    let privileges = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };

    // SAFETY: token is a valid, open token handle with TOKEN_ADJUST_PRIVILEGES access, and
    // privileges is a validly constructed TOKEN_PRIVILEGES describing a single privilege.
    let adjust_result =
        unsafe { AdjustTokenPrivileges(token, false, Some(&raw const privileges), 0, None, None) };
    // AdjustTokenPrivileges can report success while silently not assigning any privileges; the
    // only reliable signal is the last error code, which must be read immediately after the call.
    // SAFETY: GetLastError reads thread-local state set by the call above.
    let not_all_assigned = unsafe { GetLastError() } == ERROR_NOT_ALL_ASSIGNED;

    adjust_result.map_err(|error| AclError::Win32 {
        operation: "AdjustTokenPrivileges",
        code: error.code().0 as u32,
    })?;
    if not_all_assigned {
        return Err(AclError::PrivilegeNotHeld(name));
    }
    Ok(())
}
