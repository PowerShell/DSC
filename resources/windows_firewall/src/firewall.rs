// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use windows::core::{BSTR, Interface};
use windows::core::HRESULT;
use windows::Win32::Foundation::{S_FALSE, VARIANT_BOOL};
use windows::Win32::NetworkManagement::WindowsFirewall::*;
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoInitializeEx, CoUninitialize, IDispatch, COINIT_APARTMENTTHREADED};
use windows::Win32::System::Ole::IEnumVARIANT;
use windows::Win32::System::Variant::{VARIANT, VariantClear};

use crate::types::{FirewallError, FirewallRule, FirewallRuleList, RuleAction, RuleDirection};
use crate::util::matches_any_filter;

/// RAII wrapper for VARIANT that automatically calls VariantClear on drop
struct SafeVariant(VARIANT);

impl SafeVariant {
    fn new() -> Self {
        Self(VARIANT::default())
    }

    fn as_mut_ptr(&mut self) -> *mut VARIANT {
        &mut self.0
    }

    fn as_ref(&self) -> &VARIANT {
        &self.0
    }
}

impl Drop for SafeVariant {
    fn drop(&mut self) {
        if let Err(e) = unsafe { VariantClear(&mut self.0) } {
            crate::write_error(&format!("Warning: VariantClear failed with HRESULT: {:#010x}", e.code().0));
        }
    }
}

struct ComGuard;

impl ComGuard {
    fn new() -> Result<Self, FirewallError> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
            .ok()
            .map_err(|error| t!("firewall.comInitFailed", error = error.to_string()).to_string())?;
        Ok(Self)
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

struct FirewallStore {
    rules: INetFwRules,
    _com: ComGuard,
}

impl FirewallStore {
    fn open() -> Result<Self, FirewallError> {
        let com = ComGuard::new()?;
        let policy: INetFwPolicy2 = unsafe { CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER) }
            .map_err(|error| t!("firewall.policyOpenFailed", error = error.to_string()).to_string())?;
        let rules = unsafe { policy.Rules() }
            .map_err(|error| t!("firewall.policyOpenFailed", error = error.to_string()).to_string())?;
        Ok(Self { rules, _com: com })
    }

    fn enumerate_rules(&self) -> Result<Vec<INetFwRule>, FirewallError> {
        let enumerator = unsafe { self.rules._NewEnum() }
            .map_err(|error| t!("firewall.ruleEnumerationFailed", error = error.to_string()).to_string())?;
        let enum_variant: IEnumVARIANT = enumerator
            .cast()
            .map_err(|error| t!("firewall.ruleEnumerationFailed", error = error.to_string()).to_string())?;

        let mut results = Vec::new();
        loop {
            let mut fetched = 0u32;
            let mut safe_variant = SafeVariant::new();
            let variant_slice = unsafe { std::slice::from_raw_parts_mut(safe_variant.as_mut_ptr(), 1) };
            let hr = unsafe { enum_variant.Next(variant_slice, &mut fetched) };
            if hr == S_FALSE || fetched == 0 {
                break;
            }
            hr.ok()
                .map_err(|error| t!("firewall.ruleEnumerationFailed", error = error.to_string()).to_string())?;

            let dispatch = IDispatch::try_from(safe_variant.as_ref())
                .map_err(|error: windows::core::Error| t!("firewall.ruleEnumerationFailed", error = error.to_string()).to_string())?;
            let rule: INetFwRule = dispatch
                .cast()
                .map_err(|error| t!("firewall.ruleEnumerationFailed", error = error.to_string()).to_string())?;
            results.push(rule);

            // SafeVariant will automatically call VariantClear when it goes out of scope
        }

        Ok(results)
    }

    fn find_by_selector(&self, selector: &FirewallRule) -> Result<Option<INetFwRule>, FirewallError> {
        // HRESULT 0x80070002 is HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND), returned when the
        // rule name does not match any existing rule.
        const HRESULT_FILE_NOT_FOUND: HRESULT = HRESULT(0x80070002_u32 as i32);

        let Some(lookup_name) = selector.selector_name() else {
            return Ok(None);
        };

        match unsafe { self.rules.Item(&BSTR::from(lookup_name)) } {
            Ok(rule) => Ok(Some(rule)),
            Err(e) if e.code() == HRESULT_FILE_NOT_FOUND => Ok(None),
            Err(e) => Err(t!("firewall.ruleLookupFailed", name = lookup_name, error = e.to_string()).to_string().into()),
        }
    }

    fn remove_rule(&self, rule_name: &str) -> Result<(), FirewallError> {
        unsafe { self.rules.Remove(&BSTR::from(rule_name)) }
            .map_err(|error| t!("firewall.ruleRemoveFailed", name = rule_name, error = error.to_string()).to_string())?;
        Ok(())
    }

    fn create_rule_object(&self) -> Result<INetFwRule, FirewallError> {
        unsafe { CoCreateInstance(&NetFwRule, None, CLSCTX_INPROC_SERVER) }
            .map_err(|error| t!("firewall.ruleCreateFailed", error = error.to_string()).to_string().into())
    }
}

fn bstr_to_option(value: BSTR) -> Result<Option<String>, FirewallError> {
    let text = value.to_string();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

fn native_direction_to_model(direction: NET_FW_RULE_DIRECTION) -> Option<RuleDirection> {
    if direction == NET_FW_RULE_DIR_IN {
        Some(RuleDirection::Inbound)
    } else if direction == NET_FW_RULE_DIR_OUT {
        Some(RuleDirection::Outbound)
    } else {
        None
    }
}

fn model_direction_to_native(direction: &RuleDirection) -> NET_FW_RULE_DIRECTION {
    match direction {
        RuleDirection::Inbound => NET_FW_RULE_DIR_IN,
        RuleDirection::Outbound => NET_FW_RULE_DIR_OUT,
    }
}

fn native_action_to_model(action: NET_FW_ACTION) -> Option<RuleAction> {
    if action == NET_FW_ACTION_ALLOW {
        Some(RuleAction::Allow)
    } else if action == NET_FW_ACTION_BLOCK {
        Some(RuleAction::Block)
    } else {
        None
    }
}

fn model_action_to_native(action: &RuleAction) -> NET_FW_ACTION {
    match action {
        RuleAction::Allow => NET_FW_ACTION_ALLOW,
        RuleAction::Block => NET_FW_ACTION_BLOCK,
    }
}

/// Converts a bitmask of firewall profile flags to a list of profile name strings.
///
/// `NET_FW_PROFILE2_ALL` is `0x7FFFFFFF` (a sentinel meaning all profiles), while the
/// combination of the three individual profile bits is `0x7`. Both are normalized to `["All"]`.
fn profiles_from_mask(mask: i32) -> Vec<String> {
    let all_bits = NET_FW_PROFILE2_DOMAIN.0 | NET_FW_PROFILE2_PRIVATE.0 | NET_FW_PROFILE2_PUBLIC.0;
    if mask == NET_FW_PROFILE2_ALL.0 || mask == all_bits {
        return vec!["All".to_string()];
    }

    let mut profiles = Vec::new();
    if mask & NET_FW_PROFILE2_DOMAIN.0 != 0 {
        profiles.push("Domain".to_string());
    }
    if mask & NET_FW_PROFILE2_PRIVATE.0 != 0 {
        profiles.push("Private".to_string());
    }
    if mask & NET_FW_PROFILE2_PUBLIC.0 != 0 {
        profiles.push("Public".to_string());
    }
    profiles
}

fn profiles_to_mask(values: &[String]) -> Result<i32, FirewallError> {
    if values.is_empty() {
        return Ok(NET_FW_PROFILE2_ALL.0);
    }

    let mut mask = 0;
    for value in values {
        match value.to_ascii_lowercase().as_str() {
            "all" => return Ok(NET_FW_PROFILE2_ALL.0),
            "domain" => mask |= NET_FW_PROFILE2_DOMAIN.0,
            "private" => mask |= NET_FW_PROFILE2_PRIVATE.0,
            "public" => mask |= NET_FW_PROFILE2_PUBLIC.0,
            _ => return Err(t!("firewall.invalidProfiles", value = value).to_string().into()),
        }
    }
    Ok(mask)
}

fn split_csv(value: Option<String>) -> Option<Vec<String>> {
    value.map(|raw| {
        raw.split(',')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<String>>()
    }).filter(|items| !items.is_empty())
}

fn join_csv(value: &[String]) -> String {
    value.join(",")
}

fn interface_types_to_string(values: &[String]) -> Result<String, FirewallError> {
    if values.is_empty() {
        return Ok("All".to_string());
    }

    let mut normalized = Vec::new();
    for value in values {
        match value.to_ascii_lowercase().as_str() {
            "all" => return Ok("All".to_string()),
            "remoteaccess" => normalized.push("RemoteAccess".to_string()),
            "wireless" => normalized.push("Wireless".to_string()),
            "lan" => normalized.push("Lan".to_string()),
            _ => return Err(t!("firewall.invalidInterfaceType", value = value).to_string().into()),
        }
    }
    Ok(join_csv(&normalized))
}

fn protocol_supports_ports(protocol: i32) -> bool {
    protocol == NET_FW_IP_PROTOCOL_TCP.0 || protocol == NET_FW_IP_PROTOCOL_UDP.0
}

fn validate_protocol(protocol: i32) -> Result<(), FirewallError> {
    // IANA protocol numbers 0-255 plus the Windows-specific 256 (Any)
    if !(0..=256).contains(&protocol) {
        return Err(t!("firewall.invalidProtocol", value = protocol).to_string().into());
    }
    Ok(())
}

fn map_update_err(name: &str) -> impl Fn(windows::core::Error) -> FirewallError + '_ {
    move |error| t!("firewall.ruleUpdateFailed", name = name, error = error.to_string()).to_string().into()
}

fn map_read_err(name: &str) -> impl Fn(windows::core::Error) -> FirewallError + '_ {
    move |error| t!("firewall.ruleReadFailed", name = name, error = error.to_string()).to_string().into()
}

fn rule_to_model(rule: &INetFwRule) -> Result<FirewallRule, FirewallError> {
    let name = unsafe { rule.Name() }.map_err(|error| t!("firewall.ruleReadFailed", name = "<unknown>", error = error.to_string()).to_string())?;
    let name = name.to_string();
    let err = map_read_err(&name);
    let profiles = profiles_from_mask(unsafe { rule.Profiles() }.map_err(&err)?);

    Ok(FirewallRule {
        name: Some(name.clone()),
        exist: None,
        description: bstr_to_option(unsafe { rule.Description() }.map_err(&err)?)?,
        application_name: bstr_to_option(unsafe { rule.ApplicationName() }.map_err(&err)?)?,
        service_name: bstr_to_option(unsafe { rule.ServiceName() }.map_err(&err)?)?,
        protocol: Some(unsafe { rule.Protocol() }.map_err(&err)?),
        local_ports: bstr_to_option(unsafe { rule.LocalPorts() }.map_err(&err)?)?,
        remote_ports: bstr_to_option(unsafe { rule.RemotePorts() }.map_err(&err)?)?,
        local_addresses: bstr_to_option(unsafe { rule.LocalAddresses() }.map_err(&err)?)?,
        remote_addresses: bstr_to_option(unsafe { rule.RemoteAddresses() }.map_err(&err)?)?,
        direction: native_direction_to_model(unsafe { rule.Direction() }.map_err(&err)?),
        action: native_action_to_model(unsafe { rule.Action() }.map_err(&err)?),
        enabled: Some(unsafe { rule.Enabled() }.map_err(&err)?.as_bool()),
        profiles: Some(profiles),
        grouping: bstr_to_option(unsafe { rule.Grouping() }.map_err(&err)?)?,
        interface_types: split_csv(bstr_to_option(unsafe { rule.InterfaceTypes() }.map_err(&err)?)?),
        edge_traversal: Some(unsafe { rule.EdgeTraversal() }.map_err(&err)?.as_bool()),
    })
}

fn apply_rule_properties(rule: &INetFwRule, desired: &FirewallRule, existing_protocol: Option<i32>) -> Result<(), FirewallError> {
    let name = desired.selector_name().unwrap_or("<unknown>");
    let err = map_update_err(name);

    if let Some(protocol) = desired.protocol {
        validate_protocol(protocol)?;
    }

    // Determine the effective protocol: the desired value if provided, otherwise
    // the existing rule's protocol (if updating an existing rule).
    let effective_protocol = desired.protocol.or(existing_protocol);

    // If effective_protocol is None, read the current protocol from the rule.
    let effective_protocol = match effective_protocol {
        Some(protocol) => Some(protocol),
        None => Some(unsafe { rule.Protocol() }.map_err(&err)?),
    };

    // Reject port specifications for protocols that don't support them (e.g. ICMP).
    // This must be checked regardless of whether the protocol itself was changed,
    // because the caller may only be setting local_ports or remote_ports.
    if let Some(protocol) = effective_protocol
        && !protocol_supports_ports(protocol)
        && (desired.local_ports.is_some() || desired.remote_ports.is_some()) {
            return Err(t!("firewall.portsNotAllowed", name = name, protocol = protocol).to_string().into());
        }

    if let Some(protocol) = desired.protocol {
        if let Some(current_protocol) = existing_protocol
            && current_protocol != protocol && !protocol_supports_ports(protocol) {
                if desired.local_ports.is_none() {
                    unsafe { rule.SetLocalPorts(&BSTR::from("")) }.map_err(&err)?;
                }
                if desired.remote_ports.is_none() {
                    unsafe { rule.SetRemotePorts(&BSTR::from("")) }.map_err(&err)?;
                }
            }
        unsafe { rule.SetProtocol(protocol) }.map_err(&err)?;
    }

    if let Some(description) = desired.description.as_ref() {
        unsafe { rule.SetDescription(&BSTR::from(description.as_str())) }.map_err(&err)?;
    }
    if let Some(application_name) = desired.application_name.as_ref() {
        unsafe { rule.SetApplicationName(&BSTR::from(application_name.as_str())) }.map_err(&err)?;
    }
    if let Some(service_name) = desired.service_name.as_ref() {
        unsafe { rule.SetServiceName(&BSTR::from(service_name.as_str())) }.map_err(&err)?;
    }
    if let Some(local_ports) = desired.local_ports.as_ref() {
        unsafe { rule.SetLocalPorts(&BSTR::from(local_ports.as_str())) }.map_err(&err)?;
    }
    if let Some(remote_ports) = desired.remote_ports.as_ref() {
        unsafe { rule.SetRemotePorts(&BSTR::from(remote_ports.as_str())) }.map_err(&err)?;
    }
    if let Some(local_addresses) = desired.local_addresses.as_ref() {
        unsafe { rule.SetLocalAddresses(&BSTR::from(local_addresses.as_str())) }.map_err(&err)?;
    }
    if let Some(remote_addresses) = desired.remote_addresses.as_ref() {
        unsafe { rule.SetRemoteAddresses(&BSTR::from(remote_addresses.as_str())) }.map_err(&err)?;
    }
    if let Some(direction) = desired.direction.as_ref() {
        unsafe { rule.SetDirection(model_direction_to_native(direction)) }.map_err(&err)?;
    }
    if let Some(action) = desired.action.as_ref() {
        unsafe { rule.SetAction(model_action_to_native(action)) }.map_err(&err)?;
    }
    if let Some(enabled) = desired.enabled {
        unsafe { rule.SetEnabled(VARIANT_BOOL::from(enabled)) }.map_err(&err)?;
    }
    if let Some(profiles) = desired.profiles.as_ref() {
        let mask = profiles_to_mask(profiles)?;
        unsafe { rule.SetProfiles(mask) }.map_err(&err)?;
    }
    if let Some(grouping) = desired.grouping.as_ref() {
        unsafe { rule.SetGrouping(&BSTR::from(grouping.as_str())) }.map_err(&err)?;
    }
    if let Some(interface_types) = desired.interface_types.as_ref() {
        let value = interface_types_to_string(interface_types)?;
        unsafe { rule.SetInterfaceTypes(&BSTR::from(value.as_str())) }.map_err(&err)?;
    }
    if let Some(edge_traversal) = desired.edge_traversal {
        unsafe { rule.SetEdgeTraversal(VARIANT_BOOL::from(edge_traversal)) }.map_err(&err)?;
    }

    Ok(())
}

pub fn get_rules(input: &FirewallRuleList) -> Result<FirewallRuleList, FirewallError> {
    if input.rules.is_empty() {
        return Err(t!("get.rulesArrayEmpty").to_string().into());
    }

    let store = FirewallStore::open()?;
    let mut results = Vec::new();

    for desired in &input.rules {
        if desired.selector_name().is_none() {
            return Err(t!("get.selectorRequired").to_string().into());
        }

        match store.find_by_selector(desired)? {
            Some(rule) => results.push(rule_to_model(&rule)?),
            None => results.push(desired.missing_from_input()),
        }
    }

    Ok(FirewallRuleList { rules: results })
}

pub fn set_rules(input: &FirewallRuleList) -> Result<FirewallRuleList, FirewallError> {
    if input.rules.is_empty() {
        return Err(t!("set.rulesArrayEmpty").to_string().into());
    }

    let store = FirewallStore::open()?;
    let mut results = Vec::new();

    for desired in &input.rules {
        if desired.selector_name().is_none() {
            return Err(t!("set.selectorRequired").to_string().into());
        }

        match store.find_by_selector(desired)? {
            Some(rule) => {
                let current = rule_to_model(&rule)?;
                let rule_name = current.name.clone().unwrap_or_else(|| desired.selector_name().unwrap_or_default().to_string());

                if desired.exist == Some(false) {
                    store.remove_rule(&rule_name)?;
                    results.push(desired.missing_from_input());
                    continue;
                }

                apply_rule_properties(&rule, desired, current.protocol)?;
                results.push(rule_to_model(&rule)?);
            }
            None => {
                if desired.exist == Some(false) {
                    results.push(desired.missing_from_input());
                    continue;
                }

                let rule_name = desired.name.clone()
                    .ok_or_else(|| t!("set.selectorRequired").to_string())?;
                let rule = store.create_rule_object()?;
                unsafe { rule.SetName(&BSTR::from(rule_name.as_str())) }
                    .map_err(|error| t!("firewall.ruleAddFailed", name = rule_name.as_str(), error = error.to_string()).to_string())?;

                apply_rule_properties(&rule, desired, None)?;
                unsafe { store.rules.Add(&rule) }
                    .map_err(|error| t!("firewall.ruleAddFailed", name = rule_name.as_str(), error = error.to_string()).to_string())?;

                let created = store
                    .find_by_selector(&FirewallRule {
                        name: Some(rule_name),
                        ..FirewallRule::default()
                    })?
                    .ok_or_else(|| t!("firewall.ruleLookupFailed", name = desired.selector_name().unwrap_or("<unknown>"), error = "created rule not found").to_string())?;
                results.push(rule_to_model(&created)?);
            }
        }
    }

    Ok(FirewallRuleList { rules: results })
}

pub fn export_rules(filters: Option<&FirewallRuleList>) -> Result<FirewallRuleList, FirewallError> {
    let store = FirewallStore::open()?;
    let all_rules = store.enumerate_rules()?;
    let default_filter;
    let filter_rules: &[FirewallRule] = match filters {
        Some(input) if !input.rules.is_empty() => &input.rules,
        _ => { default_filter = [FirewallRule::default()]; &default_filter }
    };

    let mut results = Vec::new();
    for rule in all_rules {
        let model = rule_to_model(&rule)?;
        if matches_any_filter(&model, filter_rules) {
            results.push(model);
        }
    }

    Ok(FirewallRuleList { rules: results })
}
