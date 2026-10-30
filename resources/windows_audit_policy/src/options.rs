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

fn helper(name: AuditOptionName, data: Option<RegistryValueData>) -> Result<RegistryHelper, AuditError> {
    RegistryHelper::new(LSA_KEY, Some(name.value_name().to_string()), data).map_err(|error| {
        AuditError::Audit(t!("option.readFailed", name = name.to_string(), error = error.to_string()).to_string())
    })
}

/// Interpret a registry value as an option state. A missing value means disabled, and any
/// non-zero value (including `CrashOnAuditFail = 2`, the "already crashed" marker) means enabled.
///
/// # Errors
///
/// Returns an error when the value has a type other than `REG_DWORD`, `REG_QWORD` or `REG_BINARY`.
pub fn enabled_from_data(name: AuditOptionName, data: Option<&RegistryValueData>) -> Result<bool, AuditError> {
    match data {
        None | Some(RegistryValueData::None) => Ok(false),
        Some(RegistryValueData::DWord(value)) => Ok(*value != 0),
        Some(RegistryValueData::QWord(value)) => Ok(*value != 0),
        Some(RegistryValueData::Binary(bytes)) => Ok(bytes.first().is_some_and(|byte| *byte != 0)),
        Some(_) => Err(AuditError::Audit(t!("option.unsupportedType", name = name.to_string()).to_string())),
    }
}

/// The registry data to write for an option. An existing `REG_DWORD`, `REG_QWORD` or `REG_BINARY`
/// value keeps its type (every type `enabled_from_data` accepts); otherwise the canonical type for
/// the option is used.
#[must_use]
pub fn data_for(name: AuditOptionName, enabled: bool, current: Option<&RegistryValueData>) -> RegistryValueData {
    match current {
        Some(RegistryValueData::Binary(_)) => RegistryValueData::Binary(vec![u8::from(enabled)]),
        Some(RegistryValueData::DWord(_)) => RegistryValueData::DWord(u32::from(enabled)),
        Some(RegistryValueData::QWord(_)) => RegistryValueData::QWord(u64::from(enabled)),
        _ if name.is_binary() => RegistryValueData::Binary(vec![u8::from(enabled)]),
        _ => RegistryValueData::DWord(u32::from(enabled)),
    }
}

fn read_current(name: AuditOptionName) -> Result<Option<RegistryValueData>, AuditError> {
    let state = helper(name, None)?.get().map_err(|error| {
        AuditError::Audit(t!("option.readFailed", name = name.to_string(), error = error.to_string()).to_string())
    })?;
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

    helper(input.name, Some(data_for(input.name, desired, current_data.as_ref())))?
        .set()
        .map_err(|error| {
            AuditError::Audit(t!("option.writeFailed", name = input.name.to_string(), error = error.to_string()).to_string())
        })?;
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
        assert!(enabled_from_data(name, Some(&RegistryValueData::Binary(vec![1]))).unwrap());
        assert!(enabled_from_data(name, Some(&RegistryValueData::String("1".to_string()))).is_err());
    }

    #[test]
    fn data_preserves_existing_type_or_uses_the_canonical_one() {
        let dword = RegistryValueData::DWord(0);
        let qword = RegistryValueData::QWord(0);
        let binary = RegistryValueData::Binary(vec![0]);
        assert_eq!(
            data_for(AuditOptionName::AuditBaseObjects, true, Some(&dword)),
            RegistryValueData::DWord(1)
        );
        assert_eq!(
            data_for(AuditOptionName::AuditBaseObjects, true, Some(&qword)),
            RegistryValueData::QWord(1)
        );
        assert_eq!(
            data_for(AuditOptionName::FullPrivilegeAuditing, false, Some(&qword)),
            RegistryValueData::QWord(0)
        );
        assert_eq!(
            data_for(AuditOptionName::AuditBaseObjects, false, Some(&binary)),
            RegistryValueData::Binary(vec![0])
        );
        assert_eq!(
            data_for(AuditOptionName::AuditBaseObjects, true, None),
            RegistryValueData::DWord(1)
        );
        assert_eq!(
            data_for(AuditOptionName::FullPrivilegeAuditing, true, None),
            RegistryValueData::Binary(vec![1])
        );
        assert_eq!(
            data_for(AuditOptionName::FullPrivilegeAuditing, false, Some(&dword)),
            RegistryValueData::DWord(0)
        );
    }
}
