// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types::FilesystemAcl;
use rust_i18n::t;
use std::ptr;
use windows::Win32::Foundation::{HLOCAL, LocalFree};
use windows::Win32::Security::Authorization::{
    ConvertSecurityDescriptorToStringSecurityDescriptorW,
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{
    DACL_SECURITY_INFORMATION, GetFileSecurityW, GetSecurityDescriptorControl,
    OBJECT_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
    SE_DACL_PROTECTED, SetFileSecurityW, UNPROTECTED_DACL_SECURITY_INFORMATION,
};
use windows::core::{PCWSTR, PWSTR};

pub fn get(input: &FilesystemAcl) -> Result<FilesystemAcl, String> {
    validate_input(input)?;
    Ok(FilesystemAcl {
        path: input.path.clone(),
        sddl: read_sddl(&input.path)?,
        in_desired_state: None,
    })
}

pub fn set(input: &FilesystemAcl) -> Result<FilesystemAcl, String> {
    validate_input(input)?;
    let security_descriptor = SecurityDescriptor::from_sddl(&input.sddl)?;
    let path = to_wide(&input.path);
    let security_information = security_descriptor.dacl_security_information()?;

    unsafe {
        SetFileSecurityW(
            PCWSTR(path.as_ptr()),
            security_information,
            security_descriptor.as_ptr(),
        )
    }
    .ok()
    .map_err(|error| {
        t!(
            "acl.setError",
            path = input.path.as_str(),
            error = error.to_string()
        )
        .to_string()
    })?;

    get(input)
}

pub fn test(input: &FilesystemAcl) -> Result<FilesystemAcl, String> {
    validate_input(input)?;
    let desired = SecurityDescriptor::from_sddl(&input.sddl)?.to_sddl()?;
    let mut actual = get(input)?;
    actual.in_desired_state = Some(actual.sddl.eq_ignore_ascii_case(&desired));
    Ok(actual)
}

fn validate_input(input: &FilesystemAcl) -> Result<(), String> {
    if input.path.is_empty() {
        return Err(t!("input.emptyPath").to_string());
    }
    if input.sddl.is_empty() {
        return Err(t!("acl.emptySddl").to_string());
    }
    if !has_top_level_section(&input.sddl, b"D:")
        || [b"O:", b"G:", b"S:"]
            .iter()
            .any(|section| has_top_level_section(&input.sddl, section))
    {
        return Err(t!("acl.unsupportedSddlComponent").to_string());
    }
    if !std::path::Path::new(&input.path).exists() {
        return Err(t!("acl.pathNotFound", path = input.path.as_str()).to_string());
    }
    Ok(())
}

fn has_top_level_section(sddl: &str, section: &[u8; 2]) -> bool {
    let bytes = sddl.as_bytes();
    let mut depth = 0_u32;
    for index in 0..bytes.len().saturating_sub(1) {
        match bytes[index] {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            _ if depth == 0
                && bytes[index].eq_ignore_ascii_case(&section[0])
                && bytes[index + 1] == section[1] =>
            {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn read_sddl(path: &str) -> Result<String, String> {
    let path_wide = to_wide(path);
    let mut required_length = 0;
    let first_result = unsafe {
        GetFileSecurityW(
            PCWSTR(path_wide.as_ptr()),
            DACL_SECURITY_INFORMATION.0,
            None,
            0,
            &mut required_length,
        )
    };
    if required_length == 0 {
        return Err(t!(
            "acl.getError",
            path = path,
            error = first_result
                .ok()
                .err()
                .map_or_else(|| t!("acl.emptyDescriptor").to_string(), |error| error.to_string())
        )
        .to_string());
    }

    let mut buffer = vec![0_u8; required_length as usize];
    unsafe {
        GetFileSecurityW(
            PCWSTR(path_wide.as_ptr()),
            DACL_SECURITY_INFORMATION.0,
            Some(PSECURITY_DESCRIPTOR(buffer.as_mut_ptr().cast())),
            required_length,
            &mut required_length,
        )
    }
    .ok()
    .map_err(|error| {
        t!("acl.getError", path = path, error = error.to_string()).to_string()
    })?;

    SecurityDescriptor::borrowed(buffer.as_mut_ptr().cast()).to_sddl()
}

fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

struct SecurityDescriptor {
    pointer: PSECURITY_DESCRIPTOR,
    owned: bool,
}

impl SecurityDescriptor {
    fn from_sddl(sddl: &str) -> Result<Self, String> {
        let sddl_wide = to_wide(sddl);
        let mut pointer = PSECURITY_DESCRIPTOR(ptr::null_mut());
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                PCWSTR(sddl_wide.as_ptr()),
                SDDL_REVISION_1,
                &mut pointer,
                None,
            )
        }
        .map_err(|error| t!("acl.invalidSddl", error = error.to_string()).to_string())?;

        Ok(Self {
            pointer,
            owned: true,
        })
    }

    fn borrowed(pointer: *mut core::ffi::c_void) -> Self {
        Self {
            pointer: PSECURITY_DESCRIPTOR(pointer),
            owned: false,
        }
    }

    fn as_ptr(&self) -> PSECURITY_DESCRIPTOR {
        self.pointer
    }

    fn dacl_security_information(&self) -> Result<OBJECT_SECURITY_INFORMATION, String> {
        let mut control = 0_u16;
        let mut revision = 0_u32;
        unsafe { GetSecurityDescriptorControl(self.pointer, &mut control, &mut revision) }
            .map_err(|error| t!("acl.invalidSddl", error = error.to_string()).to_string())?;

        let protection = if control & SE_DACL_PROTECTED.0 != 0 {
            PROTECTED_DACL_SECURITY_INFORMATION
        } else {
            UNPROTECTED_DACL_SECURITY_INFORMATION
        };
        Ok(DACL_SECURITY_INFORMATION | protection)
    }

    fn to_sddl(&self) -> Result<String, String> {
        let mut output = PWSTR::null();
        unsafe {
            ConvertSecurityDescriptorToStringSecurityDescriptorW(
                self.pointer,
                SDDL_REVISION_1,
                DACL_SECURITY_INFORMATION,
                &mut output,
                None,
            )
        }
        .map_err(|error| t!("acl.convertError", error = error.to_string()).to_string())?;

        let result = unsafe { output.to_string() }
            .map_err(|error| t!("acl.convertError", error = error.to_string()).to_string());
        unsafe {
            let _ = LocalFree(Some(HLOCAL(output.0.cast())));
        }
        result
    }
}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                let _ = LocalFree(Some(HLOCAL(self.pointer.0.cast())));
            }
        }
    }
}
