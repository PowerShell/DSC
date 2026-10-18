// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Portable, resource-agnostic typed model for an access control list (ACL) expressed as
//! arrays of access control entry (ACE) objects. This module has no Win32 dependency so that
//! the shape of the model (and its JSON schema) is available on every platform; the Win32
//! operations that actually read and write ACLs live in [`crate::win32`] and are only compiled
//! on Windows.

use crate::error::AclError;
use rust_i18n::t;
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

    /// Maps generic access rights in `bits` to the resource-specific rights Windows uses when
    /// evaluating access. The default preserves every bit for resource types without a generic
    /// mapping.
    fn map_generic_bits(bits: u32) -> u32 {
        bits
    }
}

/// Combines a list of access mask values into a single raw bitmask.
#[must_use]
pub fn mask_to_bits<P: AccessMask>(values: &[P]) -> u32 {
    values.iter().fold(0_u32, |mask, value| mask | value.bits())
}

fn effective_mask_bits<P: AccessMask>(values: &[P]) -> u32 {
    P::map_generic_bits(mask_to_bits(values))
}

/// Decomposes a raw bitmask into a list of access mask values. Candidates with more bits are
/// preferred so that shorthand aliases (for example `FullControl`) are chosen over a long list
/// of individual atomic rights when they fit exactly.
///
/// Every bit in `bits` must be accounted for by the values this decomposition selects; bits that
/// don't correspond to any known value are never silently discarded; see Errors below. When this
/// function succeeds, the selected values' bits, recombined with [`mask_to_bits`], are guaranteed
/// to reproduce `bits` exactly.
///
/// # Errors
///
/// Returns [`AclError::UnrepresentableAccessMask`] when `bits` contains one or more bits that
/// don't correspond to any value in `P`. Silently dropping such bits would let a subsequent
/// merge-and-rewrite of the access control list narrow its effective access mask without any
/// indication that it had done so, so callers must surface this error (for example by failing
/// the read, or the write it would otherwise feed into) rather than treat the decomposition as
/// complete.
pub fn bits_to_mask<P: AccessMask>(bits: u32) -> Result<Vec<P>, AclError> {
    if bits == 0 {
        return Ok(Vec::new());
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
    if remaining != 0 {
        return Err(AclError::UnrepresentableAccessMask(remaining));
    }
    Ok(result)
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
                t!("validation.discretionaryType").to_string(),
            ));
        }
        if !self.audit_flags.is_empty() {
            return Err(AclError::InvalidEntry(
                t!("validation.discretionaryAuditFlags").to_string(),
            ));
        }
        if self.access_mask.is_empty() {
            return Err(AclError::InvalidEntry(
                t!("validation.accessMaskRequired").to_string(),
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
                t!("validation.systemType").to_string(),
            ));
        }
        if self.audit_flags.is_empty() {
            return Err(AclError::InvalidEntry(
                t!("validation.systemAuditFlags").to_string(),
            ));
        }
        if self.access_mask.is_empty() {
            return Err(AclError::InvalidEntry(
                t!("validation.accessMaskRequired").to_string(),
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
    /// Whether the DACL is protected from inheriting entries from its parent. Only meaningful
    /// when `discretionary` is also provided; has no effect otherwise. Omitting this property
    /// behaves according to `_purge`: when purging (the default), the DACL is written as
    /// unprotected (inheriting), since a purge write always stamps an explicit protection state;
    /// when merging (`_purge: false`), the current protection state is preserved unchanged, since
    /// a merge leaves everything it doesn't explicitly touch alone. Set this explicitly to
    /// request a particular protection state regardless of `_purge`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discretionary_protected: Option<bool>,
    /// The system access control list (SACL) entries. Omit to leave the SACL unmanaged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Vec<AccessControlEntry<P>>>,
    /// Whether the SACL is protected from inheriting entries from its parent. Only meaningful
    /// when `system` is also provided; has no effect otherwise. Omitting this property behaves
    /// according to `_purge`: when purging (the default), the SACL is written as unprotected
    /// (inheriting), since a purge write always stamps an explicit protection state; when
    /// merging (`_purge: false`), the current protection state is preserved unchanged, since a
    /// merge leaves everything it doesn't explicitly touch alone. Set this explicitly to request
    /// a particular protection state regardless of `_purge`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_protected: Option<bool>,
    /// Whether to replace (purge) each supplied section outright, or merge its entries into the
    /// section's current managed entries. Defaults to `true` (replace) when omitted, which
    /// preserves the historical full-replacement behavior. When `false`, the entries supplied
    /// for a section are merged into that section's existing entries: semantic duplicates (same
    /// normalized principal, ACE type, effective access mask, and inheritance/audit flags) are
    /// not duplicated, and the resulting discretionary section is canonicalized into Windows'
    /// canonical ACE order: every explicit `deny` entry, then every explicit `allow` entry, then
    /// every inherited entry (preserving its existing relative order) -- an inherited `deny`
    /// entry is never moved ahead of an explicit `allow` entry. A section that is omitted from
    /// the request remains unmanaged regardless of this setting.
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
/// implies, still compare equal), equal inheritance flags, and equal audit flags. Inherited and
/// explicit entries remain distinct because Windows applies and orders them differently.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AceSemanticKey {
    principal: String,
    ace_type: AceType,
    mask_bits: u32,
    flags: Vec<AceFlag>,
    audit_flags: Vec<AuditFlag>,
}

/// Sorts and dedups a copy of `flags`, so that two flag lists that set the same flags compare
/// equal regardless of the order they were specified or read back in.
fn normalized_flags(flags: &[AceFlag]) -> Vec<AceFlag> {
    let mut flags = flags.to_vec();
    flags.sort_unstable();
    flags.dedup();
    flags
}

/// Sorts and dedups a copy of `flags`, so that two audit flag lists that set the same flags
/// compare equal regardless of the order they were specified or read back in.
fn normalized_audit_flags(flags: &[AuditFlag]) -> Vec<AuditFlag> {
    let mut flags = flags.to_vec();
    flags.sort_unstable();
    flags.dedup();
    flags
}

fn semantic_key<P: AccessMask>(entry: &AccessControlEntry<P>) -> AceSemanticKey {
    AceSemanticKey {
        principal: entry.principal.clone(),
        ace_type: entry.ace_type,
        mask_bits: effective_mask_bits(&entry.access_mask),
        flags: normalized_flags(&entry.flags),
        audit_flags: normalized_audit_flags(&entry.audit_flags),
    }
}

/// Returns whether `a` and `b` are the same access control entry in substance: an equal
/// principal, an equal ACE type, the same *effective* access mask (decomposed to raw bits, so
/// that aliased permission names such as a `fullControl` shorthand compare equal to the
/// individual rights it implies), and the same normalized (order-independent) sets of
/// inheritance flags and audit flags.
fn entries_semantically_equal<P: AccessMask>(
    a: &AccessControlEntry<P>,
    b: &AccessControlEntry<P>,
) -> bool {
    a.principal == b.principal
        && a.ace_type == b.ace_type
        && effective_mask_bits(&a.access_mask) == effective_mask_bits(&b.access_mask)
        && normalized_flags(&a.flags) == normalized_flags(&b.flags)
        && normalized_audit_flags(&a.audit_flags) == normalized_audit_flags(&b.audit_flags)
}

/// Returns whether `desired` and `actual` contain the same access control entries in the same
/// order (see [`entries_semantically_equal`] for what "same" means for a single entry pair).
/// Used to compare a `_purge: true` section, which (unlike a `_purge: false` merge) must match
/// position-for-position: a purge write replaces the section outright, verbatim and in the
/// order given, so convergence requires the same count, order, and inherited status throughout,
/// not merely an equivalent unordered set of entries.
fn entries_match_in_order<P: AccessMask>(
    desired: &[AccessControlEntry<P>],
    actual: &[AccessControlEntry<P>],
) -> bool {
    desired.len() == actual.len()
        && desired
            .iter()
            .zip(actual.iter())
            .all(|(desired, actual)| entries_semantically_equal(desired, actual))
}

/// Merges `requested` access control entries into `current`, dropping any requested entry that
/// is a semantic duplicate (see [`AceSemanticKey`]) of an entry already present in `current` or
/// earlier in `requested`, and then canonicalizing the result into Windows' canonical ACE order:
/// every explicit (non-inherited) `deny` entry, then every explicit `allow`/`audit` entry, then
/// every inherited entry, preserving the relative order of entries within each of those three
/// groups. In particular, an inherited `deny` entry is never moved ahead of an explicit `allow`
/// entry, since Windows always places explicit entries before inherited ones regardless of
/// allow/deny. Both inputs are expected to already have normalized (for example resolved-SID)
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
    // `sort_by_key` is a stable sort, so entries within each of the three groups below keep
    // their relative order.
    merged.sort_by_key(|entry| canonical_ace_rank(entry));
    merged
}

/// The sort rank used by [`merge_entries`] to canonicalize ACE order: explicit `deny` entries
/// sort first, explicit `allow`/`audit` entries second, and every inherited entry (`deny` or
/// `allow`/`audit` alike) last, regardless of its own type.
fn canonical_ace_rank<P: AccessMask>(entry: &AccessControlEntry<P>) -> u8 {
    if entry.flags.contains(&AceFlag::Inherited) {
        2
    } else if entry.ace_type == AceType::Deny {
        0
    } else {
        1
    }
}

/// Computes the effective access control list that should be written (or compared against) for
/// `requested`, given its [`AccessControlList::purge`] setting and the `current` state of the
/// section(s) `requested` manages. Both `current` and `requested` are expected to already have
/// normalized (for example resolved-SID) principals.
///
/// When `purge` is `true` (the default), each supplied section is replaced outright: the result
/// is `requested`'s section verbatim. A protection flag that is not explicitly supplied in
/// `requested` defaults to `false` in this case, matching the real effect of writing a section
/// with no protection state requested (see [`crate::win32::build_self_relative_security_descriptor`]):
/// the write always stamps an explicit protected/unprotected state, so a desired-state comparison
/// must not treat the omitted flag as "don't care" and silently accept a protected or unprotected
/// `actual` state at random. When `purge` is `false`, each supplied section is merged with the
/// corresponding section of `current` (see [`merge_entries`]); a protection flag that is not
/// explicitly supplied in `requested` instead preserves `current`'s value, since a merge leaves
/// everything it doesn't explicitly touch alone.
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
            // A purge write always stamps an explicit protection state, so the omitted flag
            // resolves to its effective `false` default rather than staying unresolved.
            Some(requested.discretionary_protected.unwrap_or(false))
        } else {
            // A merge leaves everything it doesn't explicitly touch alone: fall back to
            // `current`'s value, which may itself be `None` (unknown) rather than `false`.
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
            // A purge write always stamps an explicit protection state, so the omitted flag
            // resolves to its effective `false` default rather than staying unresolved.
            Some(requested.system_protected.unwrap_or(false))
        } else {
            // A merge leaves everything it doesn't explicitly touch alone: fall back to
            // `current`'s value, which may itself be `None` (unknown) rather than `false`.
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
/// than only the partial, newly requested entries, and a protection flag omitted from the
/// original request has already been resolved to its effective value).
///
/// A section that is `None` in `desired` is always satisfied (it is unmanaged). When `desired`'s
/// `purge` is `true` (the default), a managed section must match `actual` entry-for-entry, in
/// the same order and with the same count and inherited status throughout (since a purge write
/// replaces the section outright, verbatim and in the order given) -- but each entry pair is
/// compared *semantically* (see [`entries_semantically_equal`]), so an aliased permission name
/// (for example `fullControl` versus the individual rights it implies) or a differently ordered
/// flag list that is otherwise equivalent does not cause a false mismatch. When `false`, it is
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
        entries_match_in_order(desired_entries, actual_entries)
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
        GenericAll,
    }

    impl AccessMask for TestRight {
        fn bits(self) -> u32 {
            match self {
                Self::Read => 0x1,
                Self::Write => 0x2,
                // Aliases the combination of Read and Write, the same way `fullControl` aliases
                // several atomic rights for real resources.
                Self::FullControl => 0x3,
                Self::GenericAll => 0x100,
            }
        }

        fn all() -> &'static [Self] {
            &[Self::Read, Self::Write, Self::FullControl, Self::GenericAll]
        }

        fn map_generic_bits(bits: u32) -> u32 {
            if bits & Self::GenericAll.bits() == 0 {
                bits
            } else {
                (bits & !Self::GenericAll.bits()) | Self::FullControl.bits()
            }
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
    fn merge_drops_duplicates_with_equivalent_generic_access_masks() {
        let current = vec![entry(
            "S-1-1-0",
            AceType::Allow,
            vec![TestRight::FullControl],
        )];
        let requested = vec![entry(
            "S-1-1-0",
            AceType::Allow,
            vec![TestRight::GenericAll],
        )];

        let merged = merge_entries(&current, &requested);

        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn merge_keeps_explicit_and_inherited_entries_distinct() {
        let mut inherited = entry("S-1-1-0", AceType::Allow, vec![TestRight::Read]);
        inherited.flags = vec![AceFlag::Inherited];
        let current = vec![inherited];
        let requested = vec![entry("S-1-1-0", AceType::Allow, vec![TestRight::Read])];

        let merged = merge_entries(&current, &requested);

        assert_eq!(merged.len(), 2);
        assert!(!merged[0].flags.contains(&AceFlag::Inherited));
        assert!(merged[1].flags.contains(&AceFlag::Inherited));
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

    #[test]
    fn purge_true_defaults_protected_to_false_when_omitted() {
        // `current` is protected, but a purge write always stamps an explicit protection
        // state, so omitting it in `requested` must reset the effective state to false rather
        // than silently inheriting `current`'s protected state.
        let current = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-0", AceType::Allow, vec![TestRight::Read])]),
            discretionary_protected: Some(true),
            ..Default::default()
        };
        let requested = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-1", AceType::Allow, vec![TestRight::Write])]),
            purge: Some(true),
            ..Default::default()
        };

        let effective = merge_access_control_list(&current, &requested);

        assert_eq!(effective.discretionary_protected, Some(false));
    }

    #[test]
    fn is_satisfied_is_false_when_purge_true_omits_protected_but_actual_is_protected() {
        let actual = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-0", AceType::Allow, vec![TestRight::Read])]),
            discretionary_protected: Some(true),
            ..Default::default()
        };
        let desired = AccessControlList {
            discretionary: Some(vec![entry("S-1-1-0", AceType::Allow, vec![TestRight::Read])]),
            purge: Some(true),
            ..Default::default()
        };
        let effective_desired = merge_access_control_list(&actual, &desired);

        // The omitted protection flag defaults to false under purge, so a currently-protected
        // actual state must not be reported as satisfied.
        assert!(!is_satisfied(&effective_desired, &actual));
    }

    #[test]
    fn merge_keeps_explicit_entries_before_inherited_entries_preserving_inherited_order() {
        let mut inherited_allow = entry("S-1-1-0", AceType::Allow, vec![TestRight::Read]);
        inherited_allow.flags = vec![AceFlag::Inherited];
        let mut inherited_deny = entry("S-1-1-1", AceType::Deny, vec![TestRight::Write]);
        inherited_deny.flags = vec![AceFlag::Inherited];
        // `current` reflects the real, already-canonical order read from the system: the
        // inherited allow entry happens to appear before the inherited deny entry.
        let current = vec![inherited_allow, inherited_deny];
        let requested = vec![entry("S-1-1-2", AceType::Allow, vec![TestRight::Read])];

        let merged = merge_entries(&current, &requested);

        let principals: Vec<&str> = merged.iter().map(|entry| entry.principal.as_str()).collect();
        // The explicit allow entry comes first (explicit entries always precede inherited
        // ones), and the two inherited entries keep their original relative order: the
        // inherited deny is NOT moved ahead of the explicit allow, even though deny entries are
        // otherwise ordered before allow entries among explicit entries.
        assert_eq!(principals, vec!["S-1-1-2", "S-1-1-0", "S-1-1-1"]);
    }

    #[test]
    fn bits_to_mask_errors_on_unrepresentable_bits() {
        // 0x8 doesn't correspond to any `TestRight` value.
        let result = bits_to_mask::<TestRight>(0x8);
        assert!(matches!(result, Err(AclError::UnrepresentableAccessMask(0x8))));
    }

    #[test]
    fn bits_to_mask_round_trips_known_bits() {
        let decomposed = bits_to_mask::<TestRight>(0x3).unwrap();
        assert_eq!(mask_to_bits(&decomposed), 0x3);
    }

    #[test]
    fn is_satisfied_purge_true_treats_alias_and_decomposed_mask_as_equal() {
        // `FullControl` (0x3) is the same effective mask as `Read` (0x1) + `Write` (0x2).
        let desired = AccessControlList {
            discretionary: Some(vec![entry(
                "S-1-1-0",
                AceType::Allow,
                vec![TestRight::FullControl],
            )]),
            discretionary_protected: Some(false),
            purge: Some(true),
            ..Default::default()
        };
        let actual = AccessControlList {
            discretionary: Some(vec![entry(
                "S-1-1-0",
                AceType::Allow,
                vec![TestRight::Read, TestRight::Write],
            )]),
            discretionary_protected: Some(false),
            ..Default::default()
        };

        assert!(is_satisfied(&desired, &actual));
    }

    #[test]
    fn is_satisfied_purge_true_treats_differently_ordered_flags_as_equal() {
        let mut desired_entry = entry("S-1-1-0", AceType::Allow, vec![TestRight::Read]);
        desired_entry.flags = vec![AceFlag::ContainerInherit, AceFlag::ObjectInherit];
        let mut actual_entry = entry("S-1-1-0", AceType::Allow, vec![TestRight::Read]);
        actual_entry.flags = vec![AceFlag::ObjectInherit, AceFlag::ContainerInherit];
        let desired = AccessControlList {
            discretionary: Some(vec![desired_entry]),
            discretionary_protected: Some(false),
            purge: Some(true),
            ..Default::default()
        };
        let actual = AccessControlList {
            discretionary: Some(vec![actual_entry]),
            discretionary_protected: Some(false),
            ..Default::default()
        };

        assert!(is_satisfied(&desired, &actual));
    }

    #[test]
    fn is_satisfied_purge_true_still_requires_matching_order_and_count() {
        let allow = entry("S-1-1-0", AceType::Allow, vec![TestRight::Read]);
        let deny = entry("S-1-1-1", AceType::Deny, vec![TestRight::Write]);
        let desired = AccessControlList {
            discretionary: Some(vec![allow.clone(), deny.clone()]),
            discretionary_protected: Some(false),
            purge: Some(true),
            ..Default::default()
        };
        // Same two entries, but in a different order: a purge write would produce the exact
        // order given, so a differently ordered `actual` is not an equivalent, converged state
        // even though the same entries (by alias-aware comparison) are all present.
        let actual = AccessControlList {
            discretionary: Some(vec![deny, allow]),
            discretionary_protected: Some(false),
            ..Default::default()
        };

        assert!(!is_satisfied(&desired, &actual));
    }

    #[test]
    fn is_satisfied_purge_true_distinguishes_inherited_status() {
        let desired_entry = entry("S-1-1-0", AceType::Allow, vec![TestRight::Read]);
        let mut actual_entry = entry("S-1-1-0", AceType::Allow, vec![TestRight::Read]);
        actual_entry.flags = vec![AceFlag::Inherited];
        let desired = AccessControlList {
            discretionary: Some(vec![desired_entry]),
            discretionary_protected: Some(false),
            purge: Some(true),
            ..Default::default()
        };
        let actual = AccessControlList {
            discretionary: Some(vec![actual_entry]),
            discretionary_protected: Some(false),
            ..Default::default()
        };

        // A purge write never produces an inherited entry; an inherited `actual` entry does
        // not satisfy a `desired` entry that isn't marked as inherited, even though every other
        // aspect (principal, type, effective mask) matches.
        assert!(!is_satisfied(&desired, &actual));
    }
}
