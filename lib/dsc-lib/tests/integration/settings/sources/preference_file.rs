// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    //! These tests are handled in the unit tests in
    //! `src/tests/settings/sources/preference_file.rs`
    //!
    //! We use some stubs to simulate the behavior of the preference file, but
    //! unfortunately can't do so in integration tests because we can't access
    //! the code built for `#[cfg(test)]`.
}

crate::macros::test_dsc_repo_schema!{
    define_statics: {
        dsc_lib::settings::sources::PreferenceFileData,
        properties: {
            TRACING_PROPERTY_SCHEMA => "tracing",
            RESOURCE_PATH_PROPERTY_SCHEMA => "resourcePath",
        }
    },
    test_meta_schema: {},
    test_docs: {
        title => "title",
        description => "description",
        markdown_description => "markdownDescription",
        tracing_title => (TRACING_PROPERTY_SCHEMA, "title"),
        tracing_description => (TRACING_PROPERTY_SCHEMA, "description"),
        tracing_markdown_description => (TRACING_PROPERTY_SCHEMA, "markdownDescription"),
        resource_path_title => (RESOURCE_PATH_PROPERTY_SCHEMA, "title"),
        resource_path_description => (RESOURCE_PATH_PROPERTY_SCHEMA, "description"),
        resource_path_markdown_description => (RESOURCE_PATH_PROPERTY_SCHEMA, "markdownDescription"),
    },
    test_validation: {
        empty_object_is_valid => {
            input_json: json!({}),
            expected_valid: true,
        },
        tracing_field_as_empty_object_is_valid => {
            input_json: json!({"tracing": {}}),
            expected_valid: true,
        },
        resource_path_field_as_empty_object_is_valid => {
            input_json: json!({"resourcePath": {}}),
            expected_valid: true,
        },
        resource_path_field_with_valid_data_is_valid => {
            input_json: json!({"resourcePath": {"appendEnvPath": true}}),
            expected_valid: true,
        },
        resource_path_field_with_invalid_data_is_invalid => {
            input_json: json!({"resourcePath": {"appendEnvPath": "invalid"}}),
            expected_valid: false,
        },
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::settings::{fields::ResourcePathFileData, sources::PreferenceFileData};
    use std::path::PathBuf;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(&PreferenceFileData::default(), json!({}); "without any defined fields is empty object")]
    #[test_case(
        &PreferenceFileData {
            tracing: None,
            resource_path: Some(ResourcePathFileData{
                directories: Some(vec![
                    PathBuf::from("/dsc/resources"),
                    PathBuf::from("/dsc/extensions")
                ]),
                ..Default::default()
            })
        },
        json!({
            "resourcePath": {
                "directories": ["/dsc/resources", "/dsc/extensions"]
            }
        });
        "with defined fields only includes those defined fields"
    )]
    fn serializing(data: &PreferenceFileData, expected: Value) {
        let serialized = serde_json::to_value(data)
            .expect("serialization should never fail");
        pretty_assertions::assert_eq!(serialized, expected);
    }

    #[test_case(json!({"tracing": {"level": "invalid"}}); "tracing with invalid data is invalid")]
    #[test_case(json!({"resourcePath": {"appendEnvPath": "invalid"}}); "resource_path with invalid data is invalid")]
    fn derserializing_invalid(input_value: Value) {
        let result = serde_json::from_value::<PreferenceFileData>(input_value.clone());
        assert!(result.is_err(), "expected deserialization to fail for input: {:?}", input_value);
    }
    #[test_case(json!({}); "empty object is valid")]
    #[test_case(json!({"resourcePath": {"directories": ["/dsc/resources"]}}); "resource_path with valid data is valid")]
    #[test_case(json!({"tracing": {"level": "info"}}); "tracing with valid data is valid")]
    fn deserializing_valid(input_value: Value) {
        let result = serde_json::from_value::<PreferenceFileData>(input_value.clone());
        assert!(
            result.is_ok(),
            "expected deserialization to succeed for input: {:?} but got error: {:#?}",
            input_value,
            result.unwrap_err()
        );
    }
}

#[cfg(test)]
mod traits {}
