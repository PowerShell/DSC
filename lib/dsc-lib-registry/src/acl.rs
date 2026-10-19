// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::ffi::c_void;

use lib_acl::{AccessControlList, AccessMask, AceFlag};
use lib_acl::win32::{
    build_self_relative_security_descriptor, enable_security_privilege, normalize,
    read_access_control_list, read_security_information,
};
use lib_acl::merge_access_control_list;
use windows::Win32::Foundation::{
    CloseHandle, ERROR_FILE_NOT_FOUND, ERROR_INSUFFICIENT_BUFFER, ERROR_PATH_NOT_FOUND,
    ERROR_SUCCESS, HANDLE,
};
use windows::Win32::Security::{
    CreatePrivateObjectSecurityEx, DestroyPrivateObjectSecurity, GENERIC_MAPPING,
    GetSecurityDescriptorLength, OBJECT_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
    SECURITY_AUTO_INHERIT_FLAGS, SEF_DACL_AUTO_INHERIT, SEF_SACL_AUTO_INHERIT, TOKEN_QUERY,
};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_CONFIG, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
    HKEY_USERS, REG_SAM_FLAGS, RegCloseKey, RegGetKeySecurity, RegOpenKeyExW, RegSetKeySecurity,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
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

pub(crate) struct PreparedAccessControlList {
    effective: RegistryAccessControlList,
    descriptor_bytes: Vec<u8>,
    security_information: OBJECT_SECURITY_INFORMATION,
}

impl PreparedAccessControlList {
    pub(crate) fn effective(&self) -> &RegistryAccessControlList {
        &self.effective
    }
}

struct RegistryKey(HKEY);

impl Drop for RegistryKey {
    fn drop(&mut self) {
        // SAFETY: This handle was returned by RegOpenKeyExW and is owned by this wrapper.
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

struct ProcessToken(HANDLE);

impl ProcessToken {
    fn open() -> Result<Self, RegistryError> {
        let mut token = HANDLE::default();
        // SAFETY: GetCurrentProcess returns a valid pseudo-handle, and token is writable.
        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }
            .map_err(|error| win32_error("OpenProcessToken", error.code().0 as u32))?;
        Ok(Self(token))
    }
}

impl Drop for ProcessToken {
    fn drop(&mut self) {
        // SAFETY: This handle was returned by OpenProcessToken and is owned by this wrapper.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

struct PrivateSecurityDescriptor(PSECURITY_DESCRIPTOR);

impl PrivateSecurityDescriptor {
    fn inherit_from(
        parent: PSECURITY_DESCRIPTOR,
        requested: &RegistryAccessControlList,
        token: HANDLE,
    ) -> Result<Self, RegistryError> {
        let mut descriptor = PSECURITY_DESCRIPTOR::default();
        let mut flags = SECURITY_AUTO_INHERIT_FLAGS(0);
        if requested.discretionary.is_some() {
            flags |= SEF_DACL_AUTO_INHERIT;
        }
        if requested.system.is_some() {
            flags |= SEF_SACL_AUTO_INHERIT;
        }
        let mapping = GENERIC_MAPPING {
            GenericRead: RegistryAccessRight::Read.bits(),
            GenericWrite: RegistryAccessRight::Write.bits(),
            GenericExecute: RegistryAccessRight::Execute.bits(),
            GenericAll: RegistryAccessRight::FullControl.bits(),
        };

        // SAFETY: parent points to a valid self-relative descriptor, descriptor is writable,
        // token is a valid queryable process token, and mapping lives for the duration of the call.
        unsafe {
            CreatePrivateObjectSecurityEx(
                Some(parent),
                None,
                &mut descriptor,
                None,
                true,
                flags,
                Some(token),
                &raw const mapping,
            )
        }
        .map_err(|error| {
            win32_error(
                "CreatePrivateObjectSecurityEx",
                error.code().0 as u32,
            )
        })?;
        Ok(Self(descriptor))
    }

    fn as_ptr(&self) -> PSECURITY_DESCRIPTOR {
        self.0
    }

    fn to_bytes(&self) -> Result<Vec<u8>, RegistryError> {
        // SAFETY: self owns a valid descriptor returned by CreatePrivateObjectSecurityEx.
        let length = unsafe { GetSecurityDescriptorLength(self.0) } as usize;
        if length == 0 {
            return Err(win32_error("GetSecurityDescriptorLength", 0));
        }
        // SAFETY: GetSecurityDescriptorLength returned the readable size of this descriptor.
        Ok(unsafe { core::slice::from_raw_parts(self.0.0.cast::<u8>(), length) }.to_vec())
    }
}

impl Drop for PrivateSecurityDescriptor {
    fn drop(&mut self) {
        // SAFETY: This descriptor was allocated by CreatePrivateObjectSecurityEx and is owned by
        // this wrapper. DestroyPrivateObjectSecurity releases it and clears the pointer.
        unsafe {
            let _ = DestroyPrivateObjectSecurity(&raw const self.0);
        }
    }
}

/// Validates that `requested` is well-formed: that it specifies at least one section unless
/// `_purge` is explicitly true (which clears omitted sections), and that every entry is
/// well-formed for the section it belongs to. Performs no registry access.
///
/// Called centrally by every operation ([`get`], [`set`] via [`effective`], and the registry
/// resource's `test`) before any registry read, mutation, or what-if preview, so that an invalid
/// `accessControlList: {}` is rejected consistently regardless of which operation is requested.
///
/// # Errors
///
/// Returns [`RegistryError::Acl`] when `requested` is not well-formed.
pub fn validate(requested: &RegistryAccessControlList) -> Result<(), RegistryError> {
    if requested.discretionary.is_none()
        && requested.system.is_none()
        && requested.purge != Some(true)
    {
        return Err(RegistryError::Acl(t!("error.invalidAcl").to_string()));
    }
    requested.validate().map_err(acl_error)
}

/// Validates an ACL request, resolves every principal, and enables any privilege required to
/// access its sections. Call this before a larger operation begins mutating registry keys or
/// values so ACL-specific failures cannot leave unrelated registry data partially updated.
pub fn preflight(requested: &RegistryAccessControlList) -> Result<(), RegistryError> {
    validate(requested)?;
    ensure_privilege_for(requested)?;
    normalize(requested).map_err(acl_error)?;
    Ok(())
}

/// Enables the `SeSecurityPrivilege` privilege when `requested` manages the system access
/// control list (SACL), since reading or writing a SACL -- or merely opening a key with the
/// `ACCESS_SYSTEM_SECURITY` right, as [`can_set`] does to check whether a key can be written to
/// -- requires it. Centralizes the check so every entry point ([`get`], [`set`], and [`can_set`],
/// including when running in what-if mode) enables the privilege before its first SACL-touching
/// Win32 call, rather than relying on each call site to remember to do so individually.
///
/// # Errors
///
/// Returns [`RegistryError::Acl`] when the privilege can't be enabled.
fn ensure_privilege_for(requested: &RegistryAccessControlList) -> Result<(), RegistryError> {
    if requested.manages_system() {
        enable_security_privilege().map_err(acl_error)?;
    }
    Ok(())
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
    validate(requested)?;
    let want_discretionary = requested.manages_discretionary();
    let want_system = requested.manages_system();

    let mut descriptor_bytes = read_descriptor(path, requested)?;
    // SAFETY: descriptor_bytes was populated by RegGetKeySecurity and is a complete, well-formed
    // self-relative security descriptor.
    unsafe { read_access_control_list(&mut descriptor_bytes, want_discretionary, want_system) }
        .map_err(acl_error)
}

fn read_descriptor(
    path: &str,
    requested: &RegistryAccessControlList,
) -> Result<Vec<u8>, RegistryError> {
    ensure_privilege_for(requested)?;
    let key = open(path, read_access(requested))?;
    let want_discretionary = requested.manages_discretionary();
    let want_system = requested.manages_system();
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

    Ok(descriptor_bytes)
}

/// Returns whether `path` currently exists and could accept `requested` being applied via
/// [`set`]. Opening the key for this check alone requires the `SeSecurityPrivilege` privilege
/// when `requested` manages the system access control list (SACL), since the check opens the key
/// with the `ACCESS_SYSTEM_SECURITY` right; this function enables the privilege as needed
/// (including when called to preview a what-if `set`, which never calls [`set`] itself).
///
/// # Errors
///
/// Returns [`RegistryError::Acl`] when a lower-level Win32 API call fails for a reason other
/// than the key not existing.
pub fn can_set(path: &str, requested: &RegistryAccessControlList) -> Result<bool, RegistryError> {
    ensure_privilege_for(requested)?;
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
    validate(requested)?;
    ensure_privilege_for(requested)?;

    let mut prepared = prepare(path, requested)?;
    set_prepared(path, &mut prepared)
}

/// Reads, merges, normalizes, and builds the complete descriptor without changing the registry.
pub(crate) fn prepare(
    path: &str,
    requested: &RegistryAccessControlList,
) -> Result<PreparedAccessControlList, RegistryError> {
    validate(requested)?;
    ensure_privilege_for(requested)?;
    let effective = effective(path, requested)?;
    let (descriptor_bytes, security_information) =
        build_self_relative_security_descriptor(&effective).map_err(acl_error)?;
    Ok(PreparedAccessControlList {
        effective,
        descriptor_bytes,
        security_information,
    })
}

/// Writes a descriptor returned by [`prepare`] without rereading or rebuilding it.
pub(crate) fn set_prepared(
    path: &str,
    prepared: &mut PreparedAccessControlList,
) -> Result<(), RegistryError> {
    let descriptor =
        PSECURITY_DESCRIPTOR(prepared.descriptor_bytes.as_mut_ptr().cast::<c_void>());
    let key = open(path, write_access(&prepared.effective))?;
    // SAFETY: descriptor_bytes remains alive for the duration of this call.
    let status =
        unsafe { RegSetKeySecurity(key.0, prepared.security_information, descriptor) };
    if status != ERROR_SUCCESS {
        return Err(win32_error("RegSetKeySecurity", status.0));
    }

    Ok(())
}

/// Computes the effective access control list that should be written for `requested`, honoring
/// its `_purge` setting: omitted `_purge` replaces supplied sections, explicit `true` replaces
/// both sections and clears omitted sections, and neither replacement mode needs a registry read.
/// A protection flag that is not explicitly supplied resolves to its effective `false` default
/// (see [`merge_access_control_list`]). When `false`, reads `path`'s current state for the
/// section(s) `requested` manages and merges `requested`'s entries into them.
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
        // `merge_access_control_list`'s purge branch never consults `current`, so an empty
        // placeholder avoids a registry read that isn't needed to compute the effective ACL --
        // this also lets a purge write preview succeed even when `path` doesn't exist yet.
        return Ok(merge_access_control_list(&RegistryAccessControlList::default(), &normalized));
    }

    let current = get(path, &normalized)?;
    Ok(merge_access_control_list(&current, &normalized))
}

/// Computes the effective ACL preview for a target that does not exist yet. Replacement mode does
/// not depend on existing state. Merge mode finds the nearest existing parent and uses Windows'
/// native inheritance algorithm for every missing path component before merging the requested
/// entries, so the preview includes the entries that registry key creation will propagate.
pub fn effective_for_new(
    path: &str,
    requested: &RegistryAccessControlList,
) -> Result<RegistryAccessControlList, RegistryError> {
    validate(requested)?;
    let normalized = normalize(requested).map_err(acl_error)?;
    if normalized.purge() {
        return Ok(merge_access_control_list(
            &RegistryAccessControlList::default(),
            &normalized,
        ));
    }

    let (parent_path, missing_levels) = nearest_existing_parent(path, read_access(&normalized))?;
    let mut parent_bytes = read_descriptor(&parent_path, &normalized)?;
    let token = ProcessToken::open()?;
    let mut inherited: Option<PrivateSecurityDescriptor> = None;
    for _ in 0..missing_levels {
        let parent = inherited.as_ref().map_or_else(
            || PSECURITY_DESCRIPTOR(parent_bytes.as_mut_ptr().cast::<c_void>()),
            PrivateSecurityDescriptor::as_ptr,
        );
        inherited = Some(PrivateSecurityDescriptor::inherit_from(
            parent, &normalized, token.0,
        )?);
    }
    let Some(inherited) = inherited else {
        return effective(path, &normalized);
    };
    let mut inherited_bytes = inherited.to_bytes()?;
    // SAFETY: inherited_bytes is a copy of the complete descriptor created by
    // CreatePrivateObjectSecurityEx and remains alive for this call.
    let mut current = unsafe {
        read_access_control_list(
            &mut inherited_bytes,
            normalized.manages_discretionary(),
            normalized.manages_system(),
        )
    }
    .map_err(acl_error)?;
    // Registry key creation copies inherited ACEs onto the new key without retaining the
    // INHERITED_ACE marker. CreatePrivateObjectSecurityEx performs the same propagation rules but
    // sets that marker, so remove it to match the descriptor RegCreateKeyExW actually produces.
    for entries in [&mut current.discretionary, &mut current.system]
        .into_iter()
        .flatten()
    {
        for entry in entries {
            entry.flags.retain(|flag| *flag != AceFlag::Inherited);
        }
    }

    Ok(merge_access_control_list(&current, &normalized))
}

fn nearest_existing_parent(
    path: &str,
    access: REG_SAM_FLAGS,
) -> Result<(String, usize), RegistryError> {
    let (hive, subkey) = path
        .split_once('\\')
        .map_or((path, ""), |(hive, subkey)| (hive, subkey));
    split_path(path)?;
    let parts: Vec<&str> = subkey
        .split('\\')
        .filter(|part| !part.is_empty())
        .collect();

    for existing_count in (0..parts.len()).rev() {
        let candidate = if existing_count == 0 {
            hive.to_string()
        } else {
            format!("{hive}\\{}", parts[..existing_count].join("\\"))
        };
        match open(&candidate, access) {
            Ok(_) => return Ok((candidate, parts.len() - existing_count)),
            Err(RegistryError::RegistryKeyNotFound(_)) => {}
            Err(error) => return Err(error),
        }
    }

    Err(RegistryError::RegistryKeyNotFound(path.to_string()))
}

/// Returns whether `actual` (as returned by [`get`] for the section(s) `requested` manages)
/// already satisfies `requested`, honoring its `_purge` setting exactly as [`effective`] does
/// when computing what a [`set`] would write, but without performing a registry read: the merge
/// is computed directly against the already-available `actual`, which is what a purge-mode
/// `_purge: true` write would need anyway and what a merge-mode `_purge: false` write would
/// otherwise have to re-read from the registry.
///
/// # Errors
///
/// Returns [`RegistryError::Acl`] when a principal in `requested` cannot be resolved.
pub fn is_satisfied(
    requested: &RegistryAccessControlList,
    actual: &RegistryAccessControlList,
) -> Result<bool, RegistryError> {
    let normalized = normalize(requested).map_err(acl_error)?;
    let effective_desired = merge_access_control_list(actual, &normalized);
    Ok(lib_acl::is_satisfied(&effective_desired, actual))
}

fn read_access(requested: &RegistryAccessControlList) -> REG_SAM_FLAGS {
    let mut access = REG_SAM_FLAGS(0);
    if requested.manages_discretionary() {
        access |= READ_CONTROL;
    }
    if requested.manages_system() {
        access |= ACCESS_SYSTEM_SECURITY;
    }
    access
}

fn write_access(requested: &RegistryAccessControlList) -> REG_SAM_FLAGS {
    let mut access = REG_SAM_FLAGS(0);
    if requested.manages_discretionary() {
        access |= WRITE_DAC;
    }
    if requested.manages_system() {
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
    use lib_acl::{AccessControlEntry, AceType, AuditFlag, Principal};
    use windows::Win32::Foundation::{CloseHandle, HANDLE, LUID};
    use windows::Win32::Security::{
        AdjustTokenPrivileges, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW,
        TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    use windows::core::{HSTRING, PCWSTR};

    fn entry(ace_type: AceType) -> AccessControlEntry<RegistryAccessRight> {
        AccessControlEntry {
            principal: Principal::from_sid("S-1-5-18").unwrap(),
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
    fn explicit_purge_true_requests_access_for_both_sections() {
        let requested = RegistryAccessControlList {
            purge: Some(true),
            ..Default::default()
        };

        assert_eq!(
            read_access(&requested),
            READ_CONTROL | ACCESS_SYSTEM_SECURITY
        );
        assert_eq!(
            write_access(&requested),
            WRITE_DAC | ACCESS_SYSTEM_SECURITY
        );
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
        // Explicit `_purge: true` never needs to read the key's current state, so this
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
        new_entry.principal = Principal::from_sid("S-1-5-19").unwrap(); // LOCAL_SERVICE SID
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

    #[test]
    fn validate_rejects_access_control_list_with_no_sections() {
        let requested = RegistryAccessControlList::default();
        let error = validate(&requested).unwrap_err();

        assert!(matches!(error, RegistryError::Acl(_)));
    }

    #[test]
    fn validate_accepts_access_control_list_with_no_sections_when_purge_is_true() {
        let requested = RegistryAccessControlList {
            purge: Some(true),
            ..Default::default()
        };

        validate(&requested).unwrap();
    }

    #[test]
    fn effective_with_purge_true_defaults_protected_to_false_without_reading_current() {
        let requested = RegistryAccessControlList {
            discretionary: Some(vec![entry(AceType::Allow)]),
            purge: Some(true),
            ..Default::default()
        };

        // Succeeds (and resolves the omitted protection flag to its effective default) even
        // though the key below doesn't exist, proving no registry read of "current" is needed
        // to compute the effective ACL in purge mode.
        let effective_acl =
            effective(r"HKCU\DoesNotExist\AclPurgeProtectedDefault", &requested).unwrap();

        assert_eq!(effective_acl.discretionary_protected, Some(false));
    }

    #[test]
    fn is_satisfied_reports_true_when_actual_already_matches_requested() {
        let actual = RegistryAccessControlList {
            discretionary: Some(vec![entry(AceType::Allow)]),
            discretionary_protected: Some(false),
            system: Some(Vec::new()),
            system_protected: Some(false),
            ..Default::default()
        };
        let requested = RegistryAccessControlList {
            discretionary: Some(vec![entry(AceType::Allow)]),
            purge: Some(true),
            ..Default::default()
        };

        assert!(is_satisfied(&requested, &actual).unwrap());
    }

    #[test]
    fn is_satisfied_reports_false_when_actual_differs() {
        let actual = RegistryAccessControlList {
            discretionary: Some(vec![entry(AceType::Allow)]),
            discretionary_protected: Some(false),
            system: Some(Vec::new()),
            system_protected: Some(false),
            ..Default::default()
        };
        let mut different_entry = entry(AceType::Allow);
        different_entry.principal = Principal::from_sid("S-1-5-19").unwrap();
        let requested = RegistryAccessControlList {
            discretionary: Some(vec![different_entry]),
            purge: Some(true),
            ..Default::default()
        };

        assert!(!is_satisfied(&requested, &actual).unwrap());
    }

    fn disable_security_privilege() {
        let mut token = HANDLE::default();
        // SAFETY: the current-process pseudo-handle is valid and token is a writable local.
        unsafe {
            OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                &mut token,
            )
        }
        .unwrap();

        let name = HSTRING::from(lib_acl::win32::SE_SECURITY_NAME);
        let mut luid = LUID::default();
        // SAFETY: name is a valid null-terminated string and luid is writable.
        unsafe { LookupPrivilegeValueW(PCWSTR::null(), PCWSTR(name.as_ptr()), &mut luid) }.unwrap();
        let privileges = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: Default::default(),
            }],
        };
        // SAFETY: token is valid and privileges describes one privilege to disable.
        unsafe {
            AdjustTokenPrivileges(
                token,
                false,
                Some(&raw const privileges),
                0,
                None,
                None,
            )
        }
        .unwrap();
        // SAFETY: token is owned by this helper.
        unsafe {
            let _ = CloseHandle(token);
        }
    }

    #[test]
    fn can_set_enables_privilege_in_an_isolated_process() {
        let module = module_path!()
            .strip_prefix(concat!(env!("CARGO_CRATE_NAME"), "::"))
            .unwrap_or(module_path!());
        let test_name =
            format!("{module}::can_set_enables_privilege_in_an_isolated_process_child");
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", &test_name])
            .env("DSC_ACL_PRIVILEGE_CHILD", "1")
            .status()
            .unwrap();

        assert!(status.success());
    }

    #[test]
    #[ignore = "launched by can_set_enables_privilege_in_an_isolated_process"]
    fn can_set_enables_privilege_in_an_isolated_process_child() {
        if std::env::var_os("DSC_ACL_PRIVILEGE_CHILD").is_none() {
            return;
        }
        if lib_acl::win32::enable_security_privilege().is_err() {
            return;
        }

        let key_path = format!(
            r"HKCU\Software\DscAclPrivilegeTest-{}",
            std::process::id()
        );
        let helper = crate::RegistryHelper::new(&key_path, None, None).unwrap();
        helper.set().unwrap();
        disable_security_privilege();

        let mut audit_entry = entry(AceType::Audit);
        audit_entry.audit_flags = vec![AuditFlag::Success];
        let requested = RegistryAccessControlList {
            system: Some(vec![audit_entry]),
            ..Default::default()
        };

        let can_set_result = can_set(&key_path, &requested);
        let _ = helper.remove();

        assert!(matches!(can_set_result, Ok(true)), "{can_set_result:?}");
    }
}
