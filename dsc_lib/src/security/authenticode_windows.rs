// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::dscerror::DscError;
use crate::security::{get_file_trust_level, is_file_checked, TrustLevel};
use std::{
    ffi::OsStr,
    mem::size_of,
    path::Path,
    ptr::{from_ref, null_mut},
    os::windows::ffi::OsStrExt,
};
use windows::{
    core::{PCWSTR, PWSTR, GUID},
    Win32::{
        Foundation::{
            HANDLE,
            HWND,
            TRUST_E_NOSIGNATURE,
            TRUST_E_EXPLICIT_DISTRUST,
            TRUST_E_SUBJECT_NOT_TRUSTED,
            CRYPT_E_SECURITY_SETTINGS,
        },
        Security::WinTrust::{
            WINTRUST_FILE_INFO, WINTRUST_DATA,
            WINTRUST_DATA_0, WINTRUST_DATA_UICONTEXT,
            WINTRUST_ACTION_GENERIC_VERIFY_V2, WTD_STATEACTION_CLOSE,
            WTD_UI_NONE, WTD_REVOKE_NONE, WTD_CHOICE_FILE,
            WTD_STATEACTION_VERIFY, WTD_SAFER_FLAG, WTD_CACHE_ONLY_URL_RETRIEVAL,
            WinVerifyTrustEx,
        }
    }
};
use windows_result::HRESULT;

/// Check the Authenticode signature of a file.
///
/// # Arguments
///
/// * `file_path` - The path to the file to check.
///
/// # Returns
///
/// * `Ok(())` if the file is signed and the signature is valid.
/// * `Err(DscError)` if the file is not signed or the signature is invalid
///
pub fn check_authenticode(file_path: &Path) -> Result<TrustLevel, DscError> {
    if is_file_checked(file_path) {
        return Ok(get_file_trust_level(file_path));
    }

    let wintrust_file_info = WINTRUST_FILE_INFO {
        cbStruct: u32::try_from(size_of::<WINTRUST_FILE_INFO>())?,
        pcwszFilePath: PCWSTR(OsStr::new(file_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>()
            .as_ptr()),
        hFile: HANDLE(null_mut()),
        pgKnownSubject: null_mut(),
    };

    let wintrust_data_0 = WINTRUST_DATA_0 {
        pFile: (&raw const wintrust_file_info).cast_mut(),
    };

    let mut wintrust_data = WINTRUST_DATA {
        cbStruct: u32::try_from(size_of::<WINTRUST_DATA>())?,
        pPolicyCallbackData: null_mut(),
        pSIPClientData: null_mut(),
        dwUIChoice: WTD_UI_NONE,
        fdwRevocationChecks: WTD_REVOKE_NONE,
        dwUnionChoice: WTD_CHOICE_FILE,
        dwStateAction: WTD_STATEACTION_VERIFY,
        hWVTStateData: HANDLE(null_mut()),
        pwszURLReference: PWSTR(null_mut()),
        dwProvFlags: WTD_SAFER_FLAG | WTD_CACHE_ONLY_URL_RETRIEVAL,
        dwUIContext: WINTRUST_DATA_UICONTEXT(0),
        pSignatureSettings: null_mut(),
        Anonymous: wintrust_data_0,
    };

    let result = unsafe {
        WinVerifyTrustEx(
            HWND(null_mut()),
            from_ref::<GUID>(&WINTRUST_ACTION_GENERIC_VERIFY_V2).cast_mut(),
            (&raw const wintrust_data).cast_mut(),
        )
    };

    let hresult = HRESULT(result as _);
    wintrust_data.dwStateAction = WTD_STATEACTION_CLOSE;
    let _ = unsafe { WinVerifyTrustEx(
        HWND(null_mut()),
        from_ref::<GUID>(&WINTRUST_ACTION_GENERIC_VERIFY_V2).cast_mut(),
        (&raw const wintrust_data).cast_mut(),
    ) };

    let trust_level = if hresult.is_ok() {
        TrustLevel::Trusted
    } else {
        match hresult {
            TRUST_E_NOSIGNATURE => TrustLevel::Unsigned,
            TRUST_E_EXPLICIT_DISTRUST => TrustLevel::ExplicitlyDistrusted,
            TRUST_E_SUBJECT_NOT_TRUSTED => TrustLevel::Untrusted,
            CRYPT_E_SECURITY_SETTINGS => TrustLevel::NotMeetSecuritySettings,
            _ => TrustLevel::CannotBeVerified,
        }
    };
    Ok(trust_level)
}
