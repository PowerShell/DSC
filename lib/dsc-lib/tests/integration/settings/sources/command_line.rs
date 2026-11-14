// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

crate::macros::test_dsc_repo_schema! {
    define_statics: {
        dsc_lib::settings::sources::CommandLineData,
        properties: {
            TRACE_LEVEL_SCHEMA => "--trace-level",
            TRACE_FORMAT_SCHEMA => "--trace-format",
            IGNORE_SETTINGS_FILE_SCHEMA => "--ignore-settings-file"
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
                "--trace-level": "info"
            }),
            expected_valid: true,
        },
        with_invalid_trace_level_is_invalid => {
            input_json: json!({
                "--trace-level": "invalid"
            }),
            expected_valid: false,
        },
        with_valid_trace_format_is_valid => {
            input_json: json!({
                "--trace-format": "json"
            }),
            expected_valid: true,
        },
        with_invalid_trace_format_is_invalid => {
            input_json: json!({
                "--trace-format": "invalid"
            }),
            expected_valid: false,
        },
        with_valid_ignore_settings_file_is_valid => {
            input_json: json!({
                "--ignore-settings-file": true
            }),
            expected_valid: true,
        },
        with_invalid_ignore_settings_file_is_invalid => {
            input_json: json!({
                "--ignore-settings-file": "invalid"
            }),
            expected_valid: false,
        },
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::settings::{fields::TracingLevelField, sources::CommandLineData};
    use test_case::test_case;
    use serde_json::{json, Value};
    use crate::settings::builders::*;

    #[test_case(
        &CommandLineDataBuilder::new().build(),
        json!({});
        "without any defined fields serializes as empty object"
    )]
    #[test_case(
        &CommandLineDataBuilder::new().with_trace_level(TracingLevelField::Debug).build(),
        json!({
            "--trace-level": "debug"
        });
        "with defined fields only serializes those properties"
    )]
    fn serializing(data: &CommandLineData, expected: Value) {
        let serialized = serde_json::to_value(data)
            .expect("serialization should never fail");
        pretty_assertions::assert_eq!(serialized, expected);
    }
}
