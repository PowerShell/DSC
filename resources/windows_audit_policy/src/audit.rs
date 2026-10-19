// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Reads and writes the system audit policy through the LSA audit APIs.
//!
//! Every pointer handed out by an `Audit*` API is owned by the LSA and must be released with
//! `AuditFree`; the RAII guards below do that on drop so early returns cannot leak.

use crate::subcategories::{self, format_guid, parse_guid};
use crate::types::{AuditError, AuditFlags, AuditPolicy, AuditPolicyList, Metadata};
use dsc_lib_security_context::{SecurityContext, get_security_context};
use rust_i18n::t;
use std::collections::HashSet;
use std::ptr;
use windows::Win32::Foundation::{CloseHandle, ERROR_NOT_ALL_ASSIGNED, GetLastError, HANDLE, LUID};
use windows::Win32::Security::Authentication::Identity::{
    AUDIT_POLICY_INFORMATION, AuditEnumerateCategories, AuditEnumerateSubCategories, AuditFree,
    AuditLookupCategoryNameW, AuditLookupSubCategoryNameW, AuditQuerySystemPolicy, AuditSetSystemPolicy,
};
use windows::Win32::Security::{
    AdjustTokenPrivileges, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED, SE_SECURITY_NAME,
    TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::{GUID, PCWSTR, PWSTR};

impl From<windows::core::Error> for AuditError {
    fn from(error: windows::core::Error) -> Self {
        Self::Audit(error.to_string())
    }
}

/// The error for the most recent failed Win32 call on this thread.
fn last_error() -> windows::core::Error {
    windows::core::Error::from_thread()
}

/// An array allocated by an `Audit*` API and released with `AuditFree`.
struct AuditBuffer<T> {
    ptr: *mut T,
    count: usize,
}

impl<T> AuditBuffer<T> {
    fn as_slice(&self) -> &[T] {
        if self.ptr.is_null() || self.count == 0 {
            &[]
        } else {
            // SAFETY: the LSA allocated `count` contiguous items at `ptr`; they stay valid until
            // `AuditFree` runs in `drop`.
            unsafe { std::slice::from_raw_parts(self.ptr, self.count) }
        }
    }
}

impl<T> Drop for AuditBuffer<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: `ptr` came from an `Audit*` API and is released exactly once.
            unsafe { AuditFree(self.ptr.cast()) };
        }
    }
}

/// A string allocated by an `AuditLookup*NameW` API and released with `AuditFree`.
struct AuditString(PWSTR);

impl AuditString {
    fn to_string_lossy(&self) -> String {
        if self.0.is_null() {
            String::new()
        } else {
            // SAFETY: the LSA returns a null-terminated UTF-16 string that stays valid until
            // `AuditFree` runs in `drop`.
            unsafe { String::from_utf16_lossy(self.0.as_wide()) }
        }
    }
}

impl Drop for AuditString {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: the string came from an `AuditLookup*NameW` API and is released exactly once.
            unsafe { AuditFree(self.0.as_ptr().cast()) };
        }
    }
}

/// An open access token handle, closed on drop.
struct TokenHandle(HANDLE);

impl Drop for TokenHandle {
    fn drop(&mut self) {
        // SAFETY: the handle was returned by `OpenProcessToken` and is closed exactly once.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// A subcategory resolved from user input.
#[derive(Debug)]
struct Resolved {
    guid: GUID,
    /// English name when known, otherwise the name the LSA reports.
    name: String,
    /// English category name when known from the static table.
    category: Option<String>,
}

/// Fail unless the process is elevated. `dsc` enforces this through the manifest; this check
/// covers direct invocation of the binary.
pub(crate) fn require_elevated() -> Result<(), AuditError> {
    if get_security_context() == SecurityContext::Admin {
        Ok(())
    } else {
        Err(AuditError::ElevationRequired)
    }
}

/// Enable `SeSecurityPrivilege` on the process token. The audit APIs need the privilege, and an
/// elevated token holds it but does not enable it by default.
fn enable_security_privilege() -> Result<(), AuditError> {
    let privilege_error =
        |error: windows::core::Error| AuditError::Audit(t!("audit.privilegeFailed", error = error.to_string()).to_string());

    let mut raw_token = HANDLE::default();
    // SAFETY: the pseudo handle from `GetCurrentProcess` is always valid; `raw_token` receives the
    // opened token and is owned by `TokenHandle` from here on.
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut raw_token) }
        .map_err(privilege_error)?;
    let token = TokenHandle(raw_token);

    let mut luid = LUID::default();
    // SAFETY: `SE_SECURITY_NAME` is a valid null-terminated constant and `luid` is a valid out pointer.
    unsafe { LookupPrivilegeValueW(PCWSTR::null(), SE_SECURITY_NAME, &mut luid) }.map_err(privilege_error)?;

    let privileges = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };
    // SAFETY: `token` is open with `TOKEN_ADJUST_PRIVILEGES` and `privileges` outlives the call.
    unsafe { AdjustTokenPrivileges(token.0, false, Some(ptr::from_ref(&privileges)), 0, None, None) }
        .map_err(privilege_error)?;
    // AdjustTokenPrivileges succeeds even when the token does not hold the privilege at all;
    // only the last error reveals that.
    // SAFETY: reading the calling thread's last error has no preconditions.
    if unsafe { GetLastError() } == ERROR_NOT_ALL_ASSIGNED {
        return Err(AuditError::ElevationRequired);
    }
    Ok(())
}

fn enumerate_categories() -> Result<AuditBuffer<GUID>, AuditError> {
    let mut ptr: *mut GUID = ptr::null_mut();
    let mut count: u32 = 0;
    // SAFETY: both out pointers are valid for the duration of the call.
    if !unsafe { AuditEnumerateCategories(&mut ptr, &mut count) } {
        return Err(AuditError::Audit(
            t!("audit.enumerateCategoriesFailed", error = last_error().to_string()).to_string(),
        ));
    }
    Ok(AuditBuffer { ptr, count: count as usize })
}

/// Enumerate the subcategories of one category, or of every category when `category` is `None`.
fn enumerate_subcategories(category: Option<&GUID>) -> Result<AuditBuffer<GUID>, AuditError> {
    let mut ptr: *mut GUID = ptr::null_mut();
    let mut count: u32 = 0;
    // The "retrieve all" flag makes the LSA ignore the category GUID, so it must only be set
    // when no category is given.
    let retrieve_all = category.is_none();
    // SAFETY: the optional category pointer and both out pointers are valid for the call.
    if !unsafe { AuditEnumerateSubCategories(category.map(ptr::from_ref), retrieve_all, &mut ptr, &mut count) } {
        return Err(AuditError::Audit(
            t!("audit.enumerateSubcategoriesFailed", error = last_error().to_string()).to_string(),
        ));
    }
    Ok(AuditBuffer { ptr, count: count as usize })
}

fn lookup_error(guid: &GUID) -> AuditError {
    AuditError::Audit(
        t!("audit.lookupNameFailed", guid = format_guid(guid.to_u128()), error = last_error().to_string()).to_string(),
    )
}

/// The localized subcategory name reported by the LSA.
fn lookup_subcategory_name(guid: &GUID) -> Result<String, AuditError> {
    let mut name = PWSTR::null();
    // SAFETY: `guid` and `name` are valid pointers for the duration of the call.
    if !unsafe { AuditLookupSubCategoryNameW(ptr::from_ref(guid), &mut name) } {
        return Err(lookup_error(guid));
    }
    Ok(AuditString(name).to_string_lossy())
}

/// The localized category name reported by the LSA.
fn lookup_category_name(guid: &GUID) -> Result<String, AuditError> {
    let mut name = PWSTR::null();
    // SAFETY: `guid` and `name` are valid pointers for the duration of the call.
    if !unsafe { AuditLookupCategoryNameW(ptr::from_ref(guid), &mut name) } {
        return Err(lookup_error(guid));
    }
    Ok(AuditString(name).to_string_lossy())
}

/// English subcategory name from the static table, falling back to the LSA's localized name.
fn subcategory_display_name(guid: &GUID) -> Result<String, AuditError> {
    match subcategories::find_by_guid(guid.to_u128()) {
        Some(entry) => Ok(entry.name.to_string()),
        None => lookup_subcategory_name(guid),
    }
}

/// English category name from the static table, falling back to the LSA's localized name.
fn category_display_name(guid: &GUID) -> Result<String, AuditError> {
    match subcategories::category_name(guid.to_u128()) {
        Some(name) => Ok(name.to_string()),
        None => lookup_category_name(guid),
    }
}

/// Resolve the `subcategory` input (English name or GUID) to a subcategory GUID.
fn resolve_subcategory(input: &str) -> Result<Resolved, AuditError> {
    let unknown = || AuditError::InvalidInput(t!("audit.unknownSubcategory", subcategory = input).to_string());

    if let Some(guid) = parse_guid(input) {
        if let Some(entry) = subcategories::find_by_guid(guid) {
            return Ok(Resolved {
                guid: GUID::from_u128(guid),
                name: entry.name.to_string(),
                category: Some(entry.category.name.to_string()),
            });
        }
        // Not in the table: let the LSA decide whether the GUID is a real subcategory.
        let guid = GUID::from_u128(guid);
        let name = lookup_subcategory_name(&guid).map_err(|_| unknown())?;
        return Ok(Resolved { guid, name, category: None });
    }

    if let Some(entry) = subcategories::find_by_name(input) {
        return Ok(Resolved {
            guid: GUID::from_u128(entry.guid),
            name: entry.name.to_string(),
            category: Some(entry.category.name.to_string()),
        });
    }

    // Fall back to the localized names the LSA reports, so localized input and subcategories
    // newer than the table still resolve.
    let wanted = input.trim();
    let all = enumerate_subcategories(None)?;
    for guid in all.as_slice() {
        let name = lookup_subcategory_name(guid)?;
        if name.eq_ignore_ascii_case(wanted) {
            return Ok(Resolved { guid: *guid, name, category: None });
        }
    }
    Err(unknown())
}

/// Query the effective policy for a set of subcategories.
fn query_policy(guids: &[GUID]) -> Result<AuditBuffer<AUDIT_POLICY_INFORMATION>, windows::core::Error> {
    let mut ptr: *mut AUDIT_POLICY_INFORMATION = ptr::null_mut();
    if guids.is_empty() {
        // The API rejects an empty query.
        return Ok(AuditBuffer { ptr, count: 0 });
    }
    // SAFETY: `guids` is a valid slice and `ptr` a valid out pointer for the call.
    if !unsafe { AuditQuerySystemPolicy(guids, &mut ptr) } {
        return Err(last_error());
    }
    Ok(AuditBuffer { ptr, count: guids.len() })
}

/// Build the resource state from queried policy information.
fn policy_from_info(info: &AUDIT_POLICY_INFORMATION, subcategory: String, category: Option<&str>) -> Result<AuditPolicy, AuditError> {
    let flags = AuditFlags::from_bits(info.AuditingInformation);
    let category = match category {
        Some(name) => name.to_string(),
        None => category_display_name(&info.AuditCategoryGuid)?,
    };
    Ok(AuditPolicy {
        subcategory,
        success: Some(flags.success),
        failure: Some(flags.failure),
        guid: Some(format_guid(info.AuditSubCategoryGuid.to_u128())),
        category: Some(category),
        metadata: None,
    })
}

/// Query one resolved subcategory, echoing `echo` as the `subcategory` value.
fn query_one(resolved: &Resolved, echo: &str) -> Result<(AUDIT_POLICY_INFORMATION, AuditPolicy), AuditError> {
    let query_error =
        |error: String| AuditError::Audit(t!("audit.queryFailed", subcategory = resolved.name, error = error).to_string());
    let buffer = query_policy(&[resolved.guid]).map_err(|error| query_error(error.to_string()))?;
    let info = buffer
        .as_slice()
        .first()
        .copied()
        .ok_or_else(|| query_error(t!("audit.queryEmpty").to_string()))?;
    let policy = policy_from_info(&info, echo.to_string(), resolved.category.as_deref())?;
    Ok((info, policy))
}

/// Resolve the `subcategory` of every list entry, in order.
///
/// # Arguments
///
/// * `policies` - The entries of an `AuditPolicyList`.
///
/// # Errors
///
/// Returns `InvalidInput` for an unknown subcategory, or for a subcategory that more than one entry
/// names (by any spelling), because the entries would compete for the same flags.
fn resolve_list(policies: &[AuditPolicy]) -> Result<Vec<Resolved>, AuditError> {
    let mut seen = HashSet::new();
    policies
        .iter()
        .map(|entry| {
            let resolved = resolve_subcategory(&entry.subcategory)?;
            if seen.insert(resolved.guid.to_u128()) {
                Ok(resolved)
            } else {
                Err(AuditError::InvalidInput(t!("audit.duplicateSubcategory", subcategory = resolved.name).to_string()))
            }
        })
        .collect()
}

/// The flags that `input` asks for, keeping the current value of each flag it omits, and a what-if
/// message for each flag that would change.
///
/// # Arguments
///
/// * `subcategory` - The English name of the subcategory, for the messages.
/// * `current` - The current flags of the subcategory.
/// * `input` - The desired state.
fn desired_flags(subcategory: &str, current: AuditFlags, input: &AuditPolicy) -> (AuditFlags, Vec<String>) {
    let desired = AuditFlags {
        success: input.success.unwrap_or(current.success),
        failure: input.failure.unwrap_or(current.failure),
    };
    let mut messages = Vec::new();
    if desired.success != current.success {
        messages.push(
            t!(
                "audit.whatIfChangeSuccess",
                subcategory = subcategory,
                current = current.success.to_string(),
                desired = desired.success.to_string()
            )
            .to_string(),
        );
    }
    if desired.failure != current.failure {
        messages.push(
            t!(
                "audit.whatIfChangeFailure",
                subcategory = subcategory,
                current = current.failure.to_string(),
                desired = desired.failure.to_string()
            )
            .to_string(),
        );
    }
    (desired, messages)
}

/// One subcategory's current policy and the change that a desired `AuditPolicy` asks for.
struct Change {
    resolved: Resolved,
    info: AUDIT_POLICY_INFORMATION,
    /// The current state, echoing `subcategory` as given.
    current: AuditPolicy,
    /// The flags after the change.
    desired: AuditFlags,
    /// One what-if message per flag that would change; empty when nothing would change.
    messages: Vec<String>,
}

impl Change {
    /// Query `resolved` and work out how `input` would change it.
    ///
    /// # Arguments
    ///
    /// * `resolved` - The subcategory that `input.subcategory` names.
    /// * `input` - The desired state.
    ///
    /// # Errors
    ///
    /// Returns an error when the LSA query fails. This is an error in what-if mode too: set can't
    /// create a subcategory, so there is no verified current state to project a change from.
    fn plan(resolved: Resolved, input: &AuditPolicy) -> Result<Self, AuditError> {
        let (info, current) = query_one(&resolved, &input.subcategory)?;
        let (desired, messages) = desired_flags(&resolved.name, AuditFlags::from_bits(info.AuditingInformation), input);
        Ok(Self { resolved, info, current, desired, messages })
    }

    /// Whether the subcategory isn't in the desired state yet.
    fn is_needed(&self) -> bool {
        !self.messages.is_empty()
    }

    /// The state after the change, without what-if metadata.
    fn projected(&self) -> AuditPolicy {
        AuditPolicy {
            success: Some(self.desired.success),
            failure: Some(self.desired.failure),
            ..self.current.clone()
        }
    }

    /// The state after the change, as `set` reports it: queried again when the change was applied.
    ///
    /// # Errors
    ///
    /// Returns an error when the LSA query fails.
    fn applied(self) -> Result<AuditPolicy, AuditError> {
        if self.is_needed() {
            query_one(&self.resolved, &self.current.subcategory).map(|(_, policy)| policy)
        } else {
            Ok(self.current)
        }
    }
}

/// Apply `changes` with a single `AuditSetSystemPolicy` call. Does nothing when `changes` is empty.
///
/// # Arguments
///
/// * `changes` - The changes to apply.
///
/// # Errors
///
/// Returns an error when the LSA rejects the new policy.
fn apply(changes: &[&Change]) -> Result<(), AuditError> {
    if changes.is_empty() {
        return Ok(());
    }
    let policies: Vec<AUDIT_POLICY_INFORMATION> = changes
        .iter()
        .map(|change| AUDIT_POLICY_INFORMATION {
            AuditSubCategoryGuid: change.info.AuditSubCategoryGuid,
            AuditingInformation: change.desired.to_bits(),
            AuditCategoryGuid: change.info.AuditCategoryGuid,
        })
        .collect();
    // SAFETY: the slice holds fully initialised policy entries for the duration of the call.
    if !unsafe { AuditSetSystemPolicy(&policies) } {
        let error = last_error();
        let names = changes.iter().map(|change| format!("'{}'", change.resolved.name)).collect::<Vec<_>>().join(", ");
        return Err(AuditError::Audit(t!("audit.setFailed", subcategories = names, error = error.to_string()).to_string()));
    }
    Ok(())
}

/// Get the current auditing of one subcategory.
///
/// # Errors
///
/// Returns an error when the process is not elevated, the subcategory is unknown, or the LSA
/// query fails.
pub fn get_audit_policy(input: &AuditPolicy) -> Result<AuditPolicy, AuditError> {
    require_elevated()?;
    enable_security_privilege()?;
    let resolved = resolve_subcategory(&input.subcategory)?;
    let (_, policy) = query_one(&resolved, &input.subcategory)?;
    Ok(policy)
}

/// Set the auditing of one subcategory. Flags omitted from the input keep their current value.
/// In what-if mode nothing is changed and the projected state carries `_metadata.whatIf`.
///
/// # Errors
///
/// Returns an error when the process is not elevated, the subcategory is unknown, or an LSA
/// call fails.
pub fn set_audit_policy(input: &AuditPolicy, what_if: bool) -> Result<AuditPolicy, AuditError> {
    require_elevated()?;
    enable_security_privilege()?;
    let change = Change::plan(resolve_subcategory(&input.subcategory)?, input)?;
    if what_if {
        return Ok(AuditPolicy {
            metadata: Metadata::from_messages(change.messages.clone()),
            ..change.projected()
        });
    }
    if change.is_needed() {
        apply(&[&change])?;
    }
    change.applied()
}

/// Get the current auditing of every listed subcategory, in the order given.
///
/// # Errors
///
/// Returns an error when the process is not elevated, a subcategory is unknown or listed twice,
/// or an LSA query fails.
pub fn get_audit_policy_list(input: &AuditPolicyList) -> Result<AuditPolicyList, AuditError> {
    require_elevated()?;
    enable_security_privilege()?;
    let policies = resolve_list(&input.policies)?
        .iter()
        .zip(&input.policies)
        .map(|(resolved, entry)| query_one(resolved, &entry.subcategory).map(|(_, policy)| policy))
        .collect::<Result<_, _>>()?;
    Ok(AuditPolicyList { policies, metadata: None })
}

/// Set the auditing of every listed subcategory with a single `AuditSetSystemPolicy` call. Flags
/// omitted from an entry keep their current value. In what-if mode nothing is changed, every entry
/// reports its projected state, and the list carries the messages for all of them in
/// `_metadata.whatIf`.
///
/// # Errors
///
/// Returns an error when the process is not elevated, a subcategory is unknown or listed twice,
/// or an LSA call fails. Nothing is changed when any entry is invalid.
pub fn set_audit_policy_list(input: &AuditPolicyList, what_if: bool) -> Result<AuditPolicyList, AuditError> {
    require_elevated()?;
    enable_security_privilege()?;
    let changes = resolve_list(&input.policies)?
        .into_iter()
        .zip(&input.policies)
        .map(|(resolved, entry)| Change::plan(resolved, entry))
        .collect::<Result<Vec<_>, _>>()?;

    if what_if {
        let messages = changes.iter().flat_map(|change| change.messages.iter().cloned()).collect();
        return Ok(AuditPolicyList {
            policies: changes.iter().map(Change::projected).collect(),
            metadata: Metadata::from_messages(messages),
        });
    }

    apply(&changes.iter().filter(|change| change.is_needed()).collect::<Vec<_>>())?;
    let policies = changes.into_iter().map(Change::applied).collect::<Result<_, _>>()?;
    Ok(AuditPolicyList { policies, metadata: None })
}

/// Export the auditing of every subcategory, grouped by category.
///
/// # Errors
///
/// Returns an error when the process is not elevated or an LSA call fails.
pub fn export_audit_policies() -> Result<Vec<AuditPolicy>, AuditError> {
    require_elevated()?;
    enable_security_privilege()?;

    let categories = enumerate_categories()?;
    let mut policies = Vec::with_capacity(subcategories::SUBCATEGORIES.len());
    for category in categories.as_slice() {
        let category_name = category_display_name(category)?;
        let members = enumerate_subcategories(Some(category))?;
        let buffer = query_policy(members.as_slice()).map_err(|error| {
            AuditError::Audit(
                t!("audit.queryCategoryFailed", category = category_name.clone(), error = error.to_string()).to_string(),
            )
        })?;
        for info in buffer.as_slice() {
            let name = subcategory_display_name(&info.AuditSubCategoryGuid)?;
            policies.push(policy_from_info(info, name, Some(&category_name))?);
        }
    }
    Ok(policies)
}

/// Export the auditing of every subcategory as a single list instance, grouped by category.
///
/// # Errors
///
/// Returns an error when the process is not elevated or an LSA call fails.
pub fn export_audit_policy_list() -> Result<AuditPolicyList, AuditError> {
    Ok(AuditPolicyList {
        policies: export_audit_policies()?,
        metadata: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subcategories::{CATEGORIES, LOGON_LOGOFF, SUBCATEGORIES};

    const LOGON_GUID: u128 = 0x0CCE_9215_69AE_11D9_BED3_5050_5450_3030;
    const BOGUS_GUID: u128 = 0x1111_1111_2222_3333_4444_5555_5555_5555;

    fn is_elevated() -> bool {
        get_security_context() == SecurityContext::Admin
    }

    #[test]
    fn table_values_map_to_the_expected_guid_fields() {
        let guid = GUID::from_u128(LOGON_GUID);
        assert_eq!(guid.data1, 0x0CCE_9215);
        assert_eq!(guid.data2, 0x69AE);
        assert_eq!(guid.data3, 0x11D9);
        assert_eq!(guid.data4, [0xBE, 0xD3, 0x50, 0x50, 0x54, 0x50, 0x30, 0x30]);
        assert_eq!(guid.to_u128(), LOGON_GUID);
        assert_eq!(format_guid(guid.to_u128()), "{0CCE9215-69AE-11D9-BED3-505054503030}");
    }

    // Enumeration and name lookup do not need elevation, so these run in any context.
    #[test]
    fn enumerates_categories_and_subcategories() {
        let categories = enumerate_categories().unwrap();
        assert_eq!(categories.as_slice().len(), CATEGORIES.len());
        for category in categories.as_slice() {
            assert!(
                subcategories::category_name(category.to_u128()).is_some(),
                "unknown category {}",
                format_guid(category.to_u128())
            );
        }

        let all = enumerate_subcategories(None).unwrap();
        let known = all
            .as_slice()
            .iter()
            .filter(|guid| subcategories::find_by_guid(guid.to_u128()).is_some())
            .count();
        assert!(known >= 50, "only {known} of {} reported subcategories are in the table", all.as_slice().len());

        let logon_logoff = enumerate_subcategories(Some(&GUID::from_u128(LOGON_LOGOFF.guid))).unwrap();
        assert!(logon_logoff.as_slice().iter().any(|guid| guid.to_u128() == LOGON_GUID));
        assert!(logon_logoff.as_slice().len() < all.as_slice().len());
        assert!(logon_logoff.as_slice().len() <= SUBCATEGORIES.len());
    }

    #[test]
    fn looks_up_names_for_known_guids_only() {
        assert!(!lookup_subcategory_name(&GUID::from_u128(LOGON_GUID)).unwrap().is_empty());
        assert!(!lookup_category_name(&GUID::from_u128(LOGON_LOGOFF.guid)).unwrap().is_empty());
        assert!(lookup_subcategory_name(&GUID::from_u128(BOGUS_GUID)).is_err());
        assert!(lookup_category_name(&GUID::from_u128(BOGUS_GUID)).is_err());
    }

    #[test]
    fn resolves_names_and_guids() {
        let by_name = resolve_subcategory(" lOgOn ").unwrap();
        assert_eq!(by_name.guid.to_u128(), LOGON_GUID);
        assert_eq!(by_name.name, "Logon");
        assert_eq!(by_name.category.as_deref(), Some("Logon/Logoff"));

        let by_guid = resolve_subcategory("{0cce9215-69ae-11d9-bed3-505054503030}").unwrap();
        assert_eq!(by_guid.guid.to_u128(), LOGON_GUID);
        assert_eq!(by_guid.name, "Logon");

        let bare = resolve_subcategory("0CCE9215-69AE-11D9-BED3-505054503030").unwrap();
        assert_eq!(bare.guid.to_u128(), LOGON_GUID);

        assert!(matches!(resolve_subcategory("Bogus Subcategory"), Err(AuditError::InvalidInput(_))));
        assert!(matches!(resolve_subcategory(&format_guid(BOGUS_GUID)), Err(AuditError::InvalidInput(_))));
    }

    fn entry(subcategory: &str, success: Option<bool>, failure: Option<bool>) -> AuditPolicy {
        AuditPolicy {
            subcategory: subcategory.to_string(),
            success,
            failure,
            ..AuditPolicy::default()
        }
    }

    #[test]
    fn desired_flags_keep_omitted_flags_and_describe_each_change() {
        let current = AuditFlags { success: true, failure: false };

        let (desired, messages) = desired_flags("Logon", current, &entry("logon", None, None));
        assert_eq!(desired, current);
        assert!(messages.is_empty());

        let (desired, messages) = desired_flags("Logon", current, &entry("logon", None, Some(true)));
        assert_eq!(desired, AuditFlags { success: true, failure: true });
        assert_eq!(
            messages,
            [t!("audit.whatIfChangeFailure", subcategory = "Logon", current = "false", desired = "true").to_string()]
        );

        let (desired, messages) = desired_flags("Logon", current, &entry("logon", Some(false), Some(true)));
        assert_eq!(desired, AuditFlags { success: false, failure: true });
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], t!("audit.whatIfChangeSuccess", subcategory = "Logon", current = "true", desired = "false"));

        let (_, messages) = desired_flags("Logon", current, &entry("logon", Some(true), Some(false)));
        assert!(messages.is_empty(), "flags that already match produce no messages");
    }

    #[test]
    fn list_entries_resolve_in_order() {
        let resolved = resolve_list(&[entry("Logoff", None, None), entry(" lOgOn ", None, None)]).unwrap();
        let names: Vec<&str> = resolved.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["Logoff", "Logon"]);
        assert!(resolve_list(&[]).unwrap().is_empty());
    }

    #[test]
    fn list_rejects_a_subcategory_named_twice_by_any_spelling() {
        for duplicate in ["Logon", "LOGON", "{0CCE9215-69AE-11D9-BED3-505054503030}", "0cce9215-69ae-11d9-bed3-505054503030"] {
            let error = resolve_list(&[entry("Logon", Some(true), None), entry(duplicate, None, Some(true))]).unwrap_err();
            assert!(matches!(error, AuditError::InvalidInput(_)), "{duplicate}");
            assert_eq!(error.to_string(), t!("audit.duplicateSubcategory", subcategory = "Logon"), "{duplicate}");
        }
        assert!(matches!(
            resolve_list(&[entry("Logon", None, None), entry("Bogus Subcategory", None, None)]),
            Err(AuditError::InvalidInput(_))
        ));
    }

    #[test]
    fn privilege_and_query_follow_the_security_context() {
        let privilege = enable_security_privilege();
        let query = query_policy(&[GUID::from_u128(LOGON_GUID)]);
        let input = AuditPolicy {
            subcategory: "Logon".to_string(),
            ..AuditPolicy::default()
        };
        let list = AuditPolicyList {
            policies: vec![entry("Logon", None, None), entry("Logoff", None, None)],
            metadata: None,
        };
        if is_elevated() {
            privilege.unwrap();
            assert_eq!(query.unwrap().as_slice().len(), 1);
            let policy = get_audit_policy(&input).unwrap();
            assert_eq!(policy.guid.as_deref(), Some("{0CCE9215-69AE-11D9-BED3-505054503030}"));
            assert_eq!(policy.category.as_deref(), Some("Logon/Logoff"));
            assert!(policy.success.is_some() && policy.failure.is_some());

            let current = get_audit_policy_list(&list).unwrap();
            let names: Vec<&str> = current.policies.iter().map(|p| p.subcategory.as_str()).collect();
            assert_eq!(names, ["Logon", "Logoff"]);
            let preview = set_audit_policy_list(&list, true).unwrap();
            assert!(preview.metadata.is_none(), "an entry without flags changes nothing");
            assert_eq!(export_audit_policy_list().unwrap().policies.len(), export_audit_policies().unwrap().len());
        } else {
            assert!(matches!(privilege, Err(AuditError::ElevationRequired)));
            assert!(query.is_err());
            assert!(matches!(get_audit_policy(&input), Err(AuditError::ElevationRequired)));
            assert!(matches!(set_audit_policy(&input, true), Err(AuditError::ElevationRequired)));
            assert!(matches!(export_audit_policies(), Err(AuditError::ElevationRequired)));
            assert!(matches!(get_audit_policy_list(&list), Err(AuditError::ElevationRequired)));
            assert!(matches!(set_audit_policy_list(&list, true), Err(AuditError::ElevationRequired)));
            assert!(matches!(export_audit_policy_list(), Err(AuditError::ElevationRequired)));
        }
    }

    #[test]
    fn empty_query_returns_an_empty_buffer() {
        assert!(query_policy(&[]).unwrap().as_slice().is_empty());
    }
}
