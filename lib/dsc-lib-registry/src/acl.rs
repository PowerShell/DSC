// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::ffi::c_void;

use lib_acl::AccessControlList;
use lib_acl::win32::{
    build_self_relative_security_descriptor, enable_security_privilege, normalize,
    read_access_control_list, read_security_information,
};
use lib_acl::merge_access_control_list;
use windows::Win32::Foundation::{
    ERROR_FILE_NOT_FOUND, ERROR_INSUFFICIENT_BUFFER, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS,
};
use windows::Win32::Security::PSECURITY_DESCRIPTOR;
use windows::Win32::System::Registry::{
    HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_CONFIG, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
    HKEY_USERS, REG_SAM_FLAGS, RegCloseKey, RegGetKeySecurity, RegOpenKeyExW, RegSetKeySecurity,
};
use windows::core::{HSTRING, PCWSTR};

use crate::config::RegistryAccessRight;
use crate::error::RegistryError;
use rust_i18n::t;

/// `READ_CONTROL`: required to read a DACL (but not a SACL) from a security descriptor.
const READ_CONTROL: REG_SAM_FLAGS = REG_SAM_FLAGS(0x0002_0000);
/// `WRITE_DAC`: required to write a DACL to a security descriptor.
const WRITE_DAC: REG_SAM_FLAGS = REG_SAM_FLAGS(0x0004_0000);
/// `ACCESS_SYSTEM_SECURITY`: required, in combination with the `SeSecurityPrivilege` privilege,
/// to read or write a SACL. Not exposed as a named constant by the `windows` crate for registry
/// access masks, so it is reproduced here from its well-known, stable value.
const ACCESS_SYSTEM_SECURITY: REG_SAM_FLAGS = REG_SAM_FLAGS(0x0100_0000);

type RegistryAccessControlList = AccessControlList<RegistryAccessRight>;

struct RegistryKey(HKEY);

impl Drop for RegistryKey {
    fn drop(&mut self) {
        // SAFETY: This handle was returned by RegOpenKeyExW and is owned by this wrapper.
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

/// Validates that `requested` is well-formed (for example that `audit` entries are only used in
/// the system section) without touching the registry.
///
/// # Errors
///
/// Returns [`RegistryError::Acl`] when `requested` is not well-formed.
pub fn validate(requested: &RegistryAccessControlList) -> Result<(), RegistryError> {
    requested.validate().map_err(acl_error)
}

/// Reads the sections of `path`'s access control list that are requested (non-`None`) in
/// `requested`. Reading the system access control list (SACL) requires the current process to
/// hold the `SeSecurityPrivilege` privilege, which this function enables as needed.
///
/// # Errors
///
/// Returns [`RegistryError::RegistryKeyNotFound`] when `path` does not exist, or
/// [`RegistryError::Acl`] when a lower-level Win32 API call fails.
pub fn get(
    path: &str,
    requested: &RegistryAccessControlList,
) -> Result<RegistryAccessControlList, RegistryError> {
    let want_discretionary = requested.discretionary.is_some();
    let want_system = requested.system.is_some();
    if !want_discretionary && !want_system {
        return Err(RegistryError::Acl(t!("error.invalidAcl").to_string()));
    }

    if want_system {
        enable_security_privilege().map_err(acl_error)?;
    }

    let key = open(path, read_access(requested))?;
    let security_information = read_security_information(want_discretionary, want_system);

    let mut size = 0;
    // SAFETY: The first call intentionally supplies no buffer to obtain the required size.
    let status = unsafe { RegGetKeySecurity(key.0, security_information, None, &mut size) };
    if status != ERROR_INSUFFICIENT_BUFFER {
        return Err(win32_error("RegGetKeySecurity", status.0));
    }

    let mut descriptor_bytes = vec![0_u8; size as usize];
    let descriptor = PSECURITY_DESCRIPTOR(descriptor_bytes.as_mut_ptr().cast::<c_void>());
    // SAFETY: descriptor_bytes is writable and has the size requested by RegGetKeySecurity above.
    let status =
        unsafe { RegGetKeySecurity(key.0, security_information, Some(descriptor), &mut size) };
    if status != ERROR_SUCCESS {
        return Err(win32_error("RegGetKeySecurity", status.0));
    }

    read_access_control_list(&mut descriptor_bytes, want_discretionary, want_system)
        .map_err(acl_error)
}

/// Returns whether `path` currently exists and could accept `requested` being applied via
/// [`set`].
///
/// # Errors
///
/// Returns [`RegistryError::Acl`] when a lower-level Win32 API call fails for a reason other
/// than the key not existing.
pub fn can_set(path: &str, requested: &RegistryAccessControlList) -> Result<bool, RegistryError> {
    match open(path, write_access(requested)) {
        Ok(_) => Ok(true),
        Err(RegistryError::RegistryKeyNotFound(_)) => Ok(false),
        Err(error) => Err(error),
    }
}

/// Applies the sections of `requested` that are populated (non-`None`) to `path`'s access
/// control list. Writing the system access control list (SACL) requires the current process to
/// hold the `SeSecurityPrivilege` privilege, which this function enables as needed.
///
/// # Errors
///
/// Returns [`RegistryError::RegistryKeyNotFound`] when `path` does not exist, or
/// [`RegistryError::Acl`] when `requested` is not well-formed or a lower-level Win32 API call
/// fails.
pub fn set(path: &str, requested: &RegistryAccessControlList) -> Result<(), RegistryError> {
    requested.validate().map_err(acl_error)?;
    if requested.system.is_some() {
        enable_security_privilege().map_err(acl_error)?;
    }

    let to_write = effective(path, requested)?;
    let (mut descriptor_bytes, security_information) =
        build_self_relative_security_descriptor(&to_write).map_err(acl_error)?;
    let descriptor = PSECURITY_DESCRIPTOR(descriptor_bytes.as_mut_ptr().cast::<c_void>());

    let key = open(path, write_access(requested))?;
    // SAFETY: descriptor_bytes remains alive for the duration of this call.
    let status = unsafe { RegSetKeySecurity(key.0, security_information, descriptor) };
    if status != ERROR_SUCCESS {
        return Err(win32_error("RegSetKeySecurity", status.0));
    }

    Ok(())
}

/// Computes the effective access control list that should be written for `requested`, honoring
/// its `_purge` setting: when `true` (the default), returns `requested` (normalized) as-is, so
/// that [`set`] continues to replace each supplied section outright exactly as before. When
/// `false`, reads `path`'s current state for the section(s) `requested` manages and merges
/// `requested`'s entries into them (deduplicating semantic duplicates and ordering `deny`
/// entries first in the discretionary section).
///
/// # Errors
///
/// Returns [`RegistryError::RegistryKeyNotFound`] when `path` does not exist, or
/// [`RegistryError::Acl`] when a principal cannot be resolved or a lower-level Win32 API call
/// fails.
pub fn effective(
    path: &str,
    requested: &RegistryAccessControlList,
) -> Result<RegistryAccessControlList, RegistryError> {
    let normalized = normalize(requested).map_err(acl_error)?;
    if normalized.purge() {
        return Ok(normalized);
    }

    let current = get(path, &normalized)?;
    Ok(merge_access_control_list(&current, &normalized))
}

fn read_access(requested: &RegistryAccessControlList) -> REG_SAM_FLAGS {
    let mut access = REG_SAM_FLAGS(0);
    if requested.discretionary.is_some() {
        access |= READ_CONTROL;
    }
    if requested.system.is_some() {
        access |= ACCESS_SYSTEM_SECURITY;
    }
    access
}

fn write_access(requested: &RegistryAccessControlList) -> REG_SAM_FLAGS {
    let mut access = REG_SAM_FLAGS(0);
    if requested.discretionary.is_some() {
        access |= WRITE_DAC;
    }
    if requested.system.is_some() {
        access |= ACCESS_SYSTEM_SECURITY;
    }
    access
}

fn acl_error(error: lib_acl::AclError) -> RegistryError {
    RegistryError::Acl(error.to_string())
}

fn open(path: &str, access: REG_SAM_FLAGS) -> Result<RegistryKey, RegistryError> {
    let (root, subkey) = split_path(path)?;
    let subkey = HSTRING::from(subkey);
    let mut key = HKEY::default();

    // SAFETY: root is a predefined HKEY, subkey is a null-terminated HSTRING, and key is writable.
    let status = unsafe { RegOpenKeyExW(root, PCWSTR(subkey.as_ptr()), Some(0), access, &mut key) };
    if status == ERROR_FILE_NOT_FOUND || status == ERROR_PATH_NOT_FOUND {
        return Err(RegistryError::RegistryKeyNotFound(path.to_string()));
    }
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
    RegistryError::Acl(t!("error.win32", operation = operation, code = code).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_acl::{AccessControlEntry, AceType, AuditFlag};

    fn entry(ace_type: AceType) -> AccessControlEntry<RegistryAccessRight> {
        AccessControlEntry {
            principal: "S-1-5-18".to_string(),
            ace_type,
            access_mask: vec![RegistryAccessRight::Read],
            flags: Vec::new(),
            audit_flags: Vec::new(),
        }
    }

    #[test]
    fn read_access_reflects_requested_sections() {
        let requested = RegistryAccessControlList {
            discretionary: Some(vec![entry(AceType::Allow)]),
            ..Default::default()
        };
        assert_eq!(read_access(&requested), READ_CONTROL);
    }

    #[test]
    fn write_access_includes_system_security_for_sacl() {
        let mut audit_entry = entry(AceType::Audit);
        audit_entry.audit_flags = vec![AuditFlag::Success];
        let mut requested = RegistryAccessControlList {
            system: Some(vec![audit_entry]),
            ..Default::default()
        };
        assert_eq!(write_access(&requested), ACCESS_SYSTEM_SECURITY);

        requested.discretionary = Some(vec![entry(AceType::Allow)]);
        assert_eq!(write_access(&requested), WRITE_DAC | ACCESS_SYSTEM_SECURITY);
    }

    #[test]
    fn validate_rejects_allow_entry_in_system_section() {
        let requested = RegistryAccessControlList {
            system: Some(vec![entry(AceType::Allow)]),
            ..Default::default()
        };
        let error = validate(&requested).unwrap_err();
        assert!(matches!(error, RegistryError::Acl(_)));
    }

    #[test]
    fn validate_accepts_well_formed_discretionary_entry() {
        let requested = RegistryAccessControlList {
            discretionary: Some(vec![entry(AceType::Allow)]),
            ..Default::default()
        };
        validate(&requested).unwrap();
    }

    #[test]
    fn effective_with_purge_true_does_not_require_the_key_to_exist() {
        // `_purge: true` (the default) never needs to read the key's current state, so this
        // must succeed even though the key path below does not exist.
        let requested = RegistryAccessControlList {
            discretionary: Some(vec![entry(AceType::Allow)]),
            purge: Some(true),
            ..Default::default()
        };

        let effective_acl = effective(r"HKCU\DoesNotExist\AclPurgeTrue", &requested).unwrap();

        // The well-known SID principal used by `entry` is already in canonical form, so
        // normalizing it is a no-op and the result matches the request exactly.
        assert_eq!(effective_acl.discretionary, requested.discretionary);
    }

    #[test]
    fn effective_merges_requested_entries_with_current_section_without_purge() {
        // `HKCU\Software` always exists and already has a non-empty discretionary ACL; using it
        // only for a read-only merge preview (never via `set`) keeps this test from touching any
        // real permissions.
        let key_path = r"HKCU\Software";
        let current = get(
            key_path,
            &RegistryAccessControlList {
                discretionary: Some(Vec::new()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            !current.discretionary.unwrap_or_default().is_empty(),
            "expected HKCU\\Software to already have discretionary entries"
        );

        let mut new_entry = entry(AceType::Allow);
        new_entry.principal = "S-1-5-19".to_string(); // well-known LOCAL_SERVICE SID
        let requested = RegistryAccessControlList {
            discretionary: Some(vec![new_entry]),
            purge: Some(false),
            ..Default::default()
        };

        let effective_acl = effective(key_path, &requested).unwrap();
        let merged = effective_acl.discretionary.unwrap();
        let principals: Vec<&str> = merged.iter().map(|entry| entry.principal.as_str()).collect();

        assert!(principals.contains(&"S-1-5-19"));
        // The current, pre-existing entries are still present alongside the newly requested one.
        assert!(merged.len() > 1);
    }
}
