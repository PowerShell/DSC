// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Static table of the Windows advanced audit policy categories and subcategories.
//!
//! The Win32 audit APIs identify categories and subcategories by GUID and only return
//! localized display names. This table keeps the resource locale-independent: input is
//! matched against the English names (or a GUID) and output always uses the English names.
//! GUIDs are stored as `u128` in the byte order that `windows::core::GUID::from_u128` expects,
//! so `{0CCE9215-69AE-11D9-BED3-505054503030}` is `0x0CCE9215_69AE_11D9_BED3_505054503030`.

use std::fmt::Write as _;

/// An audit policy category.
#[derive(Debug, PartialEq, Eq)]
pub struct Category {
    /// English category name as shown by `auditpol /list /category`.
    pub name: &'static str,
    /// Category GUID.
    pub guid: u128,
}

/// An audit policy subcategory.
#[derive(Debug, PartialEq, Eq)]
pub struct Subcategory {
    /// English subcategory name as shown by `auditpol /list /subcategory:*`.
    pub name: &'static str,
    /// Subcategory GUID.
    pub guid: u128,
    /// The category that contains this subcategory.
    pub category: &'static Category,
}

/// Low 96 bits shared by every subcategory GUID (`-69AE-11D9-BED3-505054503030`).
const SUBCATEGORY_SUFFIX: u128 = 0x69AE_11D9_BED3_5050_5450_3030;
/// Low 96 bits shared by every category GUID (`-797A-11D9-BED3-505054503030`).
const CATEGORY_SUFFIX: u128 = 0x797A_11D9_BED3_5050_5450_3030;

/// Build a subcategory GUID from its distinguishing first 32 bits.
#[allow(clippy::cast_lossless)] // `u128::from` cannot be used in a const fn
const fn subcategory_guid(prefix: u32) -> u128 {
    ((prefix as u128) << 96) | SUBCATEGORY_SUFFIX
}

/// Build a category GUID from its distinguishing first 32 bits.
#[allow(clippy::cast_lossless)] // `u128::from` cannot be used in a const fn
const fn category_guid(prefix: u32) -> u128 {
    ((prefix as u128) << 96) | CATEGORY_SUFFIX
}

pub static SYSTEM: Category = Category { name: "System", guid: category_guid(0x6997_9848) };
pub static LOGON_LOGOFF: Category = Category { name: "Logon/Logoff", guid: category_guid(0x6997_9849) };
pub static OBJECT_ACCESS: Category = Category { name: "Object Access", guid: category_guid(0x6997_984A) };
pub static PRIVILEGE_USE: Category = Category { name: "Privilege Use", guid: category_guid(0x6997_984B) };
pub static DETAILED_TRACKING: Category = Category { name: "Detailed Tracking", guid: category_guid(0x6997_984C) };
pub static POLICY_CHANGE: Category = Category { name: "Policy Change", guid: category_guid(0x6997_984D) };
pub static ACCOUNT_MANAGEMENT: Category = Category { name: "Account Management", guid: category_guid(0x6997_984E) };
pub static DS_ACCESS: Category = Category { name: "DS Access", guid: category_guid(0x6997_984F) };
pub static ACCOUNT_LOGON: Category = Category { name: "Account Logon", guid: category_guid(0x6997_9850) };

/// All known categories in the order `auditpol /list /subcategory:* /v` reports them.
pub static CATEGORIES: [&Category; 9] = [
    &SYSTEM,
    &LOGON_LOGOFF,
    &OBJECT_ACCESS,
    &PRIVILEGE_USE,
    &DETAILED_TRACKING,
    &POLICY_CHANGE,
    &ACCOUNT_MANAGEMENT,
    &DS_ACCESS,
    &ACCOUNT_LOGON,
];

macro_rules! subcategory {
    ($name:literal, $prefix:literal, $category:ident) => {
        Subcategory {
            name: $name,
            guid: subcategory_guid($prefix),
            category: &$category,
        }
    };
}

/// All known subcategories, grouped by category.
pub static SUBCATEGORIES: [Subcategory; 60] = [
    // System
    subcategory!("Security State Change", 0x0CCE_9210, SYSTEM),
    subcategory!("Security System Extension", 0x0CCE_9211, SYSTEM),
    subcategory!("System Integrity", 0x0CCE_9212, SYSTEM),
    subcategory!("IPsec Driver", 0x0CCE_9213, SYSTEM),
    subcategory!("Other System Events", 0x0CCE_9214, SYSTEM),
    // Logon/Logoff
    subcategory!("Logon", 0x0CCE_9215, LOGON_LOGOFF),
    subcategory!("Logoff", 0x0CCE_9216, LOGON_LOGOFF),
    subcategory!("Account Lockout", 0x0CCE_9217, LOGON_LOGOFF),
    subcategory!("IPsec Main Mode", 0x0CCE_9218, LOGON_LOGOFF),
    subcategory!("IPsec Quick Mode", 0x0CCE_9219, LOGON_LOGOFF),
    subcategory!("IPsec Extended Mode", 0x0CCE_921A, LOGON_LOGOFF),
    subcategory!("Special Logon", 0x0CCE_921B, LOGON_LOGOFF),
    subcategory!("Other Logon/Logoff Events", 0x0CCE_921C, LOGON_LOGOFF),
    subcategory!("Network Policy Server", 0x0CCE_9243, LOGON_LOGOFF),
    subcategory!("User / Device Claims", 0x0CCE_9247, LOGON_LOGOFF),
    subcategory!("Group Membership", 0x0CCE_9249, LOGON_LOGOFF),
    subcategory!("Access Rights", 0x0CCE_924B, LOGON_LOGOFF),
    // Object Access
    subcategory!("File System", 0x0CCE_921D, OBJECT_ACCESS),
    subcategory!("Registry", 0x0CCE_921E, OBJECT_ACCESS),
    subcategory!("Kernel Object", 0x0CCE_921F, OBJECT_ACCESS),
    subcategory!("SAM", 0x0CCE_9220, OBJECT_ACCESS),
    subcategory!("Certification Services", 0x0CCE_9221, OBJECT_ACCESS),
    subcategory!("Application Generated", 0x0CCE_9222, OBJECT_ACCESS),
    subcategory!("Handle Manipulation", 0x0CCE_9223, OBJECT_ACCESS),
    subcategory!("File Share", 0x0CCE_9224, OBJECT_ACCESS),
    subcategory!("Filtering Platform Packet Drop", 0x0CCE_9225, OBJECT_ACCESS),
    subcategory!("Filtering Platform Connection", 0x0CCE_9226, OBJECT_ACCESS),
    subcategory!("Other Object Access Events", 0x0CCE_9227, OBJECT_ACCESS),
    subcategory!("Detailed File Share", 0x0CCE_9244, OBJECT_ACCESS),
    subcategory!("Removable Storage", 0x0CCE_9245, OBJECT_ACCESS),
    subcategory!("Central Policy Staging", 0x0CCE_9246, OBJECT_ACCESS),
    // Privilege Use
    subcategory!("Sensitive Privilege Use", 0x0CCE_9228, PRIVILEGE_USE),
    subcategory!("Non Sensitive Privilege Use", 0x0CCE_9229, PRIVILEGE_USE),
    subcategory!("Other Privilege Use Events", 0x0CCE_922A, PRIVILEGE_USE),
    // Detailed Tracking
    subcategory!("Process Creation", 0x0CCE_922B, DETAILED_TRACKING),
    subcategory!("Process Termination", 0x0CCE_922C, DETAILED_TRACKING),
    subcategory!("DPAPI Activity", 0x0CCE_922D, DETAILED_TRACKING),
    subcategory!("RPC Events", 0x0CCE_922E, DETAILED_TRACKING),
    subcategory!("Plug and Play Events", 0x0CCE_9248, DETAILED_TRACKING),
    subcategory!("Token Right Adjusted Events", 0x0CCE_924A, DETAILED_TRACKING),
    // Policy Change
    subcategory!("Audit Policy Change", 0x0CCE_922F, POLICY_CHANGE),
    subcategory!("Authentication Policy Change", 0x0CCE_9230, POLICY_CHANGE),
    subcategory!("Authorization Policy Change", 0x0CCE_9231, POLICY_CHANGE),
    subcategory!("MPSSVC Rule-Level Policy Change", 0x0CCE_9232, POLICY_CHANGE),
    subcategory!("Filtering Platform Policy Change", 0x0CCE_9233, POLICY_CHANGE),
    subcategory!("Other Policy Change Events", 0x0CCE_9234, POLICY_CHANGE),
    // Account Management
    subcategory!("User Account Management", 0x0CCE_9235, ACCOUNT_MANAGEMENT),
    subcategory!("Computer Account Management", 0x0CCE_9236, ACCOUNT_MANAGEMENT),
    subcategory!("Security Group Management", 0x0CCE_9237, ACCOUNT_MANAGEMENT),
    subcategory!("Distribution Group Management", 0x0CCE_9238, ACCOUNT_MANAGEMENT),
    subcategory!("Application Group Management", 0x0CCE_9239, ACCOUNT_MANAGEMENT),
    subcategory!("Other Account Management Events", 0x0CCE_923A, ACCOUNT_MANAGEMENT),
    // DS Access
    subcategory!("Directory Service Access", 0x0CCE_923B, DS_ACCESS),
    subcategory!("Directory Service Changes", 0x0CCE_923C, DS_ACCESS),
    subcategory!("Directory Service Replication", 0x0CCE_923D, DS_ACCESS),
    subcategory!("Detailed Directory Service Replication", 0x0CCE_923E, DS_ACCESS),
    // Account Logon
    subcategory!("Credential Validation", 0x0CCE_923F, ACCOUNT_LOGON),
    subcategory!("Kerberos Service Ticket Operations", 0x0CCE_9240, ACCOUNT_LOGON),
    subcategory!("Other Account Logon Events", 0x0CCE_9241, ACCOUNT_LOGON),
    subcategory!("Kerberos Authentication Service", 0x0CCE_9242, ACCOUNT_LOGON),
];

/// Find a subcategory by its English name, ignoring case and surrounding whitespace.
#[must_use]
pub fn find_by_name(name: &str) -> Option<&'static Subcategory> {
    let name = name.trim();
    SUBCATEGORIES.iter().find(|entry| entry.name.eq_ignore_ascii_case(name))
}

/// Find a subcategory by its GUID.
#[must_use]
pub fn find_by_guid(guid: u128) -> Option<&'static Subcategory> {
    SUBCATEGORIES.iter().find(|entry| entry.guid == guid)
}

/// Find the English name of a category by its GUID.
#[must_use]
pub fn category_name(guid: u128) -> Option<&'static str> {
    CATEGORIES.iter().find(|entry| entry.guid == guid).map(|entry| entry.name)
}

/// Parse a GUID written as `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`, with or without braces,
/// in either case. Returns `None` for anything that is not a well-formed GUID.
#[must_use]
pub fn parse_guid(text: &str) -> Option<u128> {
    let trimmed = text.trim();
    let inner = match (trimmed.strip_prefix('{'), trimmed.strip_suffix('}')) {
        (Some(_), Some(_)) => &trimmed[1..trimmed.len() - 1],
        (None, None) => trimmed,
        _ => return None,
    };
    let parts: Vec<&str> = inner.split('-').collect();
    if parts.len() != 5 {
        return None;
    }
    const GROUP_LENGTHS: [usize; 5] = [8, 4, 4, 4, 12];
    for (part, expected) in parts.iter().zip(GROUP_LENGTHS) {
        if part.len() != expected || !part.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
    }
    u128::from_str_radix(&parts.concat(), 16).ok()
}

/// Format a GUID as `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` (braces, upper-case).
#[must_use]
pub fn format_guid(guid: u128) -> String {
    let bytes = guid.to_be_bytes();
    let mut text = String::with_capacity(38);
    text.push('{');
    for (index, byte) in bytes.iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            text.push('-');
        }
        // Writing to a String cannot fail.
        let _ = write!(text, "{byte:02X}");
    }
    text.push('}');
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const LOGON_GUID: u128 = 0x0CCE_9215_69AE_11D9_BED3_5050_5450_3030;
    const LOGON_TEXT: &str = "{0CCE9215-69AE-11D9-BED3-505054503030}";

    #[test]
    fn table_has_sixty_unique_entries() {
        assert_eq!(SUBCATEGORIES.len(), 60);
        let names: HashSet<String> = SUBCATEGORIES.iter().map(|s| s.name.to_ascii_lowercase()).collect();
        assert_eq!(names.len(), 60, "subcategory names must be unique");
        let guids: HashSet<u128> = SUBCATEGORIES.iter().map(|s| s.guid).collect();
        assert_eq!(guids.len(), 60, "subcategory GUIDs must be unique");
    }

    #[test]
    fn guids_share_the_expected_suffixes() {
        const LOW_96_BITS: u128 = (1 << 96) - 1;
        for entry in &SUBCATEGORIES {
            assert_eq!(entry.guid & LOW_96_BITS, SUBCATEGORY_SUFFIX, "{}", entry.name);
        }
        for category in CATEGORIES {
            assert_eq!(category.guid & LOW_96_BITS, CATEGORY_SUFFIX, "{}", category.name);
        }
        assert_eq!(CATEGORIES.len(), 9);
    }

    #[test]
    fn every_category_has_subcategories() {
        for category in CATEGORIES {
            assert!(
                SUBCATEGORIES.iter().any(|s| std::ptr::eq(s.category, category)),
                "{} has no subcategories",
                category.name
            );
        }
    }

    #[test]
    fn find_by_name_ignores_case_and_whitespace() {
        assert_eq!(find_by_name("Logon").map(|s| s.guid), Some(LOGON_GUID));
        assert_eq!(find_by_name("  lOgOn ").map(|s| s.guid), Some(LOGON_GUID));
        assert_eq!(find_by_name("Logon ").map(|s| s.category.name), Some("Logon/Logoff"));
        assert!(find_by_name("Logons").is_none());
        assert!(find_by_name("").is_none());
    }

    #[test]
    fn find_by_guid_and_category_name() {
        assert_eq!(find_by_guid(LOGON_GUID).map(|s| s.name), Some("Logon"));
        assert!(find_by_guid(0).is_none());
        assert_eq!(category_name(LOGON_LOGOFF.guid), Some("Logon/Logoff"));
        assert!(category_name(LOGON_GUID).is_none());
    }

    #[test]
    fn parse_guid_accepts_braced_bare_and_lowercase() {
        assert_eq!(parse_guid(LOGON_TEXT), Some(LOGON_GUID));
        assert_eq!(parse_guid("0CCE9215-69AE-11D9-BED3-505054503030"), Some(LOGON_GUID));
        assert_eq!(parse_guid("{0cce9215-69ae-11d9-bed3-505054503030}"), Some(LOGON_GUID));
        assert_eq!(parse_guid("  {0cce9215-69ae-11d9-bed3-505054503030}  "), Some(LOGON_GUID));
    }

    #[test]
    fn parse_guid_rejects_malformed_input() {
        assert!(parse_guid("Logon").is_none());
        assert!(parse_guid("").is_none());
        assert!(parse_guid("{}").is_none());
        assert!(parse_guid("{0CCE9215-69AE-11D9-BED3-505054503030").is_none());
        assert!(parse_guid("0CCE9215-69AE-11D9-BED3-505054503030}").is_none());
        assert!(parse_guid("0CCE921569AE11D9BED3505054503030").is_none());
        assert!(parse_guid("0CCE9215-69AE-11D9-BED3-50505450303").is_none());
        assert!(parse_guid("0CCE9215-69AE-11D9-BED3-50505450303G").is_none());
        assert!(parse_guid("0CCE9215-69AE-11D9-BED3-505054503030-00").is_none());
    }

    #[test]
    fn format_guid_round_trips() {
        assert_eq!(format_guid(LOGON_GUID), LOGON_TEXT);
        for entry in &SUBCATEGORIES {
            assert_eq!(parse_guid(&format_guid(entry.guid)), Some(entry.guid));
        }
        assert_eq!(format_guid(0), "{00000000-0000-0000-0000-000000000000}");
    }
}
