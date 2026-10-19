// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Static table of the Windows user rights and privileges that can be assigned to accounts.

use rust_i18n::t;
use std::borrow::Cow;

/// A user right or privilege.
#[derive(Debug)]
pub struct UserRight {
    /// The constant that the LSA, `secedit` and Group Policy use, e.g. `SeServiceLogonRight`.
    pub constant: &'static str,
    /// Translates the name shown in the Local Security Policy console. See [`UserRight::display_name`].
    display_name: fn() -> Cow<'static, str>,
}

impl UserRight {
    /// The localized name shown in the Local Security Policy console, e.g. "Log on as a service".
    #[must_use]
    pub fn display_name(&self) -> Cow<'static, str> {
        (self.display_name)()
    }
}

/// Build a table entry.
///
/// # Arguments
///
/// * `constant` - The user right constant.
/// * `display_name` - Calls `t!` with the literal `rights.displayName.<constant>` key, so that the
///   i18n tests can match every key in `locales/en-us.toml` to its use.
const fn right(constant: &'static str, display_name: fn() -> Cow<'static, str>) -> UserRight {
    UserRight { constant, display_name }
}

/// Every user right, ordered by English display name. This is also the order `export` reports them in.
pub const USER_RIGHTS: [UserRight; 45] = [
    right("SeTrustedCredManAccessPrivilege", || t!("rights.displayName.SeTrustedCredManAccessPrivilege")),
    right("SeNetworkLogonRight", || t!("rights.displayName.SeNetworkLogonRight")),
    right("SeTcbPrivilege", || t!("rights.displayName.SeTcbPrivilege")),
    right("SeMachineAccountPrivilege", || t!("rights.displayName.SeMachineAccountPrivilege")),
    right("SeIncreaseQuotaPrivilege", || t!("rights.displayName.SeIncreaseQuotaPrivilege")),
    right("SeInteractiveLogonRight", || t!("rights.displayName.SeInteractiveLogonRight")),
    right("SeRemoteInteractiveLogonRight", || t!("rights.displayName.SeRemoteInteractiveLogonRight")),
    right("SeBackupPrivilege", || t!("rights.displayName.SeBackupPrivilege")),
    right("SeChangeNotifyPrivilege", || t!("rights.displayName.SeChangeNotifyPrivilege")),
    right("SeSystemtimePrivilege", || t!("rights.displayName.SeSystemtimePrivilege")),
    right("SeTimeZonePrivilege", || t!("rights.displayName.SeTimeZonePrivilege")),
    right("SeCreatePagefilePrivilege", || t!("rights.displayName.SeCreatePagefilePrivilege")),
    right("SeCreateTokenPrivilege", || t!("rights.displayName.SeCreateTokenPrivilege")),
    right("SeCreateGlobalPrivilege", || t!("rights.displayName.SeCreateGlobalPrivilege")),
    right("SeCreatePermanentPrivilege", || t!("rights.displayName.SeCreatePermanentPrivilege")),
    right("SeCreateSymbolicLinkPrivilege", || t!("rights.displayName.SeCreateSymbolicLinkPrivilege")),
    right("SeDebugPrivilege", || t!("rights.displayName.SeDebugPrivilege")),
    right("SeDenyNetworkLogonRight", || t!("rights.displayName.SeDenyNetworkLogonRight")),
    right("SeDenyBatchLogonRight", || t!("rights.displayName.SeDenyBatchLogonRight")),
    right("SeDenyServiceLogonRight", || t!("rights.displayName.SeDenyServiceLogonRight")),
    right("SeDenyInteractiveLogonRight", || t!("rights.displayName.SeDenyInteractiveLogonRight")),
    right("SeDenyRemoteInteractiveLogonRight", || t!("rights.displayName.SeDenyRemoteInteractiveLogonRight")),
    right("SeEnableDelegationPrivilege", || t!("rights.displayName.SeEnableDelegationPrivilege")),
    right("SeRemoteShutdownPrivilege", || t!("rights.displayName.SeRemoteShutdownPrivilege")),
    right("SeAuditPrivilege", || t!("rights.displayName.SeAuditPrivilege")),
    right("SeImpersonatePrivilege", || t!("rights.displayName.SeImpersonatePrivilege")),
    right("SeIncreaseWorkingSetPrivilege", || t!("rights.displayName.SeIncreaseWorkingSetPrivilege")),
    right("SeIncreaseBasePriorityPrivilege", || t!("rights.displayName.SeIncreaseBasePriorityPrivilege")),
    right("SeLoadDriverPrivilege", || t!("rights.displayName.SeLoadDriverPrivilege")),
    right("SeLockMemoryPrivilege", || t!("rights.displayName.SeLockMemoryPrivilege")),
    right("SeBatchLogonRight", || t!("rights.displayName.SeBatchLogonRight")),
    right("SeServiceLogonRight", || t!("rights.displayName.SeServiceLogonRight")),
    right("SeSecurityPrivilege", || t!("rights.displayName.SeSecurityPrivilege")),
    right("SeRelabelPrivilege", || t!("rights.displayName.SeRelabelPrivilege")),
    right("SeSystemEnvironmentPrivilege", || t!("rights.displayName.SeSystemEnvironmentPrivilege")),
    right("SeDelegateSessionUserImpersonatePrivilege", || t!("rights.displayName.SeDelegateSessionUserImpersonatePrivilege")),
    right("SeManageVolumePrivilege", || t!("rights.displayName.SeManageVolumePrivilege")),
    right("SeProfileSingleProcessPrivilege", || t!("rights.displayName.SeProfileSingleProcessPrivilege")),
    right("SeSystemProfilePrivilege", || t!("rights.displayName.SeSystemProfilePrivilege")),
    right("SeUndockPrivilege", || t!("rights.displayName.SeUndockPrivilege")),
    right("SeAssignPrimaryTokenPrivilege", || t!("rights.displayName.SeAssignPrimaryTokenPrivilege")),
    right("SeRestorePrivilege", || t!("rights.displayName.SeRestorePrivilege")),
    right("SeShutdownPrivilege", || t!("rights.displayName.SeShutdownPrivilege")),
    right("SeSyncAgentPrivilege", || t!("rights.displayName.SeSyncAgentPrivilege")),
    right("SeTakeOwnershipPrivilege", || t!("rights.displayName.SeTakeOwnershipPrivilege")),
];

/// Find a user right by its constant, ignoring case and surrounding whitespace.
#[must_use]
pub fn find(input: &str) -> Option<&'static UserRight> {
    let wanted = input.trim();
    USER_RIGHTS.iter().find(|entry| entry.constant.eq_ignore_ascii_case(wanted))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn table_has_forty_five_unique_rights() {
        assert_eq!(USER_RIGHTS.len(), 45);
        let constants: HashSet<String> = USER_RIGHTS.iter().map(|r| r.constant.to_ascii_lowercase()).collect();
        assert_eq!(constants.len(), 45, "constants must be unique");
        let names: HashSet<String> = USER_RIGHTS.iter().map(|r| r.display_name().into_owned()).collect();
        assert_eq!(names.len(), 45, "display names must be unique");
        for entry in &USER_RIGHTS {
            assert!(entry.constant.starts_with("Se"), "{}", entry.constant);
            assert!(
                entry.constant.ends_with("Privilege") || entry.constant.ends_with("Right"),
                "{}",
                entry.constant
            );
            let name = entry.display_name();
            assert!(!name.is_empty(), "{}", entry.constant);
            assert!(!name.contains("rights.displayName"), "{} has no translation", entry.constant);
        }
    }

    #[test]
    fn display_names_are_translated() {
        let right = find("SeServiceLogonRight").unwrap();
        assert_eq!(right.display_name(), t!("rights.displayName.SeServiceLogonRight"));
        assert_eq!(right.display_name(), "Log on as a service");
    }

    #[test]
    fn find_ignores_case_and_whitespace() {
        assert_eq!(find("SeServiceLogonRight").map(|r| r.constant), Some("SeServiceLogonRight"));
        assert_eq!(find("  seservicelogonright ").map(|r| r.constant), Some("SeServiceLogonRight"));
        assert!(find("SeBogusPrivilege").is_none());
        assert!(find("Log on as a service").is_none());
        assert!(find("").is_none());
    }
}
