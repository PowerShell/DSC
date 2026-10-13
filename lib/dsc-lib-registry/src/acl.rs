// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::ffi::c_void;

use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, HLOCAL, LocalFree};
use windows::Win32::Security::Authorization::{
    ConvertSecurityDescriptorToStringSecurityDescriptorW,
    ConvertStringSecurityDescriptorToSecurityDescriptorW,
};
use windows::Win32::Security::{
    DACL_SECURITY_INFORMATION, GROUP_SECURITY_INFORMATION, OBJECT_SECURITY_INFORMATION,
    OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SACL_SECURITY_INFORMATION,
};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_CONFIG, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
    HKEY_USERS, KEY_READ, REG_SAM_FLAGS, RegCloseKey, RegGetKeySecurity, RegOpenKeyExW,
    RegSetKeySecurity,
};
use windows::core::{HSTRING, PCWSTR, PWSTR};

use crate::error::RegistryError;
use rust_i18n::t;

const SDDL_REVISION_1: u32 = 1;

struct RegistryKey(HKEY);

impl Drop for RegistryKey {
    fn drop(&mut self) {
        // SAFETY: This handle was returned by RegOpenKeyExW and is owned by this wrapper.
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

struct LocalAllocation(HLOCAL);

impl Drop for LocalAllocation {
    fn drop(&mut self) {
        // SAFETY: SDDL conversion APIs allocate this pointer with LocalAlloc.
        unsafe {
            let _ = LocalFree(Some(self.0));
        }
    }
}

pub fn get(path: &str, requested_sddl: &str) -> Result<String, RegistryError> {
    let (security_information, _) = permissions(requested_sddl)?;
    let key = open(path, KEY_READ)?;
    let mut size = 0;

    // SAFETY: The first call intentionally supplies no buffer to obtain the required size.
    let status = unsafe { RegGetKeySecurity(key.0, security_information, None, &mut size) };
    if status != ERROR_INSUFFICIENT_BUFFER {
        return Err(win32_error("RegGetKeySecurity", status.0));
    }

    let mut descriptor = vec![0_u8; size as usize];
    let security_descriptor = PSECURITY_DESCRIPTOR(descriptor.as_mut_ptr().cast::<c_void>());
    // SAFETY: The descriptor buffer is writable and has the size requested by RegGetKeySecurity.
    let status = unsafe {
        RegGetKeySecurity(
            key.0,
            security_information,
            Some(security_descriptor),
            &mut size,
        )
    };
    if status != ERROR_SUCCESS {
        return Err(win32_error("RegGetKeySecurity", status.0));
    }

    let mut sddl = PWSTR::null();
    // SAFETY: The security descriptor was populated by RegGetKeySecurity. The returned string is
    // released by LocalAllocation.
    unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            security_descriptor,
            SDDL_REVISION_1,
            security_information,
            &mut sddl,
            None,
        )
        .map_err(|error| RegistryError::Acl(error.to_string()))?;
    }
    let allocation = LocalAllocation(HLOCAL(sddl.0.cast::<c_void>()));
    // SAFETY: The conversion API returned a null-terminated UTF-16 string.
    let result = unsafe { sddl.to_string() }.map_err(|error| RegistryError::Acl(error.to_string()));
    drop(allocation);
    result
}

pub fn validate(sddl: &str) -> Result<(), RegistryError> {
    permissions(sddl)?;
    let sddl = HSTRING::from(sddl);
    let mut descriptor = PSECURITY_DESCRIPTOR::default();

    // SAFETY: The input is a valid null-terminated HSTRING. The returned descriptor is released by
    // LocalAllocation.
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(sddl.as_ptr()),
            SDDL_REVISION_1,
            &mut descriptor,
            None,
        )
        .map_err(|error| RegistryError::Acl(error.to_string()))?;
    }
    drop(LocalAllocation(HLOCAL(descriptor.0)));
    Ok(())
}

pub fn set(path: &str, sddl: &str) -> Result<(), RegistryError> {
    let (security_information, access) = permissions(sddl)?;
    let key = open(path, access)?;
    let sddl = HSTRING::from(sddl);
    let mut descriptor = PSECURITY_DESCRIPTOR::default();

    // SAFETY: The input is a valid null-terminated HSTRING. The returned descriptor is released by
    // LocalAllocation.
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(sddl.as_ptr()),
            SDDL_REVISION_1,
            &mut descriptor,
            None,
        )
        .map_err(|error| RegistryError::Acl(error.to_string()))?;
    }
    let allocation = LocalAllocation(HLOCAL(descriptor.0));

    // SAFETY: The descriptor was returned by the SDDL conversion API and remains allocated for the
    // duration of this call.
    let status = unsafe { RegSetKeySecurity(key.0, security_information, descriptor) };
    drop(allocation);
    if status != ERROR_SUCCESS {
        return Err(win32_error("RegSetKeySecurity", status.0));
    }

    Ok(())
}

fn permissions(sddl: &str) -> Result<(OBJECT_SECURITY_INFORMATION, REG_SAM_FLAGS), RegistryError> {
    let mut information = OBJECT_SECURITY_INFORMATION(0);
    let mut access = 0;
    let mut depth = 0_u32;
    let mut chars = sddl.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            'O' if depth == 0 && chars.peek() == Some(&':') => {
                information |= OWNER_SECURITY_INFORMATION;
                access |= 0x0008_0000; // WRITE_OWNER
            }
            'G' if depth == 0 && chars.peek() == Some(&':') => {
                information |= GROUP_SECURITY_INFORMATION;
                access |= 0x0008_0000; // WRITE_OWNER
            }
            'D' if depth == 0 && chars.peek() == Some(&':') => {
                information |= DACL_SECURITY_INFORMATION;
                access |= 0x0004_0000; // WRITE_DAC
            }
            'S' if depth == 0 && chars.peek() == Some(&':') => {
                information |= SACL_SECURITY_INFORMATION;
            }
            _ => {}
        }
    }
    if information.0 & SACL_SECURITY_INFORMATION.0 != 0 {
        return Err(RegistryError::Acl(t!("error.saclUnsupported").to_string()));
    }
    if information == OBJECT_SECURITY_INFORMATION(0) {
        return Err(RegistryError::Acl(t!("error.invalidAcl").to_string()));
    }
    Ok((information, REG_SAM_FLAGS(access)))
}

fn open(
    path: &str,
    access: windows::Win32::System::Registry::REG_SAM_FLAGS,
) -> Result<RegistryKey, RegistryError> {
    let (root, subkey) = split_path(path)?;
    let subkey = HSTRING::from(subkey);
    let mut key = HKEY::default();

    // SAFETY: root is a predefined HKEY, subkey is a null-terminated HSTRING, and key is writable.
    let status = unsafe { RegOpenKeyExW(root, PCWSTR(subkey.as_ptr()), Some(0), access, &mut key) };
    if status != ERROR_SUCCESS {
        return Err(win32_error("RegOpenKeyExW", status.0));
    }

    Ok(RegistryKey(key))
}

fn split_path(path: &str) -> Result<(HKEY, &str), RegistryError> {
    let (hive, subkey) = path
        .split_once('\\')
        .map_or((path, ""), |(hive, subkey)| (hive, subkey));
    let root = match hive {
        "HKCC" | "HKEY_CURRENT_CONFIG" => HKEY_CURRENT_CONFIG,
        "HKCR" | "HKEY_CLASSES_ROOT" => HKEY_CLASSES_ROOT,
        "HKCU" | "HKEY_CURRENT_USER" => HKEY_CURRENT_USER,
        "HKLM" | "HKEY_LOCAL_MACHINE" => HKEY_LOCAL_MACHINE,
        "HKU" | "HKEY_USERS" => HKEY_USERS,
        _ => return Err(RegistryError::InvalidHive(hive.to_string())),
    };
    Ok((root, subkey))
}

fn win32_error(operation: &str, code: u32) -> RegistryError {
    RegistryError::Acl(format!("{operation} failed with error code {code}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permissions_detects_requested_sections() {
        let (information, access) = permissions("O:SYG:BAD:(A;;KA;;;SY)").unwrap();

        assert_eq!(
            information.0,
            OWNER_SECURITY_INFORMATION.0
                | GROUP_SECURITY_INFORMATION.0
                | DACL_SECURITY_INFORMATION.0
        );
        assert_eq!(access.0, 0x000c_0000);
    }

    #[test]
    fn permissions_ignores_component_markers_inside_aces() {
        let (information, access) =
            permissions(r#"D:(XA;;KR;;;WD;(@User.Title == "O:operator"))"#).unwrap();

        assert_eq!(information, DACL_SECURITY_INFORMATION);
        assert_eq!(access.0, 0x0004_0000);
    }

    #[test]
    fn permissions_rejects_sacl() {
        let error = permissions("S:(AU;SA;KA;;;WD)").unwrap_err();

        assert!(matches!(error, RegistryError::Acl(_)));
    }

    #[test]
    fn validate_rejects_invalid_sddl() {
        let error = validate("D:not-valid").unwrap_err();

        assert!(matches!(error, RegistryError::Acl(_)));
    }
}
