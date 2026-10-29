// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types::{FileAccessControlList, FileAccessRight};
use dsc_lib_acl::win32::{
    build_self_relative_security_descriptor, enable_security_privilege, normalize,
    read_access_control_list, read_security_information,
};
use dsc_lib_acl::{AccessControlList, AclError, is_satisfied, merge_access_control_list};
use rust_i18n::t;
use windows::Win32::Security::{
    GetFileSecurityW, OBJECT_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SetFileSecurityW,
};
use windows::core::PCWSTR;

type FileAcl = AccessControlList<FileAccessRight>;

pub fn get(input: &FileAccessControlList) -> Result<FileAccessControlList, String> {
    let requested = validate_input(input)?;
    let actual = read_acl(&input.path, requested)?;
    Ok(FileAccessControlList {
        path: input.path.clone(),
        access_control_list: Some(actual),
        in_desired_state: None,
    })
}

pub fn set(input: &FileAccessControlList, what_if: bool) -> Result<FileAccessControlList, String> {
    let requested = validate_input(input)?;
    let normalized = normalize(requested).map_err(acl_error)?;
    if normalized.manages_system() {
        enable_security_privilege().map_err(acl_error)?;
    }

    // The Win32 ACL-writing APIs always replace a section outright, so a merge (`_purge: false`)
    // is implemented by first computing the full, merged section(s) to write. Replacement mode
    // needs no current-state read, since `merge_access_control_list` never consults `current` for
    // managed sections and resolves omitted protection flags to their effective `false` default.
    let current = if normalized.purge() {
        FileAcl::default()
    } else {
        read_acl(&input.path, &normalized)?
    };
    let to_write = merge_access_control_list(&current, &normalized);

    if what_if {
        return Ok(FileAccessControlList {
            path: input.path.clone(),
            access_control_list: Some(to_write),
            in_desired_state: None,
        });
    }

    let (mut descriptor_bytes, security_information) =
        build_self_relative_security_descriptor(&to_write).map_err(acl_error)?;
    write_acl(&input.path, &mut descriptor_bytes, security_information)?;

    get(input)
}

pub fn test(input: &FileAccessControlList) -> Result<FileAccessControlList, String> {
    let requested = validate_input(input)?;
    let desired = normalize(requested).map_err(acl_error)?;
    let actual = read_acl(&input.path, &desired)?;
    let effective_desired = merge_access_control_list(&actual, &desired);

    let in_desired_state = is_satisfied(&effective_desired, &actual);

    Ok(FileAccessControlList {
        path: input.path.clone(),
        access_control_list: Some(actual),
        in_desired_state: Some(in_desired_state),
    })
}

fn validate_input(input: &FileAccessControlList) -> Result<&FileAcl, String> {
    if input.path.is_empty() {
        return Err(t!("input.emptyPath").to_string());
    }
    match std::path::Path::new(&input.path).try_exists() {
        Ok(true) => {}
        Ok(false) => {
            return Err(t!("acl.pathNotFound", path = input.path.as_str()).to_string());
        }
        Err(error) => {
            return Err(t!(
                "acl.pathCheckError",
                path = input.path.as_str(),
                error = error.to_string()
            )
            .to_string());
        }
    }

    let Some(requested) = input.access_control_list.as_ref() else {
        return Err(t!("acl.missingAccessControlList").to_string());
    };
    if requested.discretionary.is_none()
        && requested.system.is_none()
        && requested.purge != Some(true)
    {
        return Err(t!("acl.invalidAccessControlList").to_string());
    }
    requested.validate().map_err(acl_error)?;
    Ok(requested)
}

fn read_acl(path: &str, requested: &FileAcl) -> Result<FileAcl, String> {
    let want_discretionary = requested.manages_discretionary();
    let want_system = requested.manages_system();
    if want_system {
        enable_security_privilege().map_err(acl_error)?;
    }

    let path_wide = to_wide(path);
    let security_information = read_security_information(want_discretionary, want_system);

    let mut required_length = 0_u32;
    // SAFETY: the first call intentionally supplies no buffer to obtain the required size.
    let first_result = unsafe {
        GetFileSecurityW(
            PCWSTR(path_wide.as_ptr()),
            security_information.0,
            None,
            0,
            &mut required_length,
        )
    };
    if required_length == 0 {
        let error = first_result.ok().err().map_or_else(
            || t!("acl.emptyDescriptor").to_string(),
            |error| error.to_string(),
        );
        return Err(t!("acl.getError", path = path, error = error).to_string());
    }

    let mut buffer = vec![0_u8; required_length as usize];
    // SAFETY: buffer has the size requested by GetFileSecurityW above.
    unsafe {
        GetFileSecurityW(
            PCWSTR(path_wide.as_ptr()),
            security_information.0,
            Some(PSECURITY_DESCRIPTOR(buffer.as_mut_ptr().cast())),
            required_length,
            &mut required_length,
        )
    }
    .ok()
    .map_err(|error| t!("acl.getError", path = path, error = error.to_string()).to_string())?;

    // SAFETY: buffer was just populated in-place by a successful GetFileSecurityW call above,
    // sized exactly to the `required_length` that same API reported; it is therefore a
    // complete, well-formed self-relative security descriptor.
    unsafe { read_access_control_list(&mut buffer, want_discretionary, want_system) }
        .map_err(acl_error)
}

/// Writes `security_information`'s sections (DACL and/or SACL) to `path`, including their
/// protection state.
///
/// `SetFileSecurityW` is documented as obsolete and, in practice, silently ignores the
/// `PROTECTED_DACL_SECURITY_INFORMATION` / `UNPROTECTED_DACL_SECURITY_INFORMATION` (and SACL
/// equivalent) bits in its `SecurityInformation` parameter. However, it does persist whatever
/// the security descriptor's own control word says, and
/// [`build_self_relative_security_descriptor`] stamps the requested protection state directly
/// into that control word for exactly this reason, so a single call here is sufficient; unlike
/// the newer `SetNamedSecurityInfoW`, it also applies the DACL/SACL literally, without merging
/// in inherited entries from the parent container.
fn write_acl(
    path: &str,
    descriptor_bytes: &mut [u8],
    security_information: OBJECT_SECURITY_INFORMATION,
) -> Result<(), String> {
    let path_wide = to_wide(path);
    let descriptor = PSECURITY_DESCRIPTOR(descriptor_bytes.as_mut_ptr().cast());
    // SAFETY: descriptor_bytes remains alive for the duration of this call.
    unsafe { SetFileSecurityW(PCWSTR(path_wide.as_ptr()), security_information, descriptor) }
        .ok()
        .map_err(|error| t!("acl.setError", path = path, error = error.to_string()).to_string())
}

fn acl_error(error: AclError) -> String {
    error.to_string()
}

fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
