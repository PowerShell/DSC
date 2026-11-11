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

/// The name of the registry type the LSA reads for `name`, as used in what-if messages.
fn canonical_type_name(name: AuditOptionName) -> &'static str {
    if name.is_binary() { "REG_BINARY" } else { "REG_DWORD" }
}

/// Whether `data` is stored with the registry type the LSA reads for `name`. A missing value
/// counts as canonical, because the LSA treats it as disabled and there is nothing to rewrite.
fn has_canonical_type(name: AuditOptionName, data: Option<&RegistryValueData>) -> bool {
    match data {
        None | Some(RegistryValueData::None) => true,
        Some(RegistryValueData::Binary(_)) => name.is_binary(),
        Some(RegistryValueData::DWord(_)) => !name.is_binary(),
        Some(_) => false,
    }
}

/// Interpret a registry value as the option state the LSA applies. A missing value means
/// disabled, and any non-zero value of the canonical type (including `CrashOnAuditFail = 2`, the
/// "already crashed" marker) means enabled. A `REG_BINARY` value is enabled when any of its bytes
/// is non-zero, whatever its length. A non-zero value of another supported type is reported as
/// disabled, because the LSA ignores it. That keeps DSC's synthetic test from treating such a
/// value as compliant and skipping the `set` that rewrites it.
///
/// # Errors
///
/// Returns an error when the value has a type other than `REG_DWORD`, `REG_QWORD` or `REG_BINARY`.
pub fn enabled_from_data(name: AuditOptionName, data: Option<&RegistryValueData>) -> Result<bool, AuditError> {
    let non_zero = match data {
        None | Some(RegistryValueData::None) => false,
        Some(RegistryValueData::DWord(value)) => *value != 0,
        Some(RegistryValueData::QWord(value)) => *value != 0,
        Some(RegistryValueData::Binary(bytes)) => bytes.iter().any(|byte| *byte != 0),
        Some(_) => return Err(AuditError::Audit(t!("option.unsupportedType", name = name.to_string()).to_string())),
    };
    Ok(non_zero && has_canonical_type(name, data))
}

/// Whether `set` must write the value backing `name` to reach `desired`. A write is needed when
/// the applied state differs, and also when the state matches but the value has another type,
/// so the stored value always ends up with the type the LSA reads.
///
/// # Errors
///
/// Returns an error when the value has a type other than `REG_DWORD`, `REG_QWORD` or `REG_BINARY`.
fn needs_write(name: AuditOptionName, data: Option<&RegistryValueData>, desired: bool) -> Result<bool, AuditError> {
    Ok(enabled_from_data(name, data)? != desired || !has_canonical_type(name, data))
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

/// Set one option. A value stored with another registry type is rewritten with the canonical
/// type even when it already holds the desired state. In what-if mode nothing is changed and the
/// projected state carries `_metadata.whatIf`. After a change is written the returned state
/// carries `_restartRequired`, because the LSA only reads the value at startup.
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
    if !needs_write(input.name, current_data.as_ref(), desired)? {
        return Ok(state(input.name, current, None));
    }

    let message = if current == desired {
        t!(
            "option.whatIfNormalizeType",
            name = input.name.to_string(),
            kind = canonical_type_name(input.name)
        )
    } else {
        t!(
            "option.whatIfChangeEnabled",
            name = input.name.to_string(),
            current = current.to_string(),
            desired = desired.to_string()
        )
    }
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
    fn enabled_follows_non_zero_values_of_the_canonical_type() {
        let name = AuditOptionName::AuditBaseObjects;
        assert!(!enabled_from_data(name, None).unwrap());
        assert!(!enabled_from_data(name, Some(&RegistryValueData::None)).unwrap());
        assert!(!enabled_from_data(name, Some(&RegistryValueData::DWord(0))).unwrap());
        assert!(enabled_from_data(name, Some(&RegistryValueData::DWord(1))).unwrap());
        assert!(enabled_from_data(AuditOptionName::CrashOnAuditFail, Some(&RegistryValueData::DWord(2))).unwrap());
        let binary = AuditOptionName::FullPrivilegeAuditing;
        assert!(!enabled_from_data(binary, None).unwrap());
        assert!(!enabled_from_data(binary, Some(&RegistryValueData::Binary(Vec::new()))).unwrap());
        assert!(!enabled_from_data(binary, Some(&RegistryValueData::Binary(vec![0]))).unwrap());
        assert!(!enabled_from_data(binary, Some(&RegistryValueData::Binary(vec![0, 0, 0, 0]))).unwrap());
        assert!(enabled_from_data(binary, Some(&RegistryValueData::Binary(vec![1]))).unwrap());
        assert!(enabled_from_data(binary, Some(&RegistryValueData::Binary(vec![0, 1]))).unwrap(), "every byte counts");
        assert!(enabled_from_data(binary, Some(&RegistryValueData::Binary(vec![0, 0, 0, 1]))).unwrap());
        assert!(enabled_from_data(name, Some(&RegistryValueData::String("1".to_string()))).is_err());
    }

    #[test]
    fn values_of_another_type_are_reported_as_disabled() {
        // The LSA ignores these, so get must not report them as enabled.
        let name = AuditOptionName::AuditBaseObjects;
        assert!(!enabled_from_data(name, Some(&RegistryValueData::QWord(1))).unwrap());
        assert!(!enabled_from_data(name, Some(&RegistryValueData::Binary(vec![1]))).unwrap());
        let binary = AuditOptionName::FullPrivilegeAuditing;
        assert!(!enabled_from_data(binary, Some(&RegistryValueData::DWord(1))).unwrap());
        assert!(!enabled_from_data(binary, Some(&RegistryValueData::QWord(1))).unwrap());
    }

    #[test]
    fn canonical_type_follows_the_option() {
        let name = AuditOptionName::AuditBaseObjects;
        assert!(has_canonical_type(name, None));
        assert!(has_canonical_type(name, Some(&RegistryValueData::None)));
        assert!(has_canonical_type(name, Some(&RegistryValueData::DWord(0))));
        assert!(!has_canonical_type(name, Some(&RegistryValueData::QWord(0))));
        assert!(!has_canonical_type(name, Some(&RegistryValueData::Binary(vec![0]))));
        let binary = AuditOptionName::FullPrivilegeAuditing;
        assert!(has_canonical_type(binary, Some(&RegistryValueData::Binary(vec![0]))));
        assert!(!has_canonical_type(binary, Some(&RegistryValueData::DWord(0))));
        assert_eq!(canonical_type_name(name), "REG_DWORD");
        assert_eq!(canonical_type_name(binary), "REG_BINARY");
    }

    #[test]
    fn set_rewrites_a_value_of_another_type_even_when_the_state_matches() {
        let name = AuditOptionName::AuditBaseObjects;
        let qword_one = RegistryValueData::QWord(1);
        // Regression: REG_QWORD 1 used to read as enabled, so set returned early and left it ineffective.
        assert!(needs_write(name, Some(&qword_one), true).unwrap());
        assert!(needs_write(name, Some(&qword_one), false).unwrap());
        assert!(needs_write(name, Some(&RegistryValueData::QWord(0)), false).unwrap());
        assert!(needs_write(AuditOptionName::FullPrivilegeAuditing, Some(&RegistryValueData::DWord(0)), false).unwrap());
        assert!(needs_write(name, Some(&RegistryValueData::String("1".to_string())), true).is_err());
    }

    #[test]
    fn set_leaves_a_canonical_value_in_the_desired_state_alone() {
        let name = AuditOptionName::AuditBaseObjects;
        assert!(!needs_write(name, None, false).unwrap());
        assert!(!needs_write(name, Some(&RegistryValueData::DWord(0)), false).unwrap());
        assert!(!needs_write(name, Some(&RegistryValueData::DWord(1)), true).unwrap());
        assert!(!needs_write(AuditOptionName::CrashOnAuditFail, Some(&RegistryValueData::DWord(2)), true).unwrap());
        let binary = AuditOptionName::FullPrivilegeAuditing;
        assert!(!needs_write(binary, Some(&RegistryValueData::Binary(vec![0, 1])), true).unwrap());
        assert!(needs_write(name, None, true).unwrap());
        assert!(needs_write(name, Some(&RegistryValueData::DWord(1)), false).unwrap());
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
