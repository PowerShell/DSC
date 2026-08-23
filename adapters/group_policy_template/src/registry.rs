// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::admx::{
    AdapterError, ElementKind, Policy, PolicyClass, PolicyElement, PolicyValue, load_resource,
    policy_value_to_json, registry_value_to_json,
};
use dsc_lib_registry::{RegistryHelper, config::RegistryValueData};
use rust_i18n::t;
use serde_json::{Map, Value};
use std::path::Path;

const DEFAULT_SCOPE: &str = "currentUser";

pub fn get(
    input: &str,
    resource_type: &str,
    resource_path: &str,
) -> Result<Vec<String>, AdapterError> {
    let input = parse_input(input)?;
    let scope = parse_scope(&input)?;
    let resource = load_resource(Path::new(resource_path), resource_type)?;
    let include_all = input.keys().all(|key| key == "scope");
    let mut result = Map::new();
    result.insert("scope".to_string(), Value::String(scope.to_string()));

    for policy in &resource.policies {
        if !include_all && !input.contains_key(&policy.name) {
            continue;
        }
        if include_all && !scope_is_supported(policy, scope) {
            continue;
        }
        let requested = input.get(&policy.name);
        if let Some(value) = read_policy(policy, scope, requested)? {
            result.insert(policy.name.clone(), value);
        }
    }

    serialize_result(&result)
}

pub fn set(
    input: &str,
    resource_type: &str,
    resource_path: &str,
) -> Result<Vec<String>, AdapterError> {
    let input = parse_input(input)?;
    let scope = parse_scope(&input)?;
    let resource = load_resource(Path::new(resource_path), resource_type)?;

    for (name, value) in &input {
        if name == "scope" {
            continue;
        }
        let policy = resource
            .policies
            .iter()
            .find(|policy| policy.name == *name)
            .ok_or_else(|| {
                AdapterError::Input(
                    t!(
                        "registry.unknownPolicy",
                        policy = name,
                        resource = resource_type
                    )
                    .to_string(),
                )
            })?;
        validate_scope(policy, scope)?;
        write_policy(policy, scope, value)?;
    }

    get(
        input_to_string(&input)?.as_str(),
        resource_type,
        resource_path,
    )
}

fn read_policy(
    policy: &Policy,
    scope: &str,
    requested: Option<&Value>,
) -> Result<Option<Value>, AdapterError> {
    validate_scope(policy, scope)?;
    if let Some(Value::Object(requested_elements)) = requested {
        let mut result = Map::new();
        if requested_elements.contains_key("enabled")
            && let Some(enabled) = read_enabled(policy, scope)?
        {
            result.insert("enabled".to_string(), Value::Bool(enabled));
        }
        for element in &policy.elements {
            let Some(requested_value) = requested_elements.get(&element.id) else {
                continue;
            };
            if let Some(value) = read_element(policy, element, scope, requested_value)? {
                result.insert(element.id.clone(), value);
            }
        }
        return Ok(Some(Value::Object(result)));
    }
    read_enabled(policy, scope).map(|value| value.map(Value::Bool))
}

fn read_enabled(policy: &Policy, scope: &str) -> Result<Option<bool>, AdapterError> {
    if state_matches(policy, scope, true)? {
        Ok(Some(true))
    } else if state_matches(policy, scope, false)? {
        Ok(Some(false))
    } else if policy_values_are_absent(policy, scope)? {
        Ok(None)
    } else {
        Err(AdapterError::Resource(
            t!(
                "registry.unrecognizedValue",
                key = policy.key,
                value_name = policy.value_name.as_deref().unwrap_or_default()
            )
            .to_string(),
        ))
    }
}

fn policy_values_are_absent(policy: &Policy, scope: &str) -> Result<bool, AdapterError> {
    let mut values = Vec::new();
    if let Some(value_name) = &policy.value_name {
        values.push((policy.key.as_str(), value_name.as_str()));
    }
    for setting in policy.enabled_list.iter().chain(&policy.disabled_list) {
        values.push((
            setting.key.as_deref().unwrap_or(&policy.key),
            setting.value_name.as_str(),
        ));
    }
    if values.is_empty() {
        return Ok(false);
    }

    for (key, value_name) in values {
        let actual = RegistryHelper::new(&key_path(scope, key), Some(value_name.to_string()), None)
            .map_err(registry_error)?
            .get()
            .map_err(registry_error)?
            .value_data;
        if actual.is_some() {
            return Ok(false);
        }
    }
    Ok(true)
}

fn write_policy(policy: &Policy, scope: &str, input: &Value) -> Result<(), AdapterError> {
    if let Some(enabled) = input.as_bool() {
        return write_enabled(policy, scope, enabled);
    }
    let object = input.as_object().ok_or_else(|| {
        AdapterError::Input(t!("registry.invalidPolicyValue", policy = policy.name).to_string())
    })?;
    if let Some(enabled) = object.get("enabled") {
        write_enabled(
            policy,
            scope,
            enabled.as_bool().ok_or_else(|| {
                AdapterError::Input(
                    t!("registry.policyNotBoolean", policy = policy.name).to_string(),
                )
            })?,
        )?;
    }
    for (name, value) in object {
        if name == "enabled" {
            continue;
        }
        let element = policy
            .elements
            .iter()
            .find(|element| element.id == *name)
            .ok_or_else(|| {
                AdapterError::Input(
                    t!(
                        "registry.unknownElement",
                        element = name,
                        policy = policy.name
                    )
                    .to_string(),
                )
            })?;
        write_element(policy, element, scope, value)?;
    }
    Ok(())
}

fn write_enabled(policy: &Policy, scope: &str, enabled: bool) -> Result<(), AdapterError> {
    let value = if enabled {
        policy.enabled.as_ref()
    } else {
        policy.disabled.as_ref()
    };
    let list = if enabled {
        &policy.enabled_list
    } else {
        &policy.disabled_list
    };
    if value.is_none() && list.is_empty() {
        return Err(AdapterError::Input(
            t!("registry.policyHasNoToggle", policy = policy.name).to_string(),
        ));
    }
    if let (Some(value_name), Some(value)) = (&policy.value_name, value) {
        apply_value(scope, &policy.key, value_name, value)?;
    }
    for setting in list {
        apply_value(
            scope,
            setting.key.as_deref().unwrap_or(&policy.key),
            &setting.value_name,
            &setting.value,
        )?;
    }
    Ok(())
}

fn apply_value(
    scope: &str,
    key: &str,
    value_name: &str,
    value: &PolicyValue,
) -> Result<(), AdapterError> {
    let path = key_path(scope, key);
    match value {
        PolicyValue::Data(data) => {
            RegistryHelper::new(&path, Some(value_name.to_string()), Some(data.clone()))
                .map_err(registry_error)?
                .set()
                .map_err(registry_error)?;
        }
        PolicyValue::Delete => {
            let registry = dsc_lib_registry::config::Registry {
                key_path: path,
                value_name: Some(value_name.to_string()),
                exist: Some(false),
                ..Default::default()
            };
            let helper = RegistryHelper::new_from_registry(&registry).map_err(registry_error)?;
            helper.remove().map_err(registry_error)?;
        }
    }
    Ok(())
}

fn state_matches(policy: &Policy, scope: &str, enabled: bool) -> Result<bool, AdapterError> {
    let value = if enabled {
        policy.enabled.as_ref()
    } else {
        policy.disabled.as_ref()
    };
    let list = if enabled {
        &policy.enabled_list
    } else {
        &policy.disabled_list
    };
    if value.is_none() && list.is_empty() {
        return Ok(false);
    }
    if let (Some(value_name), Some(value)) = (&policy.value_name, value)
        && !value_matches(scope, &policy.key, value_name, value)?
    {
        return Ok(false);
    }
    for setting in list {
        if !value_matches(
            scope,
            setting.key.as_deref().unwrap_or(&policy.key),
            &setting.value_name,
            &setting.value,
        )? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn value_matches(
    scope: &str,
    key: &str,
    value_name: &str,
    expected: &PolicyValue,
) -> Result<bool, AdapterError> {
    let actual = RegistryHelper::new(&key_path(scope, key), Some(value_name.to_string()), None)
        .map_err(registry_error)?
        .get()
        .map_err(registry_error)?
        .value_data;
    Ok(match (expected, actual) {
        (PolicyValue::Delete, None) => true,
        (PolicyValue::Data(expected), Some(actual)) => expected == &actual,
        _ => false,
    })
}

fn read_element(
    policy: &Policy,
    element: &PolicyElement,
    scope: &str,
    requested: &Value,
) -> Result<Option<Value>, AdapterError> {
    let element_key = element.key.as_deref().unwrap_or(&policy.key);
    if matches!(element.kind, ElementKind::List) {
        let requested_values = requested.as_object().ok_or_else(|| {
            AdapterError::Input(t!("registry.listNotObject", element = element.id).to_string())
        })?;
        let mut result = Map::new();
        for value_name in requested_values.keys() {
            let helper = RegistryHelper::new(
                &key_path(scope, element_key),
                Some(value_name.clone()),
                None,
            )
            .map_err(registry_error)?;
            if let Some(data) = helper.get().map_err(registry_error)?.value_data {
                result.insert(value_name.clone(), registry_value_to_json(&data));
            }
        }
        return Ok(Some(Value::Object(result)));
    }
    let value_name = element.value_name.as_ref().ok_or_else(|| {
        AdapterError::Resource(
            t!("registry.elementHasNoValueName", element = element.id).to_string(),
        )
    })?;
    let helper = RegistryHelper::new(
        &key_path(scope, element_key),
        Some(value_name.clone()),
        None,
    )
    .map_err(registry_error)?;
    let Some(data) = helper.get().map_err(registry_error)?.value_data else {
        return Ok(None);
    };
    Ok(Some(element_data_to_json(element, &data)?))
}

fn write_element(
    policy: &Policy,
    element: &PolicyElement,
    scope: &str,
    value: &Value,
) -> Result<(), AdapterError> {
    let element_key = element.key.as_deref().unwrap_or(&policy.key);
    if matches!(element.kind, ElementKind::List) {
        let values = value.as_object().ok_or_else(|| {
            AdapterError::Input(t!("registry.listNotObject", element = element.id).to_string())
        })?;
        for (value_name, value) in values {
            let data = RegistryValueData::String(
                value
                    .as_str()
                    .ok_or_else(|| {
                        AdapterError::Input(
                            t!("registry.listValueNotString", element = element.id).to_string(),
                        )
                    })?
                    .to_string(),
            );
            RegistryHelper::new(
                &key_path(scope, element_key),
                Some(value_name.clone()),
                Some(data),
            )
            .map_err(registry_error)?
            .set()
            .map_err(registry_error)?;
        }
        return Ok(());
    }
    let value_name = element.value_name.as_ref().ok_or_else(|| {
        AdapterError::Resource(
            t!("registry.elementHasNoValueName", element = element.id).to_string(),
        )
    })?;
    match json_to_element_data(element, value)? {
        PolicyValue::Data(data) => {
            RegistryHelper::new(
                &key_path(scope, element_key),
                Some(value_name.clone()),
                Some(data),
            )
            .map_err(registry_error)?
            .set()
            .map_err(registry_error)?;
        }
        PolicyValue::Delete => {
            let registry = dsc_lib_registry::config::Registry {
                key_path: key_path(scope, element_key),
                value_name: Some(value_name.clone()),
                exist: Some(false),
                ..Default::default()
            };
            RegistryHelper::new_from_registry(&registry)
                .map_err(registry_error)?
                .remove()
                .map_err(registry_error)?;
        }
    }
    Ok(())
}

fn json_to_element_data(
    element: &PolicyElement,
    value: &Value,
) -> Result<PolicyValue, AdapterError> {
    match &element.kind {
        ElementKind::Boolean {
            true_value,
            false_value,
        } => value
            .as_bool()
            .map(|value| {
                if value {
                    true_value.clone()
                } else {
                    false_value.clone()
                }
            })
            .ok_or_else(|| invalid_element_value(element)),
        ElementKind::Decimal { store_as_text, .. } => {
            let number = value
                .as_u64()
                .ok_or_else(|| invalid_element_value(element))?;
            if *store_as_text {
                Ok(PolicyValue::Data(RegistryValueData::String(
                    number.to_string(),
                )))
            } else {
                u32::try_from(number)
                    .map(|value| PolicyValue::Data(RegistryValueData::DWord(value)))
                    .map_err(|_| invalid_element_value(element))
            }
        }
        ElementKind::Enum(items) => items
            .iter()
            .find(|item| policy_value_to_json(&item.value) == *value)
            .map(|item| item.value.clone())
            .ok_or_else(|| invalid_element_value(element)),
        ElementKind::MultiText => value
            .as_array()
            .and_then(|items| {
                items
                    .iter()
                    .map(|item| item.as_str().map(ToString::to_string))
                    .collect::<Option<Vec<_>>>()
            })
            .map(|value| PolicyValue::Data(RegistryValueData::MultiString(value)))
            .ok_or_else(|| invalid_element_value(element)),
        ElementKind::Text { expandable } => value
            .as_str()
            .map(|value| {
                if *expandable {
                    PolicyValue::Data(RegistryValueData::ExpandString(value.to_string()))
                } else {
                    PolicyValue::Data(RegistryValueData::String(value.to_string()))
                }
            })
            .ok_or_else(|| invalid_element_value(element)),
        ElementKind::List => Err(invalid_element_value(element)),
    }
}

fn element_data_to_json(
    element: &PolicyElement,
    data: &RegistryValueData,
) -> Result<Value, AdapterError> {
    match &element.kind {
        ElementKind::Boolean {
            true_value: PolicyValue::Data(value),
            false_value,
        } if data == value => Ok(Value::Bool(true)),
        ElementKind::Boolean {
            true_value: _,
            false_value: PolicyValue::Data(value),
        } if data == value => Ok(Value::Bool(false)),
        ElementKind::Enum(items)
            if items
                .iter()
                .any(|item| matches!(&item.value, PolicyValue::Data(value) if value == data)) =>
        {
            Ok(registry_value_to_json(data))
        }
        ElementKind::Decimal {
            store_as_text: true,
            ..
        } => match data {
            RegistryValueData::String(value) => value
                .parse::<u64>()
                .map(Value::from)
                .map_err(|_| invalid_element_value(element)),
            _ => Err(invalid_element_value(element)),
        },
        ElementKind::Decimal { .. }
        | ElementKind::MultiText
        | ElementKind::Text { .. }
        | ElementKind::List => Ok(registry_value_to_json(data)),
        _ => Err(invalid_element_value(element)),
    }
}

fn invalid_element_value(element: &PolicyElement) -> AdapterError {
    AdapterError::Input(t!("registry.invalidElementValue", element = element.id).to_string())
}

fn validate_scope(policy: &Policy, scope: &str) -> Result<(), AdapterError> {
    if scope_is_supported(policy, scope) {
        Ok(())
    } else {
        Err(AdapterError::Input(
            t!(
                "registry.scopeNotSupported",
                policy = policy.name,
                scope = scope
            )
            .to_string(),
        ))
    }
}

fn scope_is_supported(policy: &Policy, scope: &str) -> bool {
    matches!(
        (policy.class, scope),
        (PolicyClass::Both, _)
            | (PolicyClass::Machine, "allUsers")
            | (PolicyClass::User, "currentUser")
    )
}

fn parse_input(input: &str) -> Result<Map<String, Value>, AdapterError> {
    serde_json::from_str(input).map_err(|error| {
        AdapterError::Input(t!("registry.invalidInput", error = error).to_string())
    })
}

fn parse_scope(input: &Map<String, Value>) -> Result<&str, AdapterError> {
    match input.get("scope") {
        None => Ok(DEFAULT_SCOPE),
        Some(Value::String(scope)) if matches!(scope.as_str(), "allUsers" | "currentUser") => {
            Ok(scope)
        }
        _ => Err(AdapterError::Input(t!("registry.invalidScope").to_string())),
    }
}

fn key_path(scope: &str, key: &str) -> String {
    let hive = if scope == "allUsers" { "HKLM" } else { "HKCU" };
    format!("{hive}\\{key}")
}

fn registry_error(error: impl std::fmt::Display) -> AdapterError {
    AdapterError::Resource(t!("registry.operationFailed", error = error).to_string())
}

fn input_to_string(input: &Map<String, Value>) -> Result<String, AdapterError> {
    serde_json::to_string(input).map_err(|error| {
        AdapterError::Resource(t!("registry.serializeResult", error = error).to_string())
    })
}

fn serialize_result(result: &Map<String, Value>) -> Result<Vec<String>, AdapterError> {
    Ok(vec![serde_json::to_string(result).map_err(|error| {
        AdapterError::Resource(t!("registry.serializeResult", error = error).to_string())
    })?])
}

#[cfg(test)]
mod tests {
    use super::{
        apply_value, element_data_to_json, get, input_to_string, json_to_element_data, key_path,
        parse_input, parse_scope, policy_values_are_absent, read_policy, scope_is_supported,
        serialize_result, set, state_matches, validate_scope, write_policy,
    };
    use crate::admx::{ElementKind, EnumItem, Policy, PolicyClass, PolicyElement, PolicyValue};
    use dsc_lib_registry::{
        RegistryHelper,
        config::{Registry, RegistryValueData},
    };
    use serde_json::{Map, Value, json};

    fn element(id: &str, kind: ElementKind) -> PolicyElement {
        PolicyElement {
            id: id.to_string(),
            key: None,
            value_name: Some(id.to_string()),
            kind,
        }
    }

    fn policy(class: PolicyClass) -> Policy {
        Policy {
            name: "Policy".to_string(),
            display_name: "Policy".to_string(),
            description: None,
            class,
            key: "Software\\Fixture".to_string(),
            value_name: None,
            enabled: None,
            disabled: None,
            elements: Vec::new(),
            enabled_list: Vec::new(),
            disabled_list: Vec::new(),
        }
    }

    #[test]
    fn maps_scope_to_registry_hive() {
        assert_eq!(
            key_path("currentUser", "Software\\Policies"),
            "HKCU\\Software\\Policies"
        );
        assert_eq!(
            key_path("allUsers", "Software\\Policies"),
            "HKLM\\Software\\Policies"
        );
    }

    #[test]
    fn parses_input_scope_and_serializes_results() {
        let input = parse_input(r#"{"scope":"allUsers","Policy":true}"#).unwrap();
        assert_eq!(parse_scope(&input).unwrap(), "allUsers");
        assert_eq!(parse_scope(&Map::new()).unwrap(), "currentUser");
        assert!(parse_scope(&Map::from_iter([("scope".to_string(), json!("invalid"))])).is_err());
        assert!(parse_input("not json").is_err());
        assert_eq!(
            serde_json::from_str::<Value>(&input_to_string(&input).unwrap()).unwrap(),
            json!({"Policy": true, "scope": "allUsers"})
        );
        assert_eq!(
            serialize_result(&Map::from_iter([("value".to_string(), json!(1))])).unwrap(),
            vec![r#"{"value":1}"#]
        );
    }

    #[test]
    fn validates_policy_scope() {
        let both = policy(PolicyClass::Both);
        let machine = policy(PolicyClass::Machine);
        let user = policy(PolicyClass::User);
        assert!(scope_is_supported(&both, "currentUser"));
        assert!(scope_is_supported(&both, "allUsers"));
        assert!(scope_is_supported(&machine, "allUsers"));
        assert!(!scope_is_supported(&machine, "currentUser"));
        assert!(scope_is_supported(&user, "currentUser"));
        assert!(!scope_is_supported(&user, "allUsers"));
        assert!(validate_scope(&machine, "allUsers").is_ok());
        assert!(validate_scope(&machine, "currentUser").is_err());
        assert!(read_policy(&machine, "currentUser", None).is_err());
    }

    #[test]
    fn round_trips_policy_state_in_current_user_registry() {
        let key = format!(
            "Software\\Microsoft\\DSC\\GroupPolicyTemplateTests\\{}",
            std::process::id()
        );
        let mut registry_policy = policy(PolicyClass::User);
        registry_policy.key = key.clone();
        registry_policy.value_name = Some("State".to_string());
        registry_policy.enabled = Some(PolicyValue::Data(RegistryValueData::DWord(1)));
        registry_policy.disabled = Some(PolicyValue::Delete);
        registry_policy.elements = vec![
            element(
                "Boolean",
                ElementKind::Boolean {
                    true_value: PolicyValue::Data(RegistryValueData::DWord(1)),
                    false_value: PolicyValue::Data(RegistryValueData::DWord(0)),
                },
            ),
            element(
                "Decimal",
                ElementKind::Decimal {
                    minimum: None,
                    maximum: None,
                    store_as_text: false,
                },
            ),
            element(
                "Enum",
                ElementKind::Enum(vec![EnumItem {
                    title: "Choice".to_string(),
                    value: PolicyValue::Data(RegistryValueData::String("choice".to_string())),
                }]),
            ),
            element("Multi", ElementKind::MultiText),
            element("Text", ElementKind::Text { expandable: false }),
            PolicyElement {
                id: "List".to_string(),
                key: None,
                value_name: None,
                kind: ElementKind::List,
            },
        ];

        let desired = json!({
            "enabled": true,
            "Boolean": true,
            "Decimal": 42,
            "Enum": "choice",
            "Multi": ["one", "two"],
            "Text": "value",
            "List": {
                "ListOne": "first",
                "ListTwo": "second"
            }
        });

        let result = (|| {
            write_policy(&registry_policy, "currentUser", &desired)?;
            assert!(state_matches(&registry_policy, "currentUser", true)?);
            assert!(!policy_values_are_absent(&registry_policy, "currentUser")?);

            let actual = read_policy(&registry_policy, "currentUser", Some(&desired))?.unwrap();
            assert_eq!(actual, desired);

            write_policy(&registry_policy, "currentUser", &json!(false))?;
            assert_eq!(
                read_policy(&registry_policy, "currentUser", None)?,
                Some(Value::Bool(false))
            );
            Ok::<(), crate::admx::AdapterError>(())
        })();

        for value_name in [
            "State", "Boolean", "Decimal", "Enum", "Multi", "Text", "ListOne", "ListTwo",
        ] {
            apply_value("currentUser", &key, value_name, &PolicyValue::Delete).unwrap();
        }
        RegistryHelper::new_from_registry(&Registry {
            key_path: key_path("currentUser", &key),
            exist: Some(false),
            ..Default::default()
        })
        .unwrap()
        .remove()
        .unwrap();
        result.unwrap();
    }

    #[test]
    fn public_operations_reject_invalid_input_before_loading_resources() {
        assert!(get("not json", "GPO.Parent/Category", "missing.admx").is_err());
        assert!(set("not json", "GPO.Parent/Category", "missing.admx").is_err());
    }

    #[test]
    fn converts_json_to_every_element_type() {
        let boolean = element(
            "Boolean",
            ElementKind::Boolean {
                true_value: PolicyValue::Data(RegistryValueData::DWord(10)),
                false_value: PolicyValue::Delete,
            },
        );
        assert_eq!(
            json_to_element_data(&boolean, &json!(true)).unwrap(),
            PolicyValue::Data(RegistryValueData::DWord(10))
        );
        assert_eq!(
            json_to_element_data(&boolean, &json!(false)).unwrap(),
            PolicyValue::Delete
        );
        assert!(json_to_element_data(&boolean, &json!("true")).is_err());

        let decimal = element(
            "Decimal",
            ElementKind::Decimal {
                minimum: None,
                maximum: None,
                store_as_text: false,
            },
        );
        assert_eq!(
            json_to_element_data(&decimal, &json!(42)).unwrap(),
            PolicyValue::Data(RegistryValueData::DWord(42))
        );
        assert!(json_to_element_data(&decimal, &json!(u64::from(u32::MAX) + 1)).is_err());

        let text_decimal = element(
            "TextDecimal",
            ElementKind::Decimal {
                minimum: None,
                maximum: None,
                store_as_text: true,
            },
        );
        assert_eq!(
            json_to_element_data(&text_decimal, &json!(42)).unwrap(),
            PolicyValue::Data(RegistryValueData::String("42".to_string()))
        );

        let choice = PolicyValue::Data(RegistryValueData::String("choice".to_string()));
        let enumeration = element(
            "Enum",
            ElementKind::Enum(vec![EnumItem {
                title: "Choice".to_string(),
                value: choice.clone(),
            }]),
        );
        assert_eq!(
            json_to_element_data(&enumeration, &json!("choice")).unwrap(),
            choice
        );
        assert!(json_to_element_data(&enumeration, &json!("other")).is_err());

        let multi = element("Multi", ElementKind::MultiText);
        assert_eq!(
            json_to_element_data(&multi, &json!(["one", "two"])).unwrap(),
            PolicyValue::Data(RegistryValueData::MultiString(vec![
                "one".to_string(),
                "two".to_string()
            ]))
        );
        assert!(json_to_element_data(&multi, &json!([1])).is_err());

        for expandable in [false, true] {
            let text = element("Text", ElementKind::Text { expandable });
            let expected = if expandable {
                RegistryValueData::ExpandString("value".to_string())
            } else {
                RegistryValueData::String("value".to_string())
            };
            assert_eq!(
                json_to_element_data(&text, &json!("value")).unwrap(),
                PolicyValue::Data(expected)
            );
            assert!(json_to_element_data(&text, &json!(1)).is_err());
        }

        let list = element("List", ElementKind::List);
        assert!(json_to_element_data(&list, &json!({})).is_err());
    }

    #[test]
    fn converts_registry_data_for_every_element_type() {
        let boolean = element(
            "Boolean",
            ElementKind::Boolean {
                true_value: PolicyValue::Data(RegistryValueData::DWord(1)),
                false_value: PolicyValue::Data(RegistryValueData::DWord(0)),
            },
        );
        assert_eq!(
            element_data_to_json(&boolean, &RegistryValueData::DWord(1)).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            element_data_to_json(&boolean, &RegistryValueData::DWord(0)).unwrap(),
            Value::Bool(false)
        );
        assert!(element_data_to_json(&boolean, &RegistryValueData::DWord(2)).is_err());

        let enumeration = element(
            "Enum",
            ElementKind::Enum(vec![EnumItem {
                title: "One".to_string(),
                value: PolicyValue::Data(RegistryValueData::DWord(1)),
            }]),
        );
        assert_eq!(
            element_data_to_json(&enumeration, &RegistryValueData::DWord(1)).unwrap(),
            json!(1)
        );
        assert!(element_data_to_json(&enumeration, &RegistryValueData::DWord(2)).is_err());

        let text_decimal = element(
            "TextDecimal",
            ElementKind::Decimal {
                minimum: None,
                maximum: None,
                store_as_text: true,
            },
        );
        assert_eq!(
            element_data_to_json(&text_decimal, &RegistryValueData::String("42".to_string()))
                .unwrap(),
            json!(42)
        );
        assert!(
            element_data_to_json(
                &text_decimal,
                &RegistryValueData::String("invalid".to_string())
            )
            .is_err()
        );

        for kind in [
            ElementKind::Decimal {
                minimum: None,
                maximum: None,
                store_as_text: false,
            },
            ElementKind::MultiText,
            ElementKind::Text { expandable: false },
            ElementKind::List,
        ] {
            assert_eq!(
                element_data_to_json(&element("Value", kind), &RegistryValueData::DWord(7))
                    .unwrap(),
                json!(7)
            );
        }
    }

    #[test]
    fn rejects_invalid_policy_objects_without_registry_access() {
        let mut value_policy = policy(PolicyClass::Both);
        assert!(write_policy(&value_policy, "currentUser", &json!("invalid")).is_err());
        assert!(write_policy(&value_policy, "currentUser", &json!(true)).is_err());
        assert!(write_policy(&value_policy, "currentUser", &json!({"enabled": "true"})).is_err());
        assert!(write_policy(&value_policy, "currentUser", &json!({"Unknown": true})).is_err());
        assert!(!state_matches(&value_policy, "currentUser", true).unwrap());
        assert!(!policy_values_are_absent(&value_policy, "currentUser").unwrap());

        value_policy.elements.push(PolicyElement {
            id: "MissingValueName".to_string(),
            key: None,
            value_name: None,
            kind: ElementKind::Text { expandable: false },
        });
        assert!(
            write_policy(
                &value_policy,
                "currentUser",
                &json!({"MissingValueName": "value"})
            )
            .is_err()
        );
    }
}
