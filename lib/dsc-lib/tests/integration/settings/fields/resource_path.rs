// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

crate::macros::test_dsc_repo_schema! {
    define_statics: {
        dsc_lib::settings::fields::ResourcePathFileData,
        properties: {
            DIRECTORIES_SCHEMA => "directories",
            APPEND_ENV_PATH_SCHEMA => "appendEnvPath",
            RESTRICTED_SCHEMA => "restricted",
        }
    },
    test_meta_schema: {},
    test_docs: {
        title => "title",
        description => "description",
        markdown_description => "markdownDescription",
        directories_title => (DIRECTORIES_SCHEMA, "title"),
        directories_description => (DIRECTORIES_SCHEMA, "description"),
        directories_markdown_description => (DIRECTORIES_SCHEMA, "markdownDescription"),
        append_env_path_title => (APPEND_ENV_PATH_SCHEMA, "title"),
        append_env_path_description => (APPEND_ENV_PATH_SCHEMA, "description"),
        append_env_path_markdown_description => (APPEND_ENV_PATH_SCHEMA, "markdownDescription"),
        restricted_title => (RESTRICTED_SCHEMA, "title"),
        restricted_description => (RESTRICTED_SCHEMA, "description"),
        restricted_markdown_description => (RESTRICTED_SCHEMA, "markdownDescription")
    },
    test_validation: {
        empty_object_is_valid => {
            input_json: json!({}),
            expected_valid: true,
        },
        with_just_directories_is_valid => {
            input_json: json!({"directories": ["/first", "/second"]}),
            expected_valid: true,
        },
        with_just_append_env_path_is_valid => {
            input_json: json!({"appendEnvPath": true}),
            expected_valid: true,
        },
        with_just_restricted_is_valid => {
            input_json: json!({"restricted": true}),
            expected_valid: true,
        },
        with_directories_and_append_env_path_fields_is_valid => {
            input_json: json!({
                "directories": ["/first", "/second"],
                "appendEnvPath": true
            }),
            expected_valid: true,
        },
        with_directories_and_restricted_fields_is_valid => {
            input_json: json!({
                "directories": ["/first", "/second"],
                "restricted": true
            }),
            expected_valid: true,
        },
        with_append_env_path_as_false_and_restricted_as_true_is_valid => {
            input_json: json!({
                "appendEnvPath": false,
                "restricted": true
            }),
            expected_valid: true,
        },
        with_append_env_path_as_true_and_restricted_as_false_is_valid => {
            input_json: json!({
                "appendEnvPath": true,
                "restricted": false
            }),
            expected_valid: true,
        },
        with_append_env_path_as_true_and_restricted_as_true_is_invalid => {
            input_json: json!({
                "appendEnvPath": true,
                "restricted": true
            }),
            expected_valid: false,
        },
        with_an_unexpected_field_is_invalid => {
            input_json: json!({
                "unexpectedField": true
            }),
            expected_valid: false,
        }
    },
}

#[cfg(test)]
mod serde {
    use std::path::PathBuf;

    use dsc_lib::settings::fields::ResourcePathFileData;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(
        ResourcePathFileData::default() => json!({});
        "default serializes as empty object"
    )]
    #[test_case(
        ResourcePathFileData {
            directories: Some(vec![]),
            ..Default::default()
        } => json!({
            "directories": []
        }); "directories as an empty vec serializes as an empty array"
    )]
    #[test_case(
        ResourcePathFileData {
            directories: Some(vec![PathBuf::from("/foo"), PathBuf::from("/bar")]),
            ..Default::default()
        } => json!({
            "directories": ["/foo", "/bar"]
        }); "directories as vec of pathbuf serializes as array of strings"
    )]
    #[test_case(
        ResourcePathFileData {
            append_env_path: Some(true),
            ..Default::default()
        } => json!({
            "appendEnvPath": true
        }); "append_env_path serializes as boolean"
    )]
    #[test_case(
        ResourcePathFileData {
            restricted: Some(true),
            ..Default::default()
        } => json!({
            "restricted": true
        }); "restricted serializes as boolean"
    )]
    #[test_case(
        ResourcePathFileData {
            directories: Some(vec![PathBuf::from("/foo"), PathBuf::from("/bar")]),
            append_env_path: Some(false),
            restricted: Some(false),
        } => json!({
            "directories": ["/foo", "/bar"],
            "appendEnvPath": false,
            "restricted": false
        }); "fully defined data serializes correctly"
    )]
    fn serializing(data: ResourcePathFileData) -> Value {
        serde_json::to_value(data).expect("serialization should never fail")
    }

    #[test_case(&json!("string"); "string value")]
    #[test_case(&json!(123); "integer value")]
    #[test_case(&json!(1.2); "number value")]
    #[test_case(&json!(true); "boolean value")]
    #[test_case(&json!(null); "null value")]
    #[test_case(&json!([{"restricted": true}]); "array value")]
    #[test_case(&json!({"directories": "invalid"}); "object with directories as string value")]
    #[test_case(&json!({"directories": 123}); "object with directories as number value")]
    #[test_case(&json!({"directories": true}); "object with directories as boolean value")]
    #[test_case(&json!({"directories": {"nested": "object"}}); "object with directories as object value")]
    #[test_case(&json!({"directories": ["valid", 123]}); "object with directories as array containing a non-string")]
    #[test_case(&json!({"appendEnvPath": "invalid"}); "object with appendEnvPath as string value")]
    #[test_case(&json!({"appendEnvPath": 123}); "object with appendEnvPath as number value")]
    #[test_case(&json!({"appendEnvPath": {"nested": "object"}}); "object with appendEnvPath as object value")]
    #[test_case(&json!({"appendEnvPath": [false]}); "object with appendEnvPath as array value")]
    #[test_case(&json!({"restricted": "invalid"}); "object with restricted as string value")]
    #[test_case(&json!({"restricted": 123}); "object with restricted as number value")]
    #[test_case(&json!({"restricted": {"nested": "object"}}); "object with restricted as object value")]
    #[test_case(&json!({"restricted": [false]}); "object with restricted as array value")]
    fn deserializing_invalid(input_value: &Value) {
        serde_json::from_value::<ResourcePathFileData>(input_value.clone())
            .expect_err(&format!("json value '{input_value}' should be invalid"));
    }

    #[test_case(
        &json!({}) => ResourcePathFileData::default();
        "empty object deserializes as default"
    )]
    #[test_case(
        &json!({
            "directories": []
        }) => ResourcePathFileData {
            directories: Some(vec![]),
            ..Default::default()
        };
        "empty directories array deserializes as empty vec"
    )]
    #[test_case(
        &json!({
            "directories": ["/foo", "/bar"]
        }) => ResourcePathFileData {
            directories: Some(vec![PathBuf::from("/foo"), PathBuf::from("/bar")]),
            ..Default::default()
        };
        "non-empty directories array deserializes as vec of pathbufs"
    )]
    #[test_case(
        &json!({
            "appendEnvPath": true
        }) => ResourcePathFileData {
            append_env_path: Some(true),
            ..Default::default()
        };
        "append_env_path deserializes as boolean"
    )]
    #[test_case(
        &json!({
            "restricted": true
        }) => ResourcePathFileData {
            restricted: Some(true),
            ..Default::default()
        };
        "restricted deserializes as boolean"
    )]
    #[test_case(
        &json!({
            "directories": ["/foo", "/bar"],
            "appendEnvPath": false,
            "restricted": false
        }) => ResourcePathFileData {
            directories: Some(vec![PathBuf::from("/foo"), PathBuf::from("/bar")]),
            append_env_path: Some(false),
            restricted: Some(false),
        }; "fully defined data deserializes correctly"
    )]
    fn deserializing_valid(input_value: &Value) -> ResourcePathFileData {
        serde_json::from_value::<ResourcePathFileData>(input_value.clone())
            .expect(&format!(
                "expected input to serialize but failed for input: {}",
                serde_json::to_string_pretty(input_value).unwrap(),
            ))
    }
}
