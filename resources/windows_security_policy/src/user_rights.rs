// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Get, test, set and export of user rights assignments.

use crate::lsa::{LsaPolicy, POLICY_READ, POLICY_WRITE};
use crate::rights::{self, USER_RIGHTS, UserRight};
use crate::sid::{self, ResolvedIdentity};
use crate::types::{Metadata, SecurityPolicyError, SetAction, UserRightsAssignment};
use dsc_lib_security_context::{SecurityContext, get_security_context};
use rust_i18n::t;

/// Fail unless the process is elevated. `dsc` enforces this through the manifest; this check
/// covers direct invocation of the binary.
pub(crate) fn require_elevated() -> Result<(), SecurityPolicyError> {
    if get_security_context() == SecurityContext::Admin {
        Ok(())
    } else {
        Err(SecurityPolicyError::ElevationRequired)
    }
}

fn find_right(input: &str) -> Result<&'static UserRight, SecurityPolicyError> {
    rights::find(input).ok_or_else(|| SecurityPolicyError::InvalidInput(t!("rights.unknownRight", right = input).to_string()))
}

/// The accounts holding `right`, sorted by name for stable output.
fn current_identities(policy: &LsaPolicy, right: &UserRight) -> Result<Vec<ResolvedIdentity>, SecurityPolicyError> {
    let mut identities = policy.holders(right.constant)?.into_iter().map(sid::describe).collect::<Result<Vec<_>, _>>()?;
    identities.sort_by_cached_key(|identity| identity.name.to_lowercase());
    Ok(identities)
}

fn names(identities: &[ResolvedIdentity]) -> Vec<String> {
    identities.iter().map(|identity| identity.name.clone()).collect()
}

fn state(right: &str, current: &[ResolvedIdentity]) -> UserRightsAssignment {
    UserRightsAssignment {
        right: right.to_string(),
        identity: Some(names(current)),
        ..UserRightsAssignment::default()
    }
}

fn require_identity(input: &UserRightsAssignment) -> Result<&[String], SecurityPolicyError> {
    input
        .identity
        .as_deref()
        .ok_or_else(|| SecurityPolicyError::InvalidInput(t!("userRights.identityRequired").to_string()))
}

fn contains(identities: &[ResolvedIdentity], wanted: &ResolvedIdentity) -> bool {
    identities.iter().any(|identity| identity.sid == wanted.sid)
}

/// The accounts to grant and to revoke so that `current` satisfies `desired` under `action`.
/// Both lists empty means the right is in the desired state.
pub(crate) fn diff<'a>(
    current: &'a [ResolvedIdentity],
    desired: &'a [ResolvedIdentity],
    action: SetAction,
) -> (Vec<&'a ResolvedIdentity>, Vec<&'a ResolvedIdentity>) {
    let missing = || desired.iter().filter(|identity| !contains(current, identity)).collect::<Vec<_>>();
    match action {
        SetAction::Replace => (missing(), current.iter().filter(|identity| !contains(desired, identity)).collect()),
        SetAction::Add => (missing(), Vec::new()),
        SetAction::Remove => (Vec::new(), desired.iter().filter(|identity| contains(current, identity)).collect()),
    }
}

/// The holder names after applying `grant` and `revoke` to `current`, for what-if output.
pub(crate) fn project(current: &[ResolvedIdentity], grant: &[&ResolvedIdentity], revoke: &[&ResolvedIdentity]) -> Vec<String> {
    let mut names: Vec<String> = current
        .iter()
        .filter(|identity| !revoke.iter().any(|revoked| revoked.sid == identity.sid))
        .map(|identity| identity.name.clone())
        .collect();
    names.extend(grant.iter().map(|identity| identity.name.clone()));
    names.sort_by_cached_key(|name| name.to_lowercase());
    names
}

/// The what-if result of `set`: the holders `right` would have once `input.setAction` applies
/// `desired` to `current`, with a `_metadata.whatIf` message for every account that would be
/// granted or revoked. `add` and `remove` keep the holders that aren't listed; only `replace`
/// revokes them. When nothing would change, the result is the current state without metadata.
///
/// # Arguments
///
/// * `input` - The requested state; supplies `right` as given and `setAction`.
/// * `right` - The user right that `input.right` names.
/// * `current` - The accounts that hold the right now.
/// * `desired` - The resolved `identity` entries of `input`.
pub(crate) fn preview(
    input: &UserRightsAssignment,
    right: &UserRight,
    current: &[ResolvedIdentity],
    desired: &[ResolvedIdentity],
) -> UserRightsAssignment {
    let (grant, revoke) = diff(current, desired, input.set_action);
    let name = right.display_name();
    let messages = grant
        .iter()
        .map(|identity| t!("userRights.whatIfGrant", right = right.constant, name = name, identity = identity.name).to_string())
        .chain(
            revoke
                .iter()
                .map(|identity| t!("userRights.whatIfRevoke", right = right.constant, name = name, identity = identity.name).to_string()),
        )
        .collect();
    UserRightsAssignment {
        right: input.right.clone(),
        identity: Some(project(current, &grant, &revoke)),
        metadata: Metadata::from_messages(messages),
        ..UserRightsAssignment::default()
    }
}

/// Get the accounts that hold a right.
///
/// # Errors
///
/// Returns an error when the process is not elevated, the right is unknown, or an LSA call fails.
pub fn get(input: &UserRightsAssignment) -> Result<UserRightsAssignment, SecurityPolicyError> {
    require_elevated()?;
    let right = find_right(&input.right)?;
    let policy = LsaPolicy::open(POLICY_READ)?;
    Ok(state(&input.right, &current_identities(&policy, right)?))
}

/// Test whether the holders of a right satisfy `identity` under `setAction`, comparing SIDs so
/// that any accepted spelling of an account matches.
///
/// # Errors
///
/// Returns an error when the process is not elevated, the right or an identity is unknown,
/// `identity` is missing, or an LSA call fails.
pub fn test(input: &UserRightsAssignment) -> Result<UserRightsAssignment, SecurityPolicyError> {
    require_elevated()?;
    let right = find_right(&input.right)?;
    let desired = sid::resolve_identities(require_identity(input)?)?;
    let policy = LsaPolicy::open(POLICY_READ)?;
    let current = current_identities(&policy, right)?;
    let (grant, revoke) = diff(&current, &desired, input.set_action);
    let mut result = state(&input.right, &current);
    result.in_desired_state = Some(grant.is_empty() && revoke.is_empty());
    Ok(result)
}

/// Apply `identity` to a right under `setAction`. In what-if mode nothing is changed and the
/// result is the [`preview`] of the same `setAction`.
///
/// # Errors
///
/// Returns an error when the process is not elevated, the right or an identity is unknown,
/// `identity` is missing, or an LSA call fails.
pub fn set(input: &UserRightsAssignment, what_if: bool) -> Result<UserRightsAssignment, SecurityPolicyError> {
    require_elevated()?;
    let right = find_right(&input.right)?;
    // Resolve first so that bad input fails even in what-if mode.
    let desired = sid::resolve_identities(require_identity(input)?)?;
    let policy = LsaPolicy::open(if what_if { POLICY_READ } else { POLICY_WRITE })?;

    let current = match current_identities(&policy, right) {
        Ok(current) => current,
        Err(error) if what_if => {
            return Ok(UserRightsAssignment {
                right: input.right.clone(),
                identity: input.identity.clone(),
                metadata: Metadata::from_messages(vec![error.to_string()]),
                ..UserRightsAssignment::default()
            });
        }
        Err(error) => return Err(error),
    };

    if what_if {
        return Ok(preview(input, right, &current, &desired));
    }

    let (grant, revoke) = diff(&current, &desired, input.set_action);
    if grant.is_empty() && revoke.is_empty() {
        return Ok(state(&input.right, &current));
    }

    // Grant before revoke so that a logon right never has zero holders part-way through.
    for identity in &grant {
        policy.add_right(&identity.sid, right.constant)?;
    }
    for identity in &revoke {
        policy.remove_right(&identity.sid, right.constant)?;
    }
    Ok(state(&input.right, &current_identities(&policy, right)?))
}

/// Export the holders of every user right.
///
/// # Errors
///
/// Returns an error when the process is not elevated or an LSA call fails.
pub fn export() -> Result<Vec<UserRightsAssignment>, SecurityPolicyError> {
    require_elevated()?;
    let policy = LsaPolicy::open(POLICY_READ)?;
    USER_RIGHTS
        .iter()
        .map(|right| Ok(state(right.constant, &current_identities(&policy, right)?)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sid::Sid;

    fn identity(name: &str, marker: u8) -> ResolvedIdentity {
        ResolvedIdentity {
            sid: Sid::from_bytes(vec![1, 1, 0, 0, 0, 0, 0, 5, marker, 0, 0, 0]),
            name: name.to_string(),
        }
    }

    fn names_of<'a>(list: &[&'a ResolvedIdentity]) -> Vec<&'a str> {
        list.iter().map(|identity| identity.name.as_str()).collect()
    }

    #[test]
    fn replace_grants_missing_and_revokes_extra() {
        let current = [identity("A", 1), identity("B", 2)];
        let desired = [identity("B", 2), identity("C", 3)];
        let (grant, revoke) = diff(&current, &desired, SetAction::Replace);
        assert_eq!(names_of(&grant), ["C"]);
        assert_eq!(names_of(&revoke), ["A"]);

        let aliases = [identity("b-alias", 2), identity("a-alias", 1)];
        let (grant, revoke) = diff(&current, &aliases, SetAction::Replace);
        assert!(grant.is_empty() && revoke.is_empty(), "same SIDs under other names are in desired state");

        let (grant, revoke) = diff(&current, &[], SetAction::Replace);
        assert!(grant.is_empty());
        assert_eq!(names_of(&revoke), ["A", "B"]);
    }

    #[test]
    fn add_only_grants() {
        let current = [identity("A", 1)];
        let present = [identity("A", 1)];
        let (grant, revoke) = diff(&current, &present, SetAction::Add);
        assert!(grant.is_empty() && revoke.is_empty());
        let missing = [identity("B", 2)];
        let (grant, revoke) = diff(&current, &missing, SetAction::Add);
        assert_eq!(names_of(&grant), ["B"]);
        assert!(revoke.is_empty());
        let (grant, revoke) = diff(&current, &[], SetAction::Add);
        assert!(grant.is_empty() && revoke.is_empty());
    }

    #[test]
    fn remove_only_revokes_listed_holders() {
        let current = [identity("A", 1), identity("B", 2)];
        let absent = [identity("C", 3)];
        let (grant, revoke) = diff(&current, &absent, SetAction::Remove);
        assert!(grant.is_empty() && revoke.is_empty());
        let mixed = [identity("A", 1), identity("C", 3)];
        let (grant, revoke) = diff(&current, &mixed, SetAction::Remove);
        assert!(grant.is_empty());
        assert_eq!(names_of(&revoke), ["A"]);
        let (grant, revoke) = diff(&current, &[], SetAction::Remove);
        assert!(grant.is_empty() && revoke.is_empty());
    }

    #[test]
    fn projection_applies_changes_and_sorts() {
        let current = [identity("Zed", 1), identity("alpha", 2)];
        let granted = identity("Mike", 3);
        let revoked = identity("Zed", 1);
        assert_eq!(project(&current, &[&granted], &[&revoked]), ["alpha", "Mike"]);
        assert_eq!(project(&current, &[], &[]), ["alpha", "Zed"]);
    }

    fn what_if_request(set_action: SetAction) -> UserRightsAssignment {
        UserRightsAssignment {
            right: "sedenybatchlogonright".to_string(),
            identity: Some(vec!["Guests".to_string(), "Anonymous".to_string()]),
            set_action,
            ..UserRightsAssignment::default()
        }
    }

    fn grant_message(right: &UserRight, identity: &str) -> String {
        t!("userRights.whatIfGrant", right = right.constant, name = right.display_name(), identity = identity).to_string()
    }

    fn revoke_message(right: &UserRight, identity: &str) -> String {
        t!("userRights.whatIfRevoke", right = right.constant, name = right.display_name(), identity = identity).to_string()
    }

    fn what_if_messages(result: &UserRightsAssignment) -> Vec<String> {
        result.metadata.as_ref().and_then(|metadata| metadata.what_if.clone()).unwrap_or_default()
    }

    #[test]
    fn preview_honours_every_set_action() {
        let right = rights::find("SeDenyBatchLogonRight").unwrap();
        let current = [identity("Guests", 1), identity("Users", 2)];
        let desired = [identity("Guests", 1), identity("Anonymous", 3)];

        let replaced = preview(&what_if_request(SetAction::Replace), right, &current, &desired);
        assert_eq!(replaced.right, "sedenybatchlogonright", "right is echoed as given");
        assert_eq!(replaced.identity.as_deref(), Some(["Anonymous", "Guests"].map(String::from).as_slice()));
        assert_eq!(what_if_messages(&replaced), [grant_message(right, "Anonymous"), revoke_message(right, "Users")]);

        let added = preview(&what_if_request(SetAction::Add), right, &current, &desired);
        assert_eq!(added.identity.as_deref(), Some(["Anonymous", "Guests", "Users"].map(String::from).as_slice()));
        assert_eq!(what_if_messages(&added), [grant_message(right, "Anonymous")]);

        let removed = preview(&what_if_request(SetAction::Remove), right, &current, &desired);
        assert_eq!(removed.identity.as_deref(), Some(["Users"].map(String::from).as_slice()));
        assert_eq!(what_if_messages(&removed), [revoke_message(right, "Guests")]);

        for result in [&replaced, &added, &removed] {
            assert!(result.in_desired_state.is_none());
        }
    }

    #[test]
    fn preview_without_changes_reports_the_current_state_without_metadata() {
        let right = rights::find("SeDenyBatchLogonRight").unwrap();
        let current = [identity("Guests", 1), identity("Users", 2)];
        let (present, absent) = ([identity("Guests", 1)], [identity("Anonymous", 3)]);
        for (action, desired) in [(SetAction::Replace, &current[..]), (SetAction::Add, &present[..]), (SetAction::Remove, &absent[..])] {
            let result = preview(&what_if_request(action), right, &current, desired);
            assert_eq!(result.identity.as_deref(), Some(["Guests", "Users"].map(String::from).as_slice()), "{action:?}");
            assert!(result.metadata.is_none(), "{action:?}");
        }
    }

    #[test]
    fn what_if_messages_name_the_right_by_constant_and_display_name() {
        let right = rights::find("SeServiceLogonRight").unwrap();
        let message = grant_message(right, "NT SERVICE\\ALL SERVICES");
        assert!(message.contains("SeServiceLogonRight"), "{message}");
        assert!(message.contains(right.display_name().as_ref()), "{message}");
    }

    #[test]
    fn operations_follow_the_security_context() {
        let input = UserRightsAssignment {
            right: "SeChangeNotifyPrivilege".to_string(),
            identity: Some(vec!["Everyone".to_string()]),
            ..UserRightsAssignment::default()
        };
        if get_security_context() == SecurityContext::Admin {
            let current = get(&input).unwrap();
            assert_eq!(current.right, "SeChangeNotifyPrivilege");
            assert!(current.identity.unwrap().iter().any(|name| name == "Everyone"));
            let tested = test(&UserRightsAssignment { set_action: SetAction::Add, ..input.clone() }).unwrap();
            assert_eq!(tested.in_desired_state, Some(true));
            assert_eq!(export().unwrap().len(), USER_RIGHTS.len());
        } else {
            assert!(matches!(get(&input), Err(SecurityPolicyError::ElevationRequired)));
            assert!(matches!(test(&input), Err(SecurityPolicyError::ElevationRequired)));
            assert!(matches!(set(&input, true), Err(SecurityPolicyError::ElevationRequired)));
            assert!(matches!(export(), Err(SecurityPolicyError::ElevationRequired)));
        }
    }

    #[test]
    fn unknown_right_and_missing_identity_are_invalid_input_before_any_lsa_call() {
        // These checks happen after the elevation check, so only assert them when elevated.
        if get_security_context() != SecurityContext::Admin {
            return;
        }
        let unknown = UserRightsAssignment { right: "SeBogus".to_string(), ..UserRightsAssignment::default() };
        assert!(matches!(get(&unknown), Err(SecurityPolicyError::InvalidInput(_))));
        let no_identity = UserRightsAssignment { right: "SeDebugPrivilege".to_string(), ..UserRightsAssignment::default() };
        assert!(matches!(test(&no_identity), Err(SecurityPolicyError::InvalidInput(_))));
        assert!(matches!(set(&no_identity, true), Err(SecurityPolicyError::InvalidInput(_))));
    }
}
