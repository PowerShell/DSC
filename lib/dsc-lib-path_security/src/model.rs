// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Platform neutral model of who may write to a path.
//!
//! The platform front ends translate a Windows security descriptor or a Unix mode into a
//! [`SecurityModel`]. The [`evaluate`] function then applies the trust rules of a
//! [`PathSecurityScope`] to that model. Keeping the rules separate from the operating system
//! calls makes them testable on every platform without special privileges.

use std::path::PathBuf;

use crate::{PathSecurityError, PathSecurityScope};

/// Classifies a principal by how the scopes trust it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrincipalKind {
    /// The local system account on Windows.
    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    LocalSystem,
    /// The built in Administrators group on Windows or the root user on Unix.
    Administrators,
    /// The account the current process runs as.
    CurrentUser,
    /// Any principal that no scope trusts.
    Other,
}

/// A principal that owns a path or appears in an access control entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Principal {
    /// How the scopes trust this principal.
    pub(crate) kind: PrincipalKind,
    /// The display form used in error messages, such as a SID string or a uid description.
    pub(crate) id: String,
}

impl Principal {
    /// Creates a principal.
    ///
    /// # Arguments
    ///
    /// 1. `kind`: how the scopes trust the principal.
    /// 1. `id`: the display form used in error messages.
    ///
    /// # Returns
    ///
    /// The new principal.
    pub(crate) fn new(kind: PrincipalKind, id: impl Into<String>) -> Self {
        Self { kind, id: id.into() }
    }
}

/// Whether an access control entry grants or denies access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryKind {
    /// The entry grants the rights in its mask.
    Allow,
    /// The entry denies the rights in its mask.
    Deny,
}

/// One access control entry reduced to the facts the trust rules need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AccessEntry {
    /// Whether the entry grants or denies access.
    pub(crate) kind: EntryKind,
    /// The principal the entry applies to.
    pub(crate) principal: Principal,
    /// Whether the entry includes any access right that conveys write.
    pub(crate) grants_write: bool,
    /// Whether the entry applies only to children and not to the path itself.
    pub(crate) inherit_only: bool,
}

/// The owner and access control entries of one path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SecurityModel {
    /// The path the model describes.
    pub(crate) path: PathBuf,
    /// The owner of the path.
    pub(crate) owner: Principal,
    /// The access control entries of the path in evaluation order.
    pub(crate) entries: Vec<AccessEntry>,
}

impl PathSecurityScope {
    /// Indicates whether this scope trusts a kind of principal to hold write access.
    ///
    /// # Arguments
    ///
    /// 1. `kind`: the kind of principal to check.
    ///
    /// # Returns
    ///
    /// `true` when the local system account or the Administrators group is checked, `true` for
    /// the current user when the scope is [`User`], and `false` otherwise.
    ///
    /// [`User`]: PathSecurityScope::User
    pub(crate) fn trusts(self, kind: PrincipalKind) -> bool {
        match kind {
            PrincipalKind::LocalSystem | PrincipalKind::Administrators => true,
            PrincipalKind::CurrentUser => self == PathSecurityScope::User,
            PrincipalKind::Other => false,
        }
    }
}

/// Applies the trust rules of a scope to a security model.
///
/// The rules are checked in this order:
///
/// 1. The owner must be a principal the scope trusts.
/// 1. Every allow entry that applies to the path itself and includes a write right must name a
///    principal the scope trusts. Inherit only entries and deny entries are ignored.
///
/// # Arguments
///
/// 1. `model`: the owner and entries of the path.
/// 1. `scope`: the scope whose trust rules apply.
///
/// # Errors
///
/// Returns [`PathSecurityError::UntrustedOwner`] when the owner is not trusted and
/// [`PathSecurityError::UntrustedWriter`] for the first entry that grants write access to an
/// untrusted principal.
pub(crate) fn evaluate(model: &SecurityModel, scope: PathSecurityScope) -> Result<(), PathSecurityError> {
    let path = model.path.display().to_string();
    if !scope.trusts(model.owner.kind) {
        return Err(PathSecurityError::UntrustedOwner {
            path,
            principal: model.owner.id.clone(),
        });
    }

    for entry in &model.entries {
        if entry.inherit_only || entry.kind == EntryKind::Deny || !entry.grants_write {
            continue;
        }
        if !scope.trusts(entry.principal.kind) {
            return Err(PathSecurityError::UntrustedWriter {
                path,
                principal: entry.principal.id.clone(),
            });
        }
    }

    Ok(())
}

/// Constants and classification for Windows access control entries.
///
/// The values are defined here instead of imported from the `windows` crate because the crate
/// gates them behind features the workspace does not enable, and defining them here keeps the
/// classification testable on every platform.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(crate) mod ace {
    use super::{AccessEntry, EntryKind, Principal};

    /// Right to write data to a file or to add a file to a folder.
    pub(crate) const FILE_WRITE_DATA: u32 = 0x0002;
    /// Right to append data to a file or to add a subfolder to a folder.
    pub(crate) const FILE_APPEND_DATA: u32 = 0x0004;
    /// Right to change the discretionary access control list.
    pub(crate) const WRITE_DAC: u32 = 0x0004_0000;
    /// Right to change the owner.
    pub(crate) const WRITE_OWNER: u32 = 0x0008_0000;
    /// Generic right that maps to the write rights of the object type.
    pub(crate) const GENERIC_WRITE: u32 = 0x4000_0000;
    /// Generic right that maps to every right of the object type.
    pub(crate) const GENERIC_ALL: u32 = 0x1000_0000;
    /// Every access right that conveys the ability to change a path or its security.
    pub(crate) const WRITE_MASK: u32 =
        FILE_WRITE_DATA | FILE_APPEND_DATA | WRITE_DAC | WRITE_OWNER | GENERIC_WRITE | GENERIC_ALL;

    /// Type of an entry that grants access and carries a plain SID.
    pub(crate) const ACCESS_ALLOWED_ACE_TYPE: u8 = 0;
    /// Type of an entry that denies access and carries a plain SID.
    pub(crate) const ACCESS_DENIED_ACE_TYPE: u8 = 1;
    /// Type of an object specific entry that denies access.
    pub(crate) const ACCESS_DENIED_OBJECT_ACE_TYPE: u8 = 6;
    /// Type of a conditional entry that denies access.
    pub(crate) const ACCESS_DENIED_CALLBACK_ACE_TYPE: u8 = 10;
    /// Type of a conditional object specific entry that denies access.
    pub(crate) const ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE: u8 = 12;
    /// Flag of an entry that applies only to children and not to the object itself.
    pub(crate) const INHERIT_ONLY_ACE: u8 = 0x08;

    /// Reduces the header fields of an access control entry to an [`AccessEntry`].
    ///
    /// Deny entries never grant access, so every deny type becomes [`EntryKind::Deny`]. Every
    /// other type becomes [`EntryKind::Allow`] so that unknown or conditional allow variants
    /// fail closed when they carry write rights.
    ///
    /// # Arguments
    ///
    /// 1. `ace_type`: the type field of the entry header.
    /// 1. `ace_flags`: the flags field of the entry header.
    /// 1. `mask`: the access mask of the entry.
    /// 1. `principal`: the principal the entry applies to.
    ///
    /// # Returns
    ///
    /// The reduced entry.
    pub(crate) fn classify(ace_type: u8, ace_flags: u8, mask: u32, principal: Principal) -> AccessEntry {
        let kind = match ace_type {
            ACCESS_DENIED_ACE_TYPE
            | ACCESS_DENIED_OBJECT_ACE_TYPE
            | ACCESS_DENIED_CALLBACK_ACE_TYPE
            | ACCESS_DENIED_CALLBACK_OBJECT_ACE_TYPE => EntryKind::Deny,
            _ => EntryKind::Allow,
        };
        AccessEntry {
            kind,
            principal,
            grants_write: (mask & WRITE_MASK) != 0,
            inherit_only: (ace_flags & INHERIT_ONLY_ACE) != 0,
        }
    }
}

/// Translation of Unix ownership and mode bits into the security model.
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub(crate) mod unix {
    use std::path::Path;

    use rust_i18n::t;

    use super::{AccessEntry, EntryKind, Principal, PrincipalKind, SecurityModel};

    /// Mode bit that grants the owner write access.
    const OWNER_WRITE: u32 = 0o200;
    /// Mode bit that grants the group write access.
    const GROUP_WRITE: u32 = 0o020;
    /// Mode bit that grants every other user write access.
    const OTHER_WRITE: u32 = 0o002;

    /// Builds a security model from Unix ownership and mode bits.
    ///
    /// The owner is the root user, the current user, or another user. The group and other
    /// classes are always untrusted because membership in a group does not make a user an
    /// administrator. Each class contributes one allow entry whose write flag reflects its
    /// write bit.
    ///
    /// # Arguments
    ///
    /// 1. `path`: the path the mode belongs to.
    /// 1. `uid`: the user id of the owner.
    /// 1. `gid`: the group id of the path.
    /// 1. `mode`: the mode bits, including any file type bits, which are ignored.
    /// 1. `effective_uid`: the effective user id of the current process.
    ///
    /// # Returns
    ///
    /// The security model for the path.
    pub(crate) fn model_from_mode(path: &Path, uid: u32, gid: u32, mode: u32, effective_uid: u32) -> SecurityModel {
        let owner = if uid == 0 {
            Principal::new(PrincipalKind::Administrators, t!("principal.root", uid = uid))
        } else if uid == effective_uid {
            Principal::new(PrincipalKind::CurrentUser, t!("principal.user", uid = uid))
        } else {
            Principal::new(PrincipalKind::Other, t!("principal.user", uid = uid))
        };

        let entry = |principal: Principal, write_bit: u32| AccessEntry {
            kind: EntryKind::Allow,
            principal,
            grants_write: (mode & write_bit) != 0,
            inherit_only: false,
        };

        let entries = vec![
            entry(owner.clone(), OWNER_WRITE),
            entry(Principal::new(PrincipalKind::Other, t!("principal.group", gid = gid)), GROUP_WRITE),
            entry(Principal::new(PrincipalKind::Other, t!("principal.otherUsers")), OTHER_WRITE),
        ];

        SecurityModel {
            path: path.to_path_buf(),
            owner,
            entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use test_case::test_case;

    use super::PrincipalKind::{Administrators, CurrentUser, LocalSystem, Other};
    use super::*;

    /// The result of an evaluation reduced to the facts the tests compare.
    #[derive(Debug, PartialEq, Eq)]
    enum Outcome {
        Secure,
        UntrustedOwner(String),
        UntrustedWriter(String),
    }

    fn outcome(result: Result<(), PathSecurityError>) -> Outcome {
        match result {
            Ok(()) => Outcome::Secure,
            Err(PathSecurityError::UntrustedOwner { principal, .. }) => Outcome::UntrustedOwner(principal),
            Err(PathSecurityError::UntrustedWriter { principal, .. }) => Outcome::UntrustedWriter(principal),
            Err(other) => panic!("unexpected error {other:?}"),
        }
    }

    fn principal(kind: PrincipalKind) -> Principal {
        let id = match kind {
            LocalSystem => "S-1-5-18",
            Administrators => "S-1-5-32-544",
            CurrentUser => "S-1-5-21-1-2-3-1001",
            Other => "S-1-1-0",
        };
        Principal::new(kind, id)
    }

    fn entry(kind: EntryKind, principal_kind: PrincipalKind, grants_write: bool, inherit_only: bool) -> AccessEntry {
        AccessEntry {
            kind,
            principal: principal(principal_kind),
            grants_write,
            inherit_only,
        }
    }

    fn allow(principal_kind: PrincipalKind) -> AccessEntry {
        entry(EntryKind::Allow, principal_kind, true, false)
    }

    fn model(owner: PrincipalKind, extra: Vec<AccessEntry>) -> SecurityModel {
        let mut entries = vec![allow(LocalSystem), allow(Administrators)];
        entries.extend(extra);
        SecurityModel {
            path: PathBuf::from("example"),
            owner: principal(owner),
            entries,
        }
    }

    #[test_case(Administrators, vec![], PathSecurityScope::Machine => Outcome::Secure; "trusted owner and writers at machine scope")]
    #[test_case(Administrators, vec![], PathSecurityScope::User => Outcome::Secure; "trusted owner and writers at user scope")]
    #[test_case(LocalSystem, vec![], PathSecurityScope::Machine => Outcome::Secure; "system owner at machine scope")]
    #[test_case(Administrators, vec![allow(Other)], PathSecurityScope::Machine => Outcome::UntrustedWriter("S-1-1-0".into()); "other writer at machine scope")]
    #[test_case(Administrators, vec![allow(Other)], PathSecurityScope::User => Outcome::UntrustedWriter("S-1-1-0".into()); "other writer at user scope")]
    #[test_case(Administrators, vec![entry(EntryKind::Allow, Other, false, false)], PathSecurityScope::Machine => Outcome::Secure; "other reader is allowed")]
    #[test_case(Administrators, vec![entry(EntryKind::Allow, Other, true, true)], PathSecurityScope::Machine => Outcome::Secure; "inherit only writer does not apply")]
    #[test_case(Administrators, vec![entry(EntryKind::Deny, Other, true, false)], PathSecurityScope::Machine => Outcome::Secure; "deny entry does not grant")]
    #[test_case(Administrators, vec![allow(CurrentUser)], PathSecurityScope::Machine => Outcome::UntrustedWriter("S-1-5-21-1-2-3-1001".into()); "current user writer at machine scope")]
    #[test_case(Administrators, vec![allow(CurrentUser)], PathSecurityScope::User => Outcome::Secure; "current user writer at user scope")]
    #[test_case(CurrentUser, vec![], PathSecurityScope::Machine => Outcome::UntrustedOwner("S-1-5-21-1-2-3-1001".into()); "current user owner at machine scope")]
    #[test_case(CurrentUser, vec![], PathSecurityScope::User => Outcome::Secure; "current user owner at user scope")]
    #[test_case(Other, vec![], PathSecurityScope::User => Outcome::UntrustedOwner("S-1-1-0".into()); "other owner at user scope")]
    #[test_case(Other, vec![allow(Other)], PathSecurityScope::Machine => Outcome::UntrustedOwner("S-1-1-0".into()); "owner is reported before writers")]
    fn evaluates(owner: PrincipalKind, extra: Vec<AccessEntry>, scope: PathSecurityScope) -> Outcome {
        outcome(evaluate(&model(owner, extra), scope))
    }

    #[test]
    fn reports_the_first_untrusted_writer() {
        let first = AccessEntry {
            kind: EntryKind::Allow,
            principal: Principal::new(Other, "S-1-5-32-545"),
            grants_write: true,
            inherit_only: false,
        };
        let result = evaluate(&model(Administrators, vec![first, allow(Other)]), PathSecurityScope::Machine);
        assert_eq!(outcome(result), Outcome::UntrustedWriter("S-1-5-32-545".into()));
    }

    #[test]
    fn errors_carry_the_model_path() {
        let result = evaluate(&model(Other, vec![]), PathSecurityScope::Machine);
        assert_eq!(
            result,
            Err(PathSecurityError::UntrustedOwner {
                path: "example".into(),
                principal: "S-1-1-0".into(),
            })
        );
    }

    #[test_case(0 => EntryKind::Allow; "access allowed")]
    #[test_case(1 => EntryKind::Deny; "access denied")]
    #[test_case(2 => EntryKind::Allow; "system audit is treated as allow")]
    #[test_case(5 => EntryKind::Allow; "access allowed object")]
    #[test_case(6 => EntryKind::Deny; "access denied object")]
    #[test_case(9 => EntryKind::Allow; "access allowed callback")]
    #[test_case(10 => EntryKind::Deny; "access denied callback")]
    #[test_case(11 => EntryKind::Allow; "access allowed callback object")]
    #[test_case(12 => EntryKind::Deny; "access denied callback object")]
    fn classifies_entry_kind(ace_type: u8) -> EntryKind {
        ace::classify(ace_type, 0, ace::WRITE_MASK, principal(Other)).kind
    }

    #[test_case(ace::FILE_WRITE_DATA => true; "file write data")]
    #[test_case(ace::FILE_APPEND_DATA => true; "file append data")]
    #[test_case(ace::WRITE_DAC => true; "write dac")]
    #[test_case(ace::WRITE_OWNER => true; "write owner")]
    #[test_case(ace::GENERIC_WRITE => true; "generic write")]
    #[test_case(ace::GENERIC_ALL => true; "generic all")]
    #[test_case(0x001F_01FF => true; "full control")]
    #[test_case(0x0012_0089 => false; "read and execute")]
    #[test_case(0x0000_0100 => false; "write attributes alone")]
    #[test_case(0x0000_0010 => false; "write extended attributes alone")]
    #[test_case(0 => false; "no rights")]
    fn classifies_write_rights(mask: u32) -> bool {
        ace::classify(ace::ACCESS_ALLOWED_ACE_TYPE, 0, mask, principal(Other)).grants_write
    }

    #[test_case(0x08 => true; "inherit only")]
    #[test_case(0x0B => true; "inherit only with object and container inherit")]
    #[test_case(0x03 => false; "object and container inherit")]
    #[test_case(0x10 => false; "inherited")]
    #[test_case(0x00 => false; "no flags")]
    fn classifies_inherit_only(ace_flags: u8) -> bool {
        ace::classify(ace::ACCESS_ALLOWED_ACE_TYPE, ace_flags, ace::WRITE_MASK, principal(Other)).inherit_only
    }

    #[test_case(0, 0, 0o755, 1000, PathSecurityScope::Machine => Outcome::Secure; "root owned folder")]
    #[test_case(0, 0, 0o40755, 1000, PathSecurityScope::Machine => Outcome::Secure; "file type bits are ignored")]
    #[test_case(0, 0, 0o775, 1000, PathSecurityScope::Machine => Outcome::UntrustedWriter("group 0".into()); "group writable")]
    #[test_case(0, 0, 0o757, 1000, PathSecurityScope::Machine => Outcome::UntrustedWriter("other users".into()); "other writable")]
    #[test_case(0, 0, 0o1777, 1000, PathSecurityScope::Machine => Outcome::UntrustedWriter("group 0".into()); "sticky world writable")]
    #[test_case(0, 0, 0o755, 0, PathSecurityScope::Machine => Outcome::Secure; "root owner when running as root")]
    #[test_case(1000, 1000, 0o700, 1000, PathSecurityScope::Machine => Outcome::UntrustedOwner("uid 1000".into()); "current user owner at machine scope")]
    #[test_case(1000, 1000, 0o700, 1000, PathSecurityScope::User => Outcome::Secure; "current user owner at user scope")]
    #[test_case(1000, 1000, 0o700, 1001, PathSecurityScope::User => Outcome::UntrustedOwner("uid 1000".into()); "another user owner at user scope")]
    #[test_case(1000, 1000, 0o770, 1000, PathSecurityScope::User => Outcome::UntrustedWriter("group 1000".into()); "group writable at user scope")]
    #[test_case(1000, 1000, 0o707, 1000, PathSecurityScope::User => Outcome::UntrustedWriter("other users".into()); "other writable at user scope")]
    fn evaluates_unix_mode(uid: u32, gid: u32, mode: u32, effective_uid: u32, scope: PathSecurityScope) -> Outcome {
        let model = unix::model_from_mode(Path::new("example"), uid, gid, mode, effective_uid);
        outcome(evaluate(&model, scope))
    }

    #[test]
    fn unix_model_describes_the_owner() {
        let model = unix::model_from_mode(Path::new("example"), 0, 20, 0o755, 1000);
        assert_eq!(model.owner, Principal::new(Administrators, "root (uid 0)"));
        assert_eq!(model.entries.len(), 3);
        assert!(model.entries[0].grants_write);
        assert!(!model.entries[1].grants_write);
        assert!(!model.entries[2].grants_write);
    }
}
