// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

crate::macros::test_dsc_repo_schema! {
    define_statics: {
        dsc_lib::settings::fields::PluginsFileData,
        properties: {
            ENABLED_SCHEMA => "enabled",
        }
    },
    test_meta_schema: {},
    test_docs: {
        title => "title",
        description => "description",
        markdown_description => "markdownDescription",
        enabled_title => (ENABLED_SCHEMA, "title"),
        enabled_description => (ENABLED_SCHEMA, "description"),
        enabled_markdown_description => (ENABLED_SCHEMA, "markdownDescription"),
    },
    test_validation: {
        empty_object_is_valid => {
            input_json: json!({}),
            expected_valid: true,
        },
        with_enabled_as_true_is_valid => {
            input_json: json!({"enabled": true}),
            expected_valid: true,
        },
        with_enabled_as_false_is_valid => {
            input_json: json!({"enabled": false}),
            expected_valid: true,
        },
        with_enabled_as_string_is_invalid => {
            input_json: json!({"enabled": "true"}),
            expected_valid: false,
        },
        with_enabled_as_integer_is_invalid => {
            input_json: json!({"enabled": 1}),
            expected_valid: false,
        },
        with_enabled_as_null_is_valid => {
            input_json: json!({"enabled": null}),
            expected_valid: true,
        },
        with_an_unexpected_field_is_invalid => {
            input_json: json!({"unexpectedField": true}),
            expected_valid: false,
        },
        array_value_is_invalid => {
            input_json: json!([]),
            expected_valid: false,
        },
        string_value_is_invalid => {
            input_json: json!("invalid"),
            expected_valid: false,
        },
        null_value_is_invalid => {
            input_json: json!(null),
            expected_valid: false,
        },
        boolean_value_is_invalid => {
            input_json: json!(true),
            expected_valid: false,
        },
        integer_value_is_invalid => {
            input_json: json!(123),
            expected_valid: false,
        },
    },
}

#[cfg(test)]
mod serde {
    use dsc_lib::settings::fields::PluginsFileData;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(
        PluginsFileData::default() => json!({});
        "default serializes as empty object"
    )]
    #[test_case(
        PluginsFileData {
            enabled: Some(true),
        } => json!({
            "enabled": true
        }); "enabled as true serializes as boolean"
    )]
    #[test_case(
        PluginsFileData {
            enabled: Some(false),
        } => json!({
            "enabled": false
        }); "enabled as false serializes as boolean"
    )]
    fn serializing(data: PluginsFileData) -> Value {
        serde_json::to_value(data).expect("serialization should never fail")
    }

    #[test_case(&json!("string"); "string value")]
    #[test_case(&json!(123); "integer value")]
    #[test_case(&json!(1.2); "number value")]
    #[test_case(&json!(true); "boolean value")]
    #[test_case(&json!(null); "null value")]
    #[test_case(&json!([{"enabled": true}]); "array value")]
    #[test_case(&json!({"enabled": "invalid"}); "object with enabled as string value")]
    #[test_case(&json!({"enabled": 123}); "object with enabled as number value")]
    #[test_case(&json!({"enabled": {"nested": "object"}}); "object with enabled as object value")]
    #[test_case(&json!({"enabled": [false]}); "object with enabled as array value")]
    fn deserializing_invalid(input_value: &Value) {
        serde_json::from_value::<PluginsFileData>(input_value.clone())
            .expect_err(&format!("json value '{input_value}' should be invalid"));
    }

    #[test_case(
        &json!({}) => PluginsFileData::default();
        "empty object deserializes as default"
    )]
    #[test_case(
        &json!({
            "enabled": null
        }) => PluginsFileData::default();
        "enabled as null deserializes as undefined"
    )]
    #[test_case(
        &json!({
            "enabled": true
        }) => PluginsFileData {
            enabled: Some(true),
        };
        "enabled as true deserializes as boolean"
    )]
    #[test_case(
        &json!({
            "enabled": false
        }) => PluginsFileData {
            enabled: Some(false),
        };
        "enabled as false deserializes as boolean"
    )]
    fn deserializing_valid(input_value: &Value) -> PluginsFileData {
        serde_json::from_value::<PluginsFileData>(input_value.clone())
            .expect(&format!(
                "expected input to serialize but failed for input: {}",
                serde_json::to_string_pretty(input_value).unwrap(),
            ))
    }
}

#[cfg(test)]
mod defaults {
    use dsc_lib::settings::{
        DscSettingsScope,
        fields::{CODE_DEFAULT_PLUGINS, PluginsCodeDefaults, PluginsResolvedSettings},
    };

    #[test] fn code_default_enables_plugins() {
        pretty_assertions::assert_eq!(CODE_DEFAULT_PLUGINS.enabled, true);
        pretty_assertions::assert_eq!(PluginsCodeDefaults::default(), CODE_DEFAULT_PLUGINS);
    }

    #[test] fn resolved_default_is_enabled_from_default_scope() {
        let resolved = PluginsResolvedSettings::default();
        pretty_assertions::assert_eq!(resolved.enabled.value, true);
        pretty_assertions::assert_eq!(resolved.enabled.scope, DscSettingsScope::Default);
    }
}
