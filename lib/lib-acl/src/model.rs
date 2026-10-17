// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Portable, resource-agnostic typed model for an access control list (ACL) expressed as
//! arrays of access control entry (ACE) objects. This module has no Win32 dependency so that
//! the shape of the model (and its JSON schema) is available on every platform; the Win32
//! operations that actually read and write ACLs live in [`crate::win32`] and are only compiled
//! on Windows.

use crate::error::AclError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A resource-specific access mask permission. Each resource (for example files or registry
/// keys) defines its own enum of the permissions it understands and implements this trait so
/// that [`AccessControlEntry`] can store a typed, human-readable `accessMask` array instead of a
/// raw bitmask.
pub trait AccessMask: Sized + Copy + Clone + Eq + 'static {
    /// The raw access mask bit(s) represented by this value. Values may alias multiple bits
    /// (for example a `FullControl` shorthand).
    fn bits(self) -> u32;

    /// All known values, used to decompose a raw access mask back into a list of flags.
    fn all() -> &'static [Self];
}

/// Combines a list of access mask values into a single raw bitmask.
#[must_use]
pub fn mask_to_bits<P: AccessMask>(values: &[P]) -> u32 {
    values.iter().fold(0_u32, |mask, value| mask | value.bits())
}

/// Decomposes a raw bitmask into a list of access mask values. Candidates with more bits are
/// preferred so that shorthand aliases (for example `FullControl`) are chosen over a long list
/// of individual atomic rights when they fit exactly. Bits that do not correspond to any known
/// value are dropped; round-tripping a mask through [`mask_to_bits`] and back is therefore not
/// guaranteed to reproduce the original list, only an equivalent mask for the bits this resource
/// understands.
#[must_use]
pub fn bits_to_mask<P: AccessMask>(bits: u32) -> Vec<P> {
    if bits == 0 {
        return Vec::new();
    }

    let mut candidates: Vec<P> = P::all().to_vec();
    candidates.sort_by_key(|b| core::cmp::Reverse(b.bits().count_ones()));

    let mut remaining = bits;
    let mut result = Vec::new();
    for candidate in candidates {
        let candidate_bits = candidate.bits();
        if candidate_bits != 0 && remaining & candidate_bits == candidate_bits {
            result.push(candidate);
            remaining &= !candidate_bits;
        }
    }
    result
}

/// The kind of access control entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AceType {
    /// Grants the access mask to the principal. Valid only in a discretionary ACL.
    Allow,
    /// Denies the access mask to the principal. Valid only in a discretionary ACL.
    Deny,
    /// Generates audit events when the principal uses the access mask. Valid only in a system
    /// ACL.
    Audit,
}

/// Inheritance and provenance flags for an access control entry. These correspond to the
/// `AceFlags` byte shared by every ACE header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AceFlag {
    /// Containers (for example folders) beneath the object inherit the entry.
    ContainerInherit,
    /// Non-container objects (for example files) beneath the object inherit the entry.
    ObjectInherit,
    /// Inheritance does not propagate past the immediate children of the object.
    NoPropagateInherit,
    /// The entry does not apply to the object itself, only to objects that inherit it.
    InheritOnly,
    /// The entry was inherited from a parent container rather than set explicitly.
    Inherited,
}

/// Audit condition flags for a system (SACL) access control entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AuditFlag {
    /// Generate an audit event when the access succeeds.
    Success,
    /// Generate an audit event when the access fails.
    Failure,
}

/// A single access control entry (ACE).
///
/// `P` is the resource-specific access mask permission enum (for example the set of rights
/// understood by the filesystem or the registry).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccessControlEntry<P> {
    /// The security principal the entry applies to. Accepts either a SID string (for example
    /// `S-1-5-32-544`) or an account or group name (for example `DOMAIN\User` or
    /// `Administrators`). Names are resolved to a security identifier; output always reports
    /// the normalized SID string.
    pub principal: String,
    /// Whether the entry allows, denies, or audits access.
    #[serde(rename = "type")]
    pub ace_type: AceType,
    /// The permissions granted, denied, or audited by this entry.
    pub access_mask: Vec<P>,
    /// Inheritance flags for the entry. Defaults to an empty list (no inheritance).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flags: Vec<AceFlag>,
    /// The audit conditions for the entry. Required (and non-empty) when `type` is `audit`;
    /// must be empty otherwise.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audit_flags: Vec<AuditFlag>,
}

impl<P: AccessMask> AccessControlEntry<P> {
    /// Validates that this entry is well-formed for inclusion in a discretionary ACL (DACL).
    ///
    /// # Errors
    ///
    /// Returns [`AclError::InvalidEntry`] when the entry's type or flags are not valid for a
    /// discretionary ACL.
    pub fn validate_discretionary(&self) -> Result<(), AclError> {
        if !matches!(self.ace_type, AceType::Allow | AceType::Deny) {
            return Err(AclError::InvalidEntry(
                "discretionary access control entries must have type 'allow' or 'deny'".to_string(),
            ));
        }
        if !self.audit_flags.is_empty() {
            return Err(AclError::InvalidEntry(
                "discretionary access control entries must not set auditFlags".to_string(),
            ));
        }
        if self.access_mask.is_empty() {
            return Err(AclError::InvalidEntry(
                "an access control entry must specify at least one accessMask value".to_string(),
            ));
        }
        Ok(())
    }

    /// Validates that this entry is well-formed for inclusion in a system ACL (SACL).
    ///
    /// # Errors
    ///
    /// Returns [`AclError::InvalidEntry`] when the entry's type or flags are not valid for a
    /// system ACL.
    pub fn validate_system(&self) -> Result<(), AclError> {
        if self.ace_type != AceType::Audit {
            return Err(AclError::InvalidEntry(
                "system access control entries must have type 'audit'".to_string(),
            ));
        }
        if self.audit_flags.is_empty() {
            return Err(AclError::InvalidEntry(
                "system access control entries must set at least one auditFlags value".to_string(),
            ));
        }
        if self.access_mask.is_empty() {
            return Err(AclError::InvalidEntry(
                "an access control entry must specify at least one accessMask value".to_string(),
            ));
        }
        Ok(())
    }
}

/// The access control list (ACL) for an object, expressed as arrays of access control entries.
///
/// Both `discretionary` (DACL) and `system` (SACL) sections are optional so that callers can
/// select which section(s) they want to manage; a section that is omitted is left untouched.
///
/// `P` is the resource-specific access mask permission enum (for example the set of rights
/// understood by the filesystem or the registry).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccessControlList<P> {
    /// The discretionary access control list (DACL) entries. Omit to leave the DACL unmanaged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discretionary: Option<Vec<AccessControlEntry<P>>>,
    /// Whether the DACL is protected from inheriting entries from its parent. Omit to leave the
    /// current protection state unchanged when `discretionary` is not provided, or to request
    /// an unprotected (inheriting) DACL when it is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discretionary_protected: Option<bool>,
    /// The system access control list (SACL) entries. Omit to leave the SACL unmanaged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Vec<AccessControlEntry<P>>>,
    /// Whether the SACL is protected from inheriting entries from its parent. Omit to leave the
    /// current protection state unchanged when `system` is not provided, or to request an
    /// unprotected (inheriting) SACL when it is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_protected: Option<bool>,
    /// Whether to replace (purge) each supplied section outright, or merge its entries into the
    /// section's current managed entries. Defaults to `true` (replace) when omitted, which
    /// preserves the historical full-replacement behavior. When `false`, the entries supplied
    /// for a section are merged into that section's existing entries: semantic duplicates (same
    /// normalized principal, ACE type, effective access mask, and inheritance/audit flags) are
    /// not duplicated, and the resulting discretionary section is canonicalized with every
    /// `deny` entry before every `allow` entry. A section that is omitted from the request
    /// remains unmanaged regardless of this setting.
    #[serde(rename = "_purge", skip_serializing_if = "Option::is_none")]
    #[schemars(default = "default_purge")]
    pub purge: Option<bool>,
}

fn default_purge() -> Option<bool> {
    Some(true)
}

impl<P> Default for AccessControlList<P> {
    fn default() -> Self {
        Self {
            discretionary: None,
            discretionary_protected: None,
            system: None,
            system_protected: None,
            purge: None,
        }
    }
}

impl<P: AccessMask> AccessControlList<P> {
    /// Validates every entry present in the list against the section it belongs to.
    ///
    /// # Errors
    ///
    /// Returns [`AclError::InvalidEntry`] when any entry is not valid for its section.
    pub fn validate(&self) -> Result<(), AclError> {
        if let Some(entries) = &self.discretionary {
            for entry in entries {
                entry.validate_discretionary()?;
            }
        }
        if let Some(entries) = &self.system {
            for entry in entries {
                entry.validate_system()?;
            }
        }
        Ok(())
    }

    /// The effective value of `purge`: `true` (replace) when not explicitly specified.
    #[must_use]
    pub fn purge(&self) -> bool {
        self.purge.unwrap_or(true)
    }
}

/// A normalized key identifying whether two access control entries are semantic duplicates:
/// an equal principal (expected to already be normalized, for example a resolved SID string),
/// an equal ACE type, an equal effective access mask (decomposed to its raw bits so that
/// aliased permission names, such as a `fullControl` shorthand versus the individual rights it
/// implies, still compare equal), equal inheritance flags (ignoring [`AceFlag::Inherited`],
/// which reflects only provenance and is never meaningful in a request), and equal audit flags.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AceSemanticKey {
    principal: String,
    ace_type: AceType,
    mask_bits: u32,
    flags: Vec<AceFlag>,
    audit_flags: Vec<AuditFlag>,
}

fn semantic_key<P: AccessMask>(entry: &AccessControlEntry<P>) -> AceSemanticKey {
    let mut flags: Vec<AceFlag> = entry
        .flags
        .iter()
        .copied()
        .filter(|flag| *flag != AceFlag::Inherited)
        .collect();
    flags.sort_unstable();
    flags.dedup();

    let mut audit_flags = entry.audit_flags.clone();
    audit_flags.sort_unstable();
    audit_flags.dedup();

    AceSemanticKey {
        principal: entry.principal.clone(),
        ace_type: entry.ace_type,
        mask_bits: mask_to_bits(&entry.access_mask),
        flags,
        audit_flags,
    }
}

/// Merges `requested` access control entries into `current`, dropping any requested entry that
/// is a semantic duplicate (see [`AceSemanticKey`]) of an entry already present in `current` or
/// earlier in `requested`, and then canonicalizing the result so that every `deny` entry
/// precedes every `allow`/`audit` entry, preserving the relative order of entries within each
/// group. Both inputs are expected to already have normalized (for example resolved-SID)
/// principals.
#[must_use]
pub fn merge_entries<P: AccessMask>(
    current: &[AccessControlEntry<P>],
    requested: &[AccessControlEntry<P>],
) -> Vec<AccessControlEntry<P>> {
    let mut seen = HashSet::with_capacity(current.len() + requested.len());
    let mut merged = Vec::with_capacity(current.len() + requested.len());
    for entry in current.iter().chain(requested.iter()) {
        if seen.insert(semantic_key(entry)) {
            merged.push(entry.clone());
        }
    }
    // `sort_by_key` is a stable sort, so entries within the `deny` group and within the
    // `allow`/`audit` group keep their relative order.
    merged.sort_by_key(|entry| entry.ace_type != AceType::Deny);
    merged
}

/// Computes the effective access control list that should be written (or compared against) for
/// `requested`, given its [`AccessControlList::purge`] setting and the `current` state of the
/// section(s) `requested` manages. Both `current` and `requested` are expected to already have
/// normalized (for example resolved-SID) principals.
///
/// When `purge` is `true` (the default), each supplied section is replaced outright: the result
/// is `requested`'s section verbatim. When `purge` is `false`, each supplied section is merged
/// with the corresponding section of `current` (see [`merge_entries`]); a protection flag that
/// is not explicitly supplied in `requested` preserves `current`'s value instead of resetting it.
///
/// A section that is not present (`None`) in `requested` is left unmanaged and is always `None`
/// in the result, regardless of `current`.
#[must_use]
pub fn merge_access_control_list<P: AccessMask>(
    current: &AccessControlList<P>,
    requested: &AccessControlList<P>,
) -> AccessControlList<P> {
    let purge = requested.purge();

    let discretionary = requested.discretionary.as_ref().map(|entries| {
        if purge {
            entries.clone()
        } else {
            merge_entries(current.discretionary.as_deref().unwrap_or(&[]), entries)
        }
    });
    let discretionary_protected = requested.discretionary.as_ref().and_then(|_| {
        if purge {
            requested.discretionary_protected
        } else {
            requested
                .discretionary_protected
                .or(current.discretionary_protected)
        }
    });

    let system = requested.system.as_ref().map(|entries| {
        if purge {
            entries.clone()
        } else {
            merge_entries(current.system.as_deref().unwrap_or(&[]), entries)
        }
    });
    let system_protected = requested.system.as_ref().and_then(|_| {
        if purge {
            requested.system_protected
        } else {
            requested.system_protected.or(current.system_protected)
        }
    });

    AccessControlList {
        discretionary,
        discretionary_protected,
        system,
        system_protected,
        purge: requested.purge,
    }
}

/// Returns whether `actual` satisfies `desired`, an access control list that has already been
/// passed through [`merge_access_control_list`] (so that, when `desired`'s
/// [`purge`](AccessControlList::purge) is `false`, it reflects the full effective merge rather
/// than only the partial, newly requested entries).
///
/// A section that is `None` in `desired` is always satisfied (it is unmanaged). When `desired`'s
/// `purge` is `true` (the default), a managed section must match `actual` exactly, including
/// order, mirroring the full-replacement semantics used when writing it. When `false`, it is
/// enough for `actual` to contain a semantic duplicate (see [`AceSemanticKey`]) of every entry in
/// `desired`; additional entries present only in `actual` do not cause a mismatch. A protection
/// flag that is not explicitly supplied in `desired` is always considered satisfied.
#[must_use]
pub fn is_satisfied<P: AccessMask>(desired: &AccessControlList<P>, actual: &AccessControlList<P>) -> bool {
    let purge = desired.purge();
    section_satisfied(purge, &desired.discretionary, &actual.discretionary)
        && protected_satisfied(desired.discretionary_protected, actual.discretionary_protected)
        && section_satisfied(purge, &desired.system, &actual.system)
        && protected_satisfied(desired.system_protected, actual.system_protected)
}

fn section_satisfied<P: AccessMask>(
    purge: bool,
    desired: &Option<Vec<AccessControlEntry<P>>>,
    actual: &Option<Vec<AccessControlEntry<P>>>,
) -> bool {
    let Some(desired_entries) = desired else {
        return true;
    };
    let Some(actual_entries) = actual else {
        return false;
    };
    if purge {
        desired_entries == actual_entries
    } else {
        let desired_keys: HashSet<_> = desired_entries.iter().map(semantic_key).collect();
        let actual_keys: HashSet<_> = actual_entries.iter().map(semantic_key).collect();
        desired_keys == actual_keys
    }
}

fn protected_satisfied(desired: Option<bool>, actual: Option<bool>) -> bool {
    match desired {
        None => true,
        Some(value) => actual == Some(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestRight {
        Read,
        Write,
        FullControl,
    }

    impl AccessMask for TestRight {
        fn bits(self) -> u32 {
            match self {
                Self::Read => 0x1,
                Self::Write => 0x2,
                // Aliases the combination of Read and Write, the same way `fullControl` aliases
                // several atomic rights for real resources.
                Self::FullControl => 0x3,
            }
        }

        fn all() -> &'static [Self] {
            &[Self::Read, Self::Write, Self::FullControl]
        }
    }

    fn entry(
        principal: &str,
        ace_type: AceType,
        access_mask: Vec<TestRight>,
    ) -> AccessControlEntry<TestRight> {
        AccessControlEntry {
            principal: principal.to_string(),
            ace_type,
            access_mask,
            flags: Vec::new(),
            audit_flags: Vec::new(),
        }
    }

    #[test]
    fn purge_defaults_to_true_when_omitted() {
        let acl = AccessControlList::<TestRight> {
            discretionary: Some(Vec::new()),
            ..Default::default()
        };
        assert!(acl.purge.is_none());
        assert!(acl.purge());
    }

    #[test]
    fn purge_reflects_explicit_false() {
        let acl = AccessControlList::<TestRight> {
            discretionary: Some(Vec::new()),
            purge: Some(false),
            ..Default::default()
        };
        assert!(!acl.purge());
    }

    #[test]
    fn purge_true_replaces_the_section_outright() {
        let current = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-0", AceType::Allow, vec![TestRight::Read])]),
            ..Default::default()
        };
        let requested = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-1", AceType::Allow, vec![TestRight::Write])]),
            purge: Some(true),
            ..Default::default()
        };

        let effective = merge_access_control_list(&current, &requested);

        assert_eq!(effective.discretionary, requested.discretionary);
    }

    #[test]
    fn purge_false_merges_new_entries_with_current_ones() {
        let current = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-0", AceType::Allow, vec![TestRight::Read])]),
            ..Default::default()
        };
        let requested = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-1", AceType::Allow, vec![TestRight::Write])]),
            purge: Some(false),
            ..Default::default()
        };

        let effective = merge_access_control_list(&current, &requested);

        let principals: Vec<&str> = effective
            .discretionary
            .as_ref()
            .unwrap()
            .iter()
            .map(|entry| entry.principal.as_str())
            .collect();
        assert_eq!(principals, vec!["S-1-1-0", "S-1-1-1"]);
    }

    #[test]
    fn merge_drops_semantic_duplicates_by_effective_access_mask() {
        // FullControl (0x3) is the same effective mask as Read (0x1) + Write (0x2) combined, so
        // an entry requesting FullControl is a duplicate of two existing entries that together
        // grant the same bits... but a single current entry already granting FullControl must
        // also be recognized as a duplicate of a request for the equivalent alias.
        let current = vec![entry("S-1-1-0", AceType::Allow, vec![TestRight::FullControl])];
        let requested = vec![entry(
            "S-1-1-0",
            AceType::Allow,
            vec![TestRight::Read, TestRight::Write],
        )];

        let merged = merge_entries(&current, &requested);

        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn merge_drops_duplicates_differing_only_by_inherited_flag() {
        let mut inherited = entry("S-1-1-0", AceType::Allow, vec![TestRight::Read]);
        inherited.flags = vec![AceFlag::Inherited];
        let current = vec![inherited];
        // The same entry, requested explicitly (never carries the Inherited flag).
        let requested = vec![entry("S-1-1-0", AceType::Allow, vec![TestRight::Read])];

        let merged = merge_entries(&current, &requested);

        assert_eq!(merged.len(), 1);
        // The existing, inherited entry is preserved rather than being replaced.
        assert!(merged[0].flags.contains(&AceFlag::Inherited));
    }

    #[test]
    fn merge_orders_deny_entries_before_allow_entries_preserving_relative_order() {
        let current = vec![
            entry("S-1-1-0", AceType::Allow, vec![TestRight::Read]),
            entry("S-1-1-1", AceType::Deny, vec![TestRight::Write]),
        ];
        let requested = vec![
            entry("S-1-1-2", AceType::Allow, vec![TestRight::Read]),
            entry("S-1-1-3", AceType::Deny, vec![TestRight::Write]),
        ];

        let merged = merge_entries(&current, &requested);

        let principals: Vec<&str> = merged.iter().map(|entry| entry.principal.as_str()).collect();
        // Both deny entries precede both allow entries; within each group, entries keep their
        // original relative order (current before newly requested).
        assert_eq!(
            principals,
            vec!["S-1-1-1", "S-1-1-3", "S-1-1-0", "S-1-1-2"]
        );
    }

    #[test]
    fn is_satisfied_requires_exact_match_when_purge_is_true() {
        let actual = AccessControlList {
            discretionary: Some(vec![
                entry("S-1-1-0", AceType::Allow, vec![TestRight::Read]),
                entry("S-1-1-1", AceType::Allow, vec![TestRight::Write]),
            ]),
            ..Default::default()
        };
        let desired = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-0", AceType::Allow, vec![TestRight::Read])]),
            purge: Some(true),
            ..Default::default()
        };

        // Extra entries in `actual` are not tolerated when purge is true.
        assert!(!is_satisfied(&desired, &actual));
    }

    #[test]
    fn is_satisfied_allows_extra_entries_when_purge_is_false() {
        let actual = AccessControlList {
            discretionary: Some(vec![
                entry("S-1-1-0", AceType::Allow, vec![TestRight::Read]),
                entry("S-1-1-1", AceType::Allow, vec![TestRight::Write]),
            ]),
            ..Default::default()
        };
        let desired = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-0", AceType::Allow, vec![TestRight::Read])]),
            purge: Some(false),
            ..Default::default()
        };
        let effective_desired = merge_access_control_list(&actual, &desired);

        assert!(is_satisfied(&effective_desired, &actual));
    }

    #[test]
    fn is_satisfied_is_false_when_purge_is_false_and_an_entry_is_missing() {
        let actual = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-0", AceType::Allow, vec![TestRight::Read])]),
            ..Default::default()
        };
        let desired = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-1", AceType::Allow, vec![TestRight::Write])]),
            purge: Some(false),
            ..Default::default()
        };
        let effective_desired = merge_access_control_list(&actual, &desired);

        assert!(!is_satisfied(&effective_desired, &actual));
    }

    #[test]
    fn protection_flag_is_preserved_when_omitted_during_merge() {
        let current = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-0", AceType::Allow, vec![TestRight::Read])]),
            discretionary_protected: Some(true),
            ..Default::default()
        };
        let requested = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-1", AceType::Allow, vec![TestRight::Write])]),
            purge: Some(false),
            ..Default::default()
        };

        let effective = merge_access_control_list(&current, &requested);

        assert_eq!(effective.discretionary_protected, Some(true));
    }

    #[test]
    fn protection_flag_explicitly_supplied_overrides_current_during_merge() {
        let current = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-0", AceType::Allow, vec![TestRight::Read])]),
            discretionary_protected: Some(true),
            ..Default::default()
        };
        let requested = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-1", AceType::Allow, vec![TestRight::Write])]),
            discretionary_protected: Some(false),
            purge: Some(false),
            ..Default::default()
        };

        let effective = merge_access_control_list(&current, &requested);

        assert_eq!(effective.discretionary_protected, Some(false));
    }
}
