// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Static table of the Windows user rights and privileges that can be assigned to accounts.

/// A user right or privilege.
#[derive(Debug, PartialEq, Eq)]
pub struct UserRight {
    /// The constant that the LSA, `secedit` and Group Policy use, e.g. `SeServiceLogonRight`.
    pub constant: &'static str,
    /// The name shown in the Local Security Policy console, e.g. "Log on as a service".
    pub display_name: &'static str,
}

const fn right(constant: &'static str, display_name: &'static str) -> UserRight {
    UserRight { constant, display_name }
}

/// Every user right, ordered by display name. This is also the order `export` reports them in.
pub const USER_RIGHTS: [UserRight; 45] = [
    right("SeTrustedCredManAccessPrivilege", "Access Credential Manager as a trusted caller"),
    right("SeNetworkLogonRight", "Access this computer from the network"),
    right("SeTcbPrivilege", "Act as part of the operating system"),
    right("SeMachineAccountPrivilege", "Add workstations to domain"),
    right("SeIncreaseQuotaPrivilege", "Adjust memory quotas for a process"),
    right("SeInteractiveLogonRight", "Allow log on locally"),
    right("SeRemoteInteractiveLogonRight", "Allow log on through Remote Desktop Services"),
    right("SeBackupPrivilege", "Back up files and directories"),
    right("SeChangeNotifyPrivilege", "Bypass traverse checking"),
    right("SeSystemtimePrivilege", "Change the system time"),
    right("SeTimeZonePrivilege", "Change the time zone"),
    right("SeCreatePagefilePrivilege", "Create a pagefile"),
    right("SeCreateTokenPrivilege", "Create a token object"),
    right("SeCreateGlobalPrivilege", "Create global objects"),
    right("SeCreatePermanentPrivilege", "Create permanent shared objects"),
    right("SeCreateSymbolicLinkPrivilege", "Create symbolic links"),
    right("SeDebugPrivilege", "Debug programs"),
    right("SeDenyNetworkLogonRight", "Deny access to this computer from the network"),
    right("SeDenyBatchLogonRight", "Deny log on as a batch job"),
    right("SeDenyServiceLogonRight", "Deny log on as a service"),
    right("SeDenyInteractiveLogonRight", "Deny log on locally"),
    right("SeDenyRemoteInteractiveLogonRight", "Deny log on through Remote Desktop Services"),
    right("SeEnableDelegationPrivilege", "Enable computer and user accounts to be trusted for delegation"),
    right("SeRemoteShutdownPrivilege", "Force shutdown from a remote system"),
    right("SeAuditPrivilege", "Generate security audits"),
    right("SeImpersonatePrivilege", "Impersonate a client after authentication"),
    right("SeIncreaseWorkingSetPrivilege", "Increase a process working set"),
    right("SeIncreaseBasePriorityPrivilege", "Increase scheduling priority"),
    right("SeLoadDriverPrivilege", "Load and unload device drivers"),
    right("SeLockMemoryPrivilege", "Lock pages in memory"),
    right("SeBatchLogonRight", "Log on as a batch job"),
    right("SeServiceLogonRight", "Log on as a service"),
    right("SeSecurityPrivilege", "Manage auditing and security log"),
    right("SeRelabelPrivilege", "Modify an object label"),
    right("SeSystemEnvironmentPrivilege", "Modify firmware environment values"),
    right("SeDelegateSessionUserImpersonatePrivilege", "Obtain an impersonation token for another user in the same session"),
    right("SeManageVolumePrivilege", "Perform volume maintenance tasks"),
    right("SeProfileSingleProcessPrivilege", "Profile single process"),
    right("SeSystemProfilePrivilege", "Profile system performance"),
    right("SeUndockPrivilege", "Remove computer from docking station"),
    right("SeAssignPrimaryTokenPrivilege", "Replace a process level token"),
    right("SeRestorePrivilege", "Restore files and directories"),
    right("SeShutdownPrivilege", "Shut down the system"),
    right("SeSyncAgentPrivilege", "Synchronize directory service data"),
    right("SeTakeOwnershipPrivilege", "Take ownership of files or other objects"),
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
        let names: HashSet<&str> = USER_RIGHTS.iter().map(|r| r.display_name).collect();
        assert_eq!(names.len(), 45, "display names must be unique");
        for entry in &USER_RIGHTS {
            assert!(entry.constant.starts_with("Se"), "{}", entry.constant);
            assert!(
                entry.constant.ends_with("Privilege") || entry.constant.ends_with("Right"),
                "{}",
                entry.constant
            );
            assert!(!entry.display_name.is_empty(), "{}", entry.constant);
        }
    }

    #[test]
    fn find_ignores_case_and_whitespace() {
        assert_eq!(find("SeServiceLogonRight").map(|r| r.display_name), Some("Log on as a service"));
        assert_eq!(find("  seservicelogonright ").map(|r| r.constant), Some("SeServiceLogonRight"));
        assert!(find("SeBogusPrivilege").is_none());
        assert!(find("Log on as a service").is_none());
        assert!(find("").is_none());
    }
}
