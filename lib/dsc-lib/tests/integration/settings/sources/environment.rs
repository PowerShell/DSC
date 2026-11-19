// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    use dsc_lib::settings::{DscSettingsError, sources::EnvironmentData};
    use test_case::test_case;

    #[test_case("true" => matches Ok(v) if v == true; "lower case true parses as true")]
    #[test_case("TRUE" => matches Ok(v) if v == true; "upper case true parses as true")]
    #[test_case("tRuE" => matches Ok(v) if v == true; "mixed case true parses as true")]
    #[test_case("false" => matches Ok(v) if v == false; "lower case false parses as false")]
    #[test_case("FALSE" => matches Ok(v) if v == false; "upper case false parses as false")]
    #[test_case("fAlSe" => matches Ok(v) if v == false; "mixed case false parses as false")]
    #[test_case("1" => matches Ok(v) if v == true; "numeric 1 parses as true")]
    #[test_case("0" => matches Ok(v) if v == false; "numeric 0 parses as false")]
    #[test_case("invalid" => matches Err(_); "invalid value returns parse error")]
    fn parse_boolean_env_var(input: &str) -> Result<bool, DscSettingsError> {
        EnvironmentData::parse_boolean_env_var(input)
    }
}

crate::macros::test_dsc_repo_schema!{
    define_statics: {
        dsc_lib::settings::sources::EnvironmentData,
        properties: {
            TRACE_LEVEL_SCHEMA => "DSC_TRACE_LEVEL",
            TRACE_FORMAT_SCHEMA => "DSC_TRACE_FORMAT",
            RESOURCE_PATH_SCHEMA => "DSC_RESOURCE_PATH",
            RESTRICTED_PATH_SCHEMA => "DSC_RESTRICTED_PATH",
            IGNORE_SETTINGS_FILE_SCHEMA => "DSC_IGNORE_SETTINGS_FILE",
        }
    },
    test_meta_schema: {},
    test_docs: {
        title => "title",
        description => "description",
        markdown_description => "markdownDescription",
        trace_level_title => (TRACE_LEVEL_SCHEMA, "title"),
        trace_level_description => (TRACE_LEVEL_SCHEMA, "description"),
        trace_level_markdown_description => (TRACE_LEVEL_SCHEMA, "markdownDescription"),
        trace_format_title => (TRACE_FORMAT_SCHEMA, "title"),
        trace_format_description => (TRACE_FORMAT_SCHEMA, "description"),
        trace_format_markdown_description => (TRACE_FORMAT_SCHEMA, "markdownDescription"),
        resource_path_title => (RESOURCE_PATH_SCHEMA, "title"),
        resource_path_description => (RESOURCE_PATH_SCHEMA, "description"),
        resource_path_markdown_description => (RESOURCE_PATH_SCHEMA, "markdownDescription"),
        restricted_path_title => (RESTRICTED_PATH_SCHEMA, "title"),
        restricted_path_description => (RESTRICTED_PATH_SCHEMA, "description"),
        restricted_path_markdown_description => (RESTRICTED_PATH_SCHEMA, "markdownDescription"),
        ignore_settings_file_title => (IGNORE_SETTINGS_FILE_SCHEMA, "title"),
        ignore_settings_file_description => (IGNORE_SETTINGS_FILE_SCHEMA, "description"),
        ignore_settings_file_markdown_description => (IGNORE_SETTINGS_FILE_SCHEMA, "markdownDescription"),
    },
    test_validation: {
        empty_object_is_valid => {
            input_json: json!({}),
            expected_valid: true,
        },
        with_valid_trace_level_is_valid => {
            input_json: json!({
                "DSC_TRACE_LEVEL": "info"
            }),
            expected_valid: true,
        },
        with_invalid_trace_level_is_invalid => {
            input_json: json!({
                "DSC_TRACE_LEVEL": "invalid_value"
            }),
            expected_valid: false,
        },
        with_valid_trace_format_is_valid => {
            input_json: json!({
                "DSC_TRACE_FORMAT": "json"
            }),
            expected_valid: true,
        },
        with_invalid_trace_format_is_invalid => {
            input_json: json!({
                "DSC_TRACE_FORMAT": "invalid_value"
            }),
            expected_valid: false,
        },
        with_resource_path_as_string_array_value_is_valid => {
            input_json: json!({
                "DSC_RESOURCE_PATH": ["/valid/path"]
            }),
            expected_valid: true,
        },
        with_resource_path_as_non_array_is_invalid => {
            input_json: json!({
                "DSC_RESOURCE_PATH": "invalid_path"
            }),
            expected_valid: false,
        },
        with_restricted_path_as_string_array_value_is_valid => {
            input_json: json!({
                "DSC_RESTRICTED_PATH": ["/valid/path"]
            }),
            expected_valid: true,
        },
        with_restricted_path_as_non_array_is_invalid => {
            input_json: json!({
                "DSC_RESTRICTED_PATH": "invalid_path"
            }),
            expected_valid: false,
        },
        with_ignore_settings_file_as_boolean_is_valid => {
            input_json: json!({
                "DSC_IGNORE_SETTINGS_FILE": true
            }),
            expected_valid: true,
        },
        with_ignore_settings_file_as_non_boolean_is_invalid => {
            input_json: json!({
                "DSC_IGNORE_SETTINGS_FILE": "invalid_value"
            }),
            expected_valid: false,
        },
    },
}

#[cfg(test)]
mod serde {
    use dsc_lib::settings::sources::EnvironmentData;
    use dsc_lib::settings::fields::TracingLevelField;
    use serde_json::{json, Value};
    use test_case::test_case;

    use crate::settings::builders::EnvironmentDataBuilder;

    #[test_case(
        &EnvironmentData::default(),
        &json!({});
        "Without any data serializes as empty object"
    )]
    #[test_case(
        &EnvironmentDataBuilder::new().with_trace_level(TracingLevelField::Info).build(),
        &json!({
            "DSC_TRACE_LEVEL": "info"
        });
        "With defined properties only serializes those properties"
    )]
    fn serializing(input: &EnvironmentData, expected: &Value) {
        let serialized = serde_json::to_value(input).unwrap();
        assert_eq!(&serialized, expected);
    }

    // No deserialization tests because we don't implement deserialize since the
    // data is read from environment variables.
}
