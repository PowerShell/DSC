// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Reads and writes the audit policy options stored as values under the LSA registry key.
//! These are the same values that `secedit` and Group Policy write; the LSA reads them at
//! startup, so a change takes effect after a restart, which `set` reports in `_restartRequired`.

use crate::audit::require_elevated;
use crate::types::{AuditError, AuditOptionName, AuditPolicyOption, Metadata, RestartRequired};
use dsc_lib_registry::{RegistryHelper, config::RegistryValueData};
use rust_i18n::t;

const LSA_KEY: &str = r"HKLM\SYSTEM\CurrentControlSet\Control\Lsa";

/// The error for a failed read of `name`.
fn read_error(name: AuditOptionName, error: &str) -> AuditError {
    AuditError::Audit(t!("option.readFailed", name = name.to_string(), error = error).to_string())
}

/// The error for a failed write of `name`.
fn write_error(name: AuditOptionName, error: &str) -> AuditError {
    AuditError::Audit(t!("option.writeFailed", name = name.to_string(), error = error).to_string())
}

/// A registry helper for the value backing `name`.
///
/// # Arguments
///
/// * `name` - The option whose value the helper addresses.
/// * `data` - The data to write, for a helper used by `set`.
/// * `error` - Builds the error for the operation the helper is for, so that a failure on the
///   write path is never reported as a failed read.
fn helper(
    name: AuditOptionName,
    data: Option<RegistryValueData>,
    error: fn(AuditOptionName, &str) -> AuditError,
) -> Result<RegistryHelper, AuditError> {
    RegistryHelper::new(LSA_KEY, Some(name.value_name().to_string()), data).map_err(|e| error(name, &e.to_string()))
}

/// Interpret a registry value as an option state. A missing value means disabled, and any
/// non-zero value (including `CrashOnAuditFail = 2`, the "already crashed" marker) means enabled.
/// A `REG_BINARY` value is enabled when any of its bytes is non-zero, whatever its length.
///
/// # Errors
///
/// Returns an error when the value has a type other than `REG_DWORD`, `REG_QWORD` or `REG_BINARY`.
pub fn enabled_from_data(name: AuditOptionName, data: Option<&RegistryValueData>) -> Result<bool, AuditError> {
    match data {
        None | Some(RegistryValueData::None) => Ok(false),
        Some(RegistryValueData::DWord(value)) => Ok(*value != 0),
        Some(RegistryValueData::QWord(value)) => Ok(*value != 0),
        Some(RegistryValueData::Binary(bytes)) => Ok(bytes.iter().any(|byte| *byte != 0)),
        Some(_) => Err(AuditError::Audit(t!("option.unsupportedType", name = name.to_string()).to_string())),
    }
}

/// The registry data to write for an option: always the canonical type for the option, the one
/// the LSA reads and `secedit` and Group Policy write. Reading is lenient about the type, but
/// writing another type back would report the option as enabled while the LSA ignores it.
#[must_use]
pub fn data_for(name: AuditOptionName, enabled: bool) -> RegistryValueData {
    if name.is_binary() {
        RegistryValueData::Binary(vec![u8::from(enabled)])
    } else {
        RegistryValueData::DWord(u32::from(enabled))
    }
}

fn read_current(name: AuditOptionName) -> Result<Option<RegistryValueData>, AuditError> {
    let state = helper(name, None, read_error)?.get().map_err(|error| read_error(name, &error.to_string()))?;
    Ok(state.value_data)
}

fn state(name: AuditOptionName, enabled: bool, metadata: Option<Metadata>) -> AuditPolicyOption {
    AuditPolicyOption {
        name,
        enabled: Some(enabled),
        metadata,
        restart_required: None,
    }
}

/// Get the current state of one option.
///
/// # Errors
///
/// Returns an error when the registry value cannot be read or has an unexpected type.
pub fn get_option(input: &AuditPolicyOption) -> Result<AuditPolicyOption, AuditError> {
    let current = read_current(input.name)?;
    Ok(state(input.name, enabled_from_data(input.name, current.as_ref())?, None))
}

/// Set one option. In what-if mode nothing is changed and the projected state carries
/// `_metadata.whatIf`. After a change is written the returned state carries `_restartRequired`,
/// because the LSA only reads the value at startup.
///
/// # Errors
///
/// Returns an error when `enabled` is missing, the process is not elevated, or the registry
/// value cannot be read or written.
pub fn set_option(input: &AuditPolicyOption, what_if: bool) -> Result<AuditPolicyOption, AuditError> {
    let desired = input
        .enabled
        .ok_or_else(|| AuditError::InvalidInput(t!("option.enabledRequired").to_string()))?;
    require_elevated()?;

    let current_data = read_current(input.name)?;
    let current = enabled_from_data(input.name, current_data.as_ref())?;
    if current == desired {
        return Ok(state(input.name, current, None));
    }

    let message = t!(
        "option.whatIfChangeEnabled",
        name = input.name.to_string(),
        current = current.to_string(),
        desired = desired.to_string()
    )
    .to_string();
    if what_if {
        return Ok(state(input.name, desired, Metadata::from_messages(vec![message])));
    }

    helper(input.name, Some(data_for(input.name, desired)), write_error)?
        .set()
        .map_err(|error| write_error(input.name, &error.to_string()))?;
    eprintln!(
        "{}",
        serde_json::json!({ "warn": t!("option.restartRequired", name = input.name.to_string()).to_string() })
    );
    Ok(AuditPolicyOption {
        restart_required: Some(vec![RestartRequired::local_system()]),
        ..get_option(input)?
    })
}

/// Export the state of every option.
///
/// # Errors
///
/// Returns an error when any registry value cannot be read or has an unexpected type.
pub fn export_options() -> Result<Vec<AuditPolicyOption>, AuditError> {
    AuditOptionName::ALL
        .iter()
        .map(|name| {
            get_option(&AuditPolicyOption {
                name: *name,
                enabled: None,
                metadata: None,
                restart_required: None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_follows_non_zero_values() {
        let name = AuditOptionName::AuditBaseObjects;
        assert!(!enabled_from_data(name, None).unwrap());
        assert!(!enabled_from_data(name, Some(&RegistryValueData::None)).unwrap());
        assert!(!enabled_from_data(name, Some(&RegistryValueData::DWord(0))).unwrap());
        assert!(enabled_from_data(name, Some(&RegistryValueData::DWord(1))).unwrap());
        assert!(enabled_from_data(AuditOptionName::CrashOnAuditFail, Some(&RegistryValueData::DWord(2))).unwrap());
        assert!(enabled_from_data(name, Some(&RegistryValueData::QWord(1))).unwrap());
        assert!(!enabled_from_data(name, Some(&RegistryValueData::Binary(Vec::new()))).unwrap());
        assert!(!enabled_from_data(name, Some(&RegistryValueData::Binary(vec![0]))).unwrap());
        assert!(!enabled_from_data(name, Some(&RegistryValueData::Binary(vec![0, 0, 0, 0]))).unwrap());
        assert!(enabled_from_data(name, Some(&RegistryValueData::Binary(vec![1]))).unwrap());
        assert!(enabled_from_data(name, Some(&RegistryValueData::Binary(vec![0, 1]))).unwrap(), "every byte counts");
        assert!(enabled_from_data(name, Some(&RegistryValueData::Binary(vec![0, 0, 0, 1]))).unwrap());
        assert!(enabled_from_data(name, Some(&RegistryValueData::String("1".to_string()))).is_err());
    }

    #[test]
    fn data_always_uses_the_canonical_type() {
        assert_eq!(data_for(AuditOptionName::AuditBaseObjects, true), RegistryValueData::DWord(1));
        assert_eq!(data_for(AuditOptionName::AuditBaseObjects, false), RegistryValueData::DWord(0));
        assert_eq!(data_for(AuditOptionName::AuditBaseDirectories, true), RegistryValueData::DWord(1));
        assert_eq!(data_for(AuditOptionName::CrashOnAuditFail, true), RegistryValueData::DWord(1));
        assert_eq!(data_for(AuditOptionName::FullPrivilegeAuditing, true), RegistryValueData::Binary(vec![1]));
        assert_eq!(data_for(AuditOptionName::FullPrivilegeAuditing, false), RegistryValueData::Binary(vec![0]));
    }

    #[test]
    fn helper_reports_failures_for_the_operation_it_is_built_for() {
        let name = AuditOptionName::AuditBaseObjects;
        assert_eq!(read_error(name, "boom").to_string(), t!("option.readFailed", name = "AuditBaseObjects", error = "boom"));
        assert_eq!(write_error(name, "boom").to_string(), t!("option.writeFailed", name = "AuditBaseObjects", error = "boom"));
        assert!(helper(name, None, read_error).is_ok());
        assert!(helper(name, Some(data_for(name, true)), write_error).is_ok());
    }
}
