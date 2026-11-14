// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib::settings::{DscSettings, DscSettingsScope};
use serde_json::json;

#[test]
fn jsonrpc_settings_resolve_each_leaf_and_preserve_policy() {
    let mut settings = DscSettings::new();
    settings.machine = Some(serde_json::from_value(json!({
        "resourceReuse": false, "resourceTimeouts": {"initializeTimeoutMS": 500, "requestTimeoutMS": 1000}
    })).unwrap());
    settings.workspace = Some(serde_json::from_value(json!({
        "resourceReuse": true, "resourceTimeouts": {"requestTimeoutMS": 25}
    })).unwrap());
    settings.environment = Some(dsc_lib::settings::sources::EnvironmentData {
        dsc_resource_reuse: Some(false), ..Default::default()
    });
    settings.policy = Some(serde_json::from_value(json!({
        "resourceReuse": true, "resourceTimeouts": {"shutdownTimeoutMS": 750, "requestTimeoutMS": 0}
    })).unwrap());
    let resolved = settings.resolved();
    assert!(resolved.resource_reuse.value);
    assert_eq!(resolved.resource_reuse.scope, DscSettingsScope::Policy);
    assert_eq!(resolved.resource_timeouts.initialize_timeout_ms.value.get(), 500);
    assert_eq!(resolved.resource_timeouts.initialize_timeout_ms.scope, DscSettingsScope::Machine);
    assert_eq!(resolved.resource_timeouts.request_timeout_ms.value, 0);
    assert_eq!(resolved.resource_timeouts.request_timeout_ms.scope, DscSettingsScope::Policy);
    assert_eq!(resolved.resource_timeouts.shutdown_timeout_ms.value.get(), 750);
}

#[test]
fn jsonrpc_settings_environment_overrides_preferences() {
    let mut settings = DscSettings::new();
    settings.workspace = Some(serde_json::from_value(json!({"resourceReuse": true})).unwrap());
    settings.environment = Some(dsc_lib::settings::sources::EnvironmentData {
        dsc_resource_reuse: Some(false), ..Default::default()
    });
    assert!(!settings.resolved().resource_reuse.value);
    assert_eq!(settings.resolved().resource_reuse.scope, DscSettingsScope::Environment);
}

#[test]
fn jsonrpc_settings_reject_zero_lifecycle_timeouts() {
    use dsc_lib::settings::sources::PreferenceFileData;
    for field in ["initializeTimeoutMS", "shutdownTimeoutMS"] {
        assert!(serde_json::from_value::<PreferenceFileData>(json!({"resourceTimeouts": {(field): 0}})).is_err());
    }
    assert!(serde_json::from_value::<PreferenceFileData>(json!({"resourceTimeouts": {"requestTimeoutMS": 0}})).is_ok());
}
