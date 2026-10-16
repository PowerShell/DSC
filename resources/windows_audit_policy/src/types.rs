// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde::{Deserialize, Serialize};

/// Desired or actual state of one audit policy subcategory (`Microsoft.Windows/AuditPolicy`).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditPolicy {
    /// The subcategory, as its English name (case-insensitive) or its GUID. Echoed back as given.
    pub subcategory: String,

    /// Whether successful events in the subcategory are audited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,

    /// Whether failed events in the subcategory are audited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<bool>,

    /// Read-only: the subcategory GUID, e.g. `{0CCE9215-69AE-11D9-BED3-505054503030}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guid: Option<String>,

    /// Read-only: the English name of the category that contains the subcategory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Metadata returned by what-if operations describing the changes that would be applied.
    #[serde(rename = "_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

/// The audit policy options stored as values under `HKLM\SYSTEM\CurrentControlSet\Control\Lsa`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditOptionName {
    /// Shut down the system immediately if it is unable to log security audits (`REG_DWORD`).
    CrashOnAuditFail,
    /// Audit the use of the Backup and Restore privileges (`REG_BINARY`, one byte).
    FullPrivilegeAuditing,
    /// Audit the access of global system objects (`REG_DWORD`).
    AuditBaseObjects,
    /// Audit the access of global system directories (`REG_DWORD`).
    AuditBaseDirectories,
}

impl AuditOptionName {
    /// Every option, in the order `export` reports them.
    pub const ALL: [Self; 4] = [
        Self::CrashOnAuditFail,
        Self::FullPrivilegeAuditing,
        Self::AuditBaseObjects,
        Self::AuditBaseDirectories,
    ];

    /// The registry value name backing the option.
    #[must_use]
    pub fn value_name(self) -> &'static str {
        match self {
            Self::CrashOnAuditFail => "CrashOnAuditFail",
            Self::FullPrivilegeAuditing => "FullPrivilegeAuditing",
            Self::AuditBaseObjects => "AuditBaseObjects",
            Self::AuditBaseDirectories => "AuditBaseDirectories",
        }
    }

    /// Whether Windows stores the option as a one-byte `REG_BINARY` value instead of a `REG_DWORD`.
    #[must_use]
    pub fn is_binary(self) -> bool {
        matches!(self, Self::FullPrivilegeAuditing)
    }
}

impl std::fmt::Display for AuditOptionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.value_name())
    }
}

/// Desired or actual state of one audit policy option (`Microsoft.Windows/AuditPolicyOption`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditPolicyOption {
    /// The option to manage.
    pub name: AuditOptionName,

    /// Whether the option is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// Metadata returned by what-if operations describing the changes that would be applied.
    #[serde(rename = "_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

/// Metadata returned by what-if operations.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// Human-readable descriptions of the changes that would be applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub what_if: Option<Vec<String>>,
}

impl Metadata {
    /// Wrap what-if messages, returning `None` when there is nothing to report.
    #[must_use]
    pub fn from_messages(messages: Vec<String>) -> Option<Self> {
        if messages.is_empty() {
            None
        } else {
            Some(Self { what_if: Some(messages) })
        }
    }
}

/// `AuditingInformation` bit for success auditing (`POLICY_AUDIT_EVENT_SUCCESS`).
pub const AUDIT_SUCCESS: u32 = 0x1;
/// `AuditingInformation` bit for failure auditing (`POLICY_AUDIT_EVENT_FAILURE`).
pub const AUDIT_FAILURE: u32 = 0x2;
/// `AuditingInformation` value that explicitly disables auditing (`POLICY_AUDIT_EVENT_NONE`).
pub const AUDIT_NONE: u32 = 0x4;

/// The success and failure auditing flags of a subcategory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditFlags {
    pub success: bool,
    pub failure: bool,
}

impl AuditFlags {
    /// Decode an `AuditingInformation` value. Only the success and failure bits are inspected,
    /// so both `POLICY_AUDIT_EVENT_UNCHANGED` (0) and `POLICY_AUDIT_EVENT_NONE` (4) decode as
    /// "no auditing".
    #[must_use]
    pub fn from_bits(bits: u32) -> Self {
        Self {
            success: bits & AUDIT_SUCCESS != 0,
            failure: bits & AUDIT_FAILURE != 0,
        }
    }

    /// Encode as an `AuditingInformation` value for `AuditSetSystemPolicy`. When neither flag is
    /// set this returns `POLICY_AUDIT_EVENT_NONE`, never 0, because 0 means "leave unchanged".
    #[must_use]
    pub fn to_bits(self) -> u32 {
        let bits = (u32::from(self.success) * AUDIT_SUCCESS) | (u32::from(self.failure) * AUDIT_FAILURE);
        if bits == 0 { AUDIT_NONE } else { bits }
    }
}

/// Errors from the audit policy resource.
#[derive(Debug)]
pub enum AuditError {
    /// The operation needs an elevated process.
    ElevationRequired,
    /// The input is well-formed JSON but not usable (unknown subcategory, missing property).
    InvalidInput(String),
    /// A Win32, LSA or registry operation failed.
    Audit(String),
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ElevationRequired => f.write_str(&t!("main.elevationRequired")),
            Self::InvalidInput(message) | Self::Audit(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for AuditError {}

impl From<String> for AuditError {
    fn from(message: String) -> Self {
        Self::Audit(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_decode_only_the_success_and_failure_bits() {
        let off = AuditFlags { success: false, failure: false };
        assert_eq!(AuditFlags::from_bits(0), off);
        assert_eq!(AuditFlags::from_bits(AUDIT_NONE), off);
        assert_eq!(AuditFlags::from_bits(1), AuditFlags { success: true, failure: false });
        assert_eq!(AuditFlags::from_bits(2), AuditFlags { success: false, failure: true });
        assert_eq!(AuditFlags::from_bits(3), AuditFlags { success: true, failure: true });
        assert_eq!(AuditFlags::from_bits(5), AuditFlags { success: true, failure: false });
    }

    #[test]
    fn flags_encode_none_when_both_are_off() {
        assert_eq!(AuditFlags { success: false, failure: false }.to_bits(), AUDIT_NONE);
        assert_eq!(AuditFlags { success: true, failure: false }.to_bits(), 1);
        assert_eq!(AuditFlags { success: false, failure: true }.to_bits(), 2);
        assert_eq!(AuditFlags { success: true, failure: true }.to_bits(), 3);
    }

    #[test]
    fn flags_round_trip() {
        for bits in 1..=3 {
            assert_eq!(AuditFlags::from_bits(bits).to_bits(), bits);
        }
    }

    #[test]
    fn option_name_serializes_as_its_registry_value_name() {
        for name in AuditOptionName::ALL {
            let json = serde_json::to_string(&name).unwrap();
            assert_eq!(json, format!("\"{}\"", name.value_name()));
            let parsed: AuditOptionName = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, name);
        }
        assert!(serde_json::from_str::<AuditOptionName>("\"NotAnOption\"").is_err());
        assert!(AuditOptionName::FullPrivilegeAuditing.is_binary());
        assert!(!AuditOptionName::CrashOnAuditFail.is_binary());
    }

    #[test]
    fn audit_policy_rejects_unknown_fields_and_requires_subcategory() {
        assert!(serde_json::from_str::<AuditPolicy>(r#"{"subcategory":"Logon","bogus":1}"#).is_err());
        assert!(serde_json::from_str::<AuditPolicy>(r#"{"success":true}"#).is_err());
        let policy: AuditPolicy = serde_json::from_str(r#"{"subcategory":"Logon","success":true}"#).unwrap();
        assert_eq!(policy.subcategory, "Logon");
        assert_eq!(policy.success, Some(true));
        assert_eq!(policy.failure, None);
        assert_eq!(serde_json::to_string(&policy).unwrap(), r#"{"subcategory":"Logon","success":true}"#);
    }

    #[test]
    fn option_rejects_unknown_fields_and_requires_name() {
        assert!(serde_json::from_str::<AuditPolicyOption>(r#"{"enabled":true}"#).is_err());
        assert!(serde_json::from_str::<AuditPolicyOption>(r#"{"name":"AuditBaseObjects","bogus":1}"#).is_err());
        let option: AuditPolicyOption =
            serde_json::from_str(r#"{"name":"AuditBaseObjects","enabled":false}"#).unwrap();
        assert_eq!(option.name, AuditOptionName::AuditBaseObjects);
        assert_eq!(option.enabled, Some(false));
    }

    #[test]
    fn metadata_is_none_without_messages() {
        assert!(Metadata::from_messages(Vec::new()).is_none());
        let metadata = Metadata::from_messages(vec!["Would change".to_string()]).unwrap();
        assert_eq!(metadata.what_if.unwrap().len(), 1);
    }
}
