// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod file_data {
    crate::macros::test_dsc_repo_schema! {
        define_statics: {
            dsc_lib::settings::fields::TracingFileData,
            properties: {
                LEVEL_SCHEMA => "level",
                FORMAT_SCHEMA => "format",
            }
        },
        test_meta_schema: {},
        test_docs: {
            title => "title",
            description => "description",
            markdown_description => "markdownDescription",
            level_title => (LEVEL_SCHEMA, "title"),
            level_description => (LEVEL_SCHEMA, "description"),
            level_markdown_description => (LEVEL_SCHEMA, "markdownDescription"),
            format_title => (FORMAT_SCHEMA, "title"),
            format_description => (FORMAT_SCHEMA, "description"),
            format_markdown_description => (FORMAT_SCHEMA, "markdownDescription"),
        },
        test_validation: {
            empty_object_is_valid => {
                input_json: json!({}),
                expected_valid: true,
            },
            with_just_level_is_valid => {
                input_json: json!({"level": "info"}),
                expected_valid: true,
            },
            with_just_format_is_valid => {
                input_json: json!({"format": "json"}),
                expected_valid: true,
            },
            with_level_and_format_is_valid => {
                input_json: json!({"level": "info", "format": "json"}),
                expected_valid: true,
            },
            with_an_unknown_level_is_invalid => {
                input_json: json!({"level": "unknown"}),
                expected_valid: false,
            },
            with_an_unknown_format_is_invalid => {
                input_json: json!({"format": "unknown"}),
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
            number_value_is_invalid => {
                input_json: json!(1.2),
                expected_valid: false,
            },
        },
    }

    mod serde {
        use dsc_lib::settings::fields::{TracingFileData, TracingLevelField, TracingFormatField};
        use serde_json::{json, Value};
        use test_case::test_case;

        #[test_case(
            TracingFileData::default() => json!({});
            "default serializes as empty object"
        )]
        #[test_case(
            TracingFileData {
                level: Some(TracingLevelField::Info),
                ..Default::default()
            } => json!({
                "level": "info"
            }); "level serializes as a string"
        )]
        #[test_case(
            TracingFileData {
                format: Some(TracingFormatField::Json),
                ..Default::default()
            } => json!({
                "format": "json"
            }); "format serializes as a string"
        )]
        #[test_case(
            TracingFileData {
                level: Some(TracingLevelField::Info),
                format: Some(TracingFormatField::Json),
            } => json!({
                "level": "info",
                "format": "json"
            }); "fully defined data serializes correctly"
        )]
        fn serializing(data: TracingFileData) -> Value {
            serde_json::to_value(data).expect("serialization should never fail")
        }

        #[test_case(&json!("string"); "string value")]
        #[test_case(&json!(123); "integer value")]
        #[test_case(&json!(1.2); "number value")]
        #[test_case(&json!(true); "boolean value")]
        #[test_case(&json!(null); "null value")]
        #[test_case(&json!([{"level": "info"}]); "array value")]
        #[test_case(&json!({"level": "invalid"}); "object with level as invalid string value")]
        #[test_case(&json!({"level": 123}); "object with level as number value")]
        #[test_case(&json!({"level": true}); "object with level as boolean value")]
        #[test_case(&json!({"level": {"nested": "object"}}); "object with level as object value")]
        #[test_case(&json!({"level": ["info"]}); "object with level as array value")]
        #[test_case(&json!({"format": "invalid"}); "object with format as invalid string value")]
        #[test_case(&json!({"format": 123}); "object with format as number value")]
        #[test_case(&json!({"format": true}); "object with format as boolean value")]
        #[test_case(&json!({"format": {"nested": "object"}}); "object with format as object value")]
        #[test_case(&json!({"format": ["json"]}); "object with format as array value")]
        fn deserializing_invalid(input_value: &Value) {
            serde_json::from_value::<TracingFileData>(input_value.clone())
                .expect_err(&format!("json value '{input_value}' should be invalid"));
        }

        #[test_case(
            &json!({}) => TracingFileData::default();
            "empty object deserializes as default"
        )]
        #[test_case(
            &json!({
                "level": "info"
            }) => TracingFileData {
                level: Some(TracingLevelField::Info),
                ..Default::default()
            };
            "level deserializes as string enum field"
        )]
        #[test_case(
            &json!({
                "format": "json"
            }) => TracingFileData {
                format: Some(TracingFormatField::Json),
                ..Default::default()
            };
            "format deserializes as string enum field"
        )]
        #[test_case(
            &json!({
                "level": "info",
                "format": "json"
            }) => TracingFileData {
                level: Some(TracingLevelField::Info),
                format: Some(TracingFormatField::Json),
            }; "fully defined data deserializes correctly"
        )]
        fn deserializing_valid(input_value: &Value) -> TracingFileData {
            serde_json::from_value::<TracingFileData>(input_value.clone())
                .expect(&format!(
                    "expected input to serialize but failed for input: {}",
                    serde_json::to_string_pretty(input_value).unwrap(),
                ))
        }
    }
}

mod level_field {
    mod methods {
        mod parse {
            use dsc_lib::settings::DscSettingsError;
            use dsc_lib::settings::fields::TracingLevelField;
            use test_case::test_case;

            #[test_case("error" => matches TracingLevelField::Error ; "lower case error")]
            #[test_case("ERROR" => matches TracingLevelField::Error ; "upper case error")]
            #[test_case("eRrOr" => matches TracingLevelField::Error ; "mixed case error")]
            #[test_case("warn" => matches TracingLevelField::Warn ; "lower case warn")]
            #[test_case("WARN" => matches TracingLevelField::Warn ; "upper case warn")]
            #[test_case("wArN" => matches TracingLevelField::Warn ; "mixed case warn")]
            #[test_case("info" => matches TracingLevelField::Info ; "lower case info")]
            #[test_case("INFO" => matches TracingLevelField::Info ; "upper case info")]
            #[test_case("iNfO" => matches TracingLevelField::Info ; "mixed case info")]
            #[test_case("debug" => matches TracingLevelField::Debug ; "lower case debug")]
            #[test_case("DEBUG" => matches TracingLevelField::Debug ; "upper case debug")]
            #[test_case("dEbUg" => matches TracingLevelField::Debug ; "mixed case debug")]
            #[test_case("trace" => matches TracingLevelField::Trace ; "lower case trace")]
            #[test_case("TRACE" => matches TracingLevelField::Trace ; "upper case trace")]
            #[test_case("tRaCe" => matches TracingLevelField::Trace ; "mixed case trace")]
            fn valid(input: &str) -> TracingLevelField {
                TracingLevelField::parse(input)
                    .expect(&format!("expected input '{}' to be valid", input))
            }

            #[test_case(""; "empty string")]
            #[test_case(" "; "space only string")]
            #[test_case("invalid"; "unrecognized variant")]
            fn invalid(input: &str) {
                let err = TracingLevelField::parse(input)
                    .expect_err(&format!("expected input '{input}' to be invalid"));

                match err {
                    DscSettingsError::InvalidTracingLevel { text } => {
                        pretty_assertions::assert_eq!(text, input);
                    }
                    _ => panic!("expected InvalidTracingLevel error but got: {err:?}"),
                }
            }
        }
    }

    crate::macros::test_dsc_repo_schema! {
        define_statics: {
            dsc_lib::settings::fields::TracingLevelField
        },
        test_meta_schema: {},
        test_docs: {
            title => "title",
            description => "description",
            markdown_description => "markdownDescription",
            markdown_enum_descriptions => "markdownEnumDescriptions",
        },
        test_validation: {
            error_string_value_is_valid => {
                input_json: json!("error"),
                expected_valid: true,
            },
            warn_string_value_is_valid => {
                input_json: json!("warn"),
                expected_valid: true,
            },
            info_string_value_is_valid => {
                input_json: json!("info"),
                expected_valid: true,
            },
            debug_string_value_is_valid => {
                input_json: json!("debug"),
                expected_valid: true,
            },
            trace_string_value_is_valid => {
                input_json: json!("trace"),
                expected_valid: true,
            },
            unknown_string_value_is_invalid => {
                input_json: json!("unknown"),
                expected_valid: false,
            },
            boolean_value_is_invalid => {
                input_json: json!(true),
                expected_valid: false,
            },
            null_value_is_invalid => {
                input_json: json!(null),
                expected_valid: false,
            },
            empty_string_value_is_invalid => {
                input_json: json!(""),
                expected_valid: false,
            },
            integer_value_is_invalid => {
                input_json: json!(42),
                expected_valid: false,
            },
            number_value_is_invalid => {
                input_json: json!(3.14),
                expected_valid: false,
            }
        }
    }

    mod serde {
        use dsc_lib::settings::fields::TracingLevelField;
        use serde_json::{json, Value};
        use test_case::test_case;

        #[test_case(&TracingLevelField::Error => json!("error"); "error variant")]
        #[test_case(&TracingLevelField::Warn => json!("warn"); "warn variant")]
        #[test_case(&TracingLevelField::Info => json!("info"); "info variant")]
        #[test_case(&TracingLevelField::Debug => json!("debug"); "debug variant")]
        #[test_case(&TracingLevelField::Trace => json!("trace"); "trace variant")]
        fn serializing(input: &TracingLevelField) -> serde_json::Value {
            serde_json::to_value(input)
                .expect("serialization should never fail")
        }

        #[test_case(&json!("invalid"); "unrecognized variant string value")]
        #[test_case(&json!(123); "integer value")]
        #[test_case(&json!(true); "boolean value")]
        #[test_case(&json!(null); "null value")]
        #[test_case(&json!(["info"]); "array value")]
        #[test_case(&json!({"level": "info"}); "object value")]
        fn deserializing_invalid(input_value: &Value) {
            serde_json::from_value::<TracingLevelField>(input_value.clone())
                .expect_err(&format!("json value '{input_value}' should be invalid"));
        }

        #[test_case(&json!("error") => TracingLevelField::Error; "error variant")]
        #[test_case(&json!("warn") => TracingLevelField::Warn; "warn variant")]
        #[test_case(&json!("info") => TracingLevelField::Info; "info variant")]
        #[test_case(&json!("debug") => TracingLevelField::Debug; "debug variant")]
        #[test_case(&json!("trace") => TracingLevelField::Trace; "trace variant")]
        fn deserializing_valid(input_value: &Value) -> TracingLevelField {
            serde_json::from_value::<TracingLevelField>(input_value.clone())
                .expect(&format!(
                    "expected input to serialize but failed for input: {}",
                    serde_json::to_string_pretty(input_value).unwrap(),
                ))
        }
    }
    mod traits {
        mod display {
            use dsc_lib::settings::fields::TracingLevelField;
            use test_case::test_case;

            #[test_case("error"; "error variant")]
            #[test_case("warn"; "warn variant")]
            #[test_case("info"; "info variant")]
            #[test_case("debug"; "debug variant")]
            #[test_case("trace"; "trace variant")]
            fn format(variant: &str) {
                pretty_assertions::assert_eq!(
                    format!("level: {}", TracingLevelField::parse(variant).unwrap()),
                    format!("level: {variant}"),
                )
            }

            #[test_case("error"; "error variant")]
            #[test_case("warn"; "warn variant")]
            #[test_case("info"; "info variant")]
            #[test_case("debug"; "debug variant")]
            #[test_case("trace"; "trace variant")]
            fn to_string(variant: &str) {
                pretty_assertions::assert_eq!(
                    TracingLevelField::parse(variant).unwrap().to_string(),
                    variant.to_string(),
                );
            }
        }
        mod from_str {
            use dsc_lib::settings::{DscSettingsError, fields::TracingLevelField};
            // use std::str::FromStr;
            use test_case::test_case;

            #[test_case("error" => matches Ok(_); "error variant")]
            #[test_case("warn" => matches Ok(_); "warn variant")]
            #[test_case("info" => matches Ok(_); "info variant")]
            #[test_case("debug" => matches Ok(_); "debug variant")]
            #[test_case("trace" => matches Ok(_); "trace variant")]
            #[test_case("invalid" => matches Err(_); "invalid variant string")]
            #[test_case("" => matches Err(_); "empty string")]
            #[test_case(" " => matches Err(_); "space only string")]
            fn parse(input: &str) -> Result<TracingLevelField, DscSettingsError> {
                input.parse()
            }
        }
        mod try_from {
            use dsc_lib::settings::{DscSettingsError, fields::TracingLevelField};
            use std::convert::TryFrom;
            use test_case::test_case;

            #[test_case("error" => matches Ok(_); "error variant")]
            #[test_case("warn" => matches Ok(_); "warn variant")]
            #[test_case("info" => matches Ok(_); "info variant")]
            #[test_case("debug" => matches Ok(_); "debug variant")]
            #[test_case("trace" => matches Ok(_); "trace variant")]
            #[test_case("invalid" => matches Err(_); "invalid variant string")]
            #[test_case("" => matches Err(_); "empty string")]
            #[test_case(" " => matches Err(_); "space only string")]
            fn string(input: &str) -> Result<TracingLevelField, DscSettingsError> {
                TracingLevelField::try_from(input.to_string())
            }
        }
        mod from {
            use dsc_lib::settings::fields::TracingLevelField;
            use test_case::test_case;

            #[test_case(tracing::Level::ERROR => TracingLevelField::Error; "error variant")]
            #[test_case(tracing::Level::WARN => TracingLevelField::Warn; "warn variant")]
            #[test_case(tracing::Level::INFO => TracingLevelField::Info; "info variant")]
            #[test_case(tracing::Level::DEBUG => TracingLevelField::Debug; "debug variant")]
            #[test_case(tracing::Level::TRACE => TracingLevelField::Trace; "trace variant")]
            fn tokio_tracing_level(input: tracing::Level) -> TracingLevelField {
                TracingLevelField::from(input)
            }
        }
        mod into {
            use dsc_lib::settings::fields::TracingLevelField;
            use test_case::test_case;

            #[test_case(TracingLevelField::Error => tracing::Level::ERROR; "error variant")]
            #[test_case(TracingLevelField::Warn => tracing::Level::WARN; "warn variant")]
            #[test_case(TracingLevelField::Info => tracing::Level::INFO; "info variant")]
            #[test_case(TracingLevelField::Debug => tracing::Level::DEBUG; "debug variant")]
            #[test_case(TracingLevelField::Trace => tracing::Level::TRACE; "trace variant")]
            fn tokio_tracing_level(input: TracingLevelField) -> tracing::Level {
                input.into()
            }

            #[test_case(TracingLevelField::Error => "error".to_string(); "error variant")]
            #[test_case(TracingLevelField::Warn => "warn".to_string(); "warn variant")]
            #[test_case(TracingLevelField::Info => "info".to_string(); "info variant")]
            #[test_case(TracingLevelField::Debug => "debug".to_string(); "debug variant")]
            #[test_case(TracingLevelField::Trace => "trace".to_string(); "trace variant")]
            fn string(input: TracingLevelField) -> String {
                input.into()
            }
        }
    }
}

mod format_field {
    mod methods {
        mod parse {
            use dsc_lib::settings::DscSettingsError;
            use dsc_lib::settings::fields::TracingFormatField;
            use test_case::test_case;

            #[test_case("default" => matches TracingFormatField::Default ; "lower case default")]
            #[test_case("DEFAULT" => matches TracingFormatField::Default ; "upper case default")]
            #[test_case("dEfAuLt" => matches TracingFormatField::Default ; "mixed case default")]
            #[test_case("plaintext" => matches TracingFormatField::Plaintext ; "lower case warn")]
            #[test_case("PLAINTEXT" => matches TracingFormatField::Plaintext ; "upper case warn")]
            #[test_case("pLaInTeXt" => matches TracingFormatField::Plaintext ; "mixed case warn")]
            #[test_case("json" => matches TracingFormatField::Json ; "lower case info")]
            #[test_case("JSON" => matches TracingFormatField::Json ; "upper case info")]
            #[test_case("jSoN" => matches TracingFormatField::Json ; "mixed case info")]
            fn valid(input: &str) -> TracingFormatField {
                TracingFormatField::parse(input)
                    .expect(&format!("expected input '{}' to be valid", input))
            }

            #[test_case(""; "empty string")]
            #[test_case(" "; "space only string")]
            #[test_case("invalid"; "unrecognized variant")]
            fn invalid(input: &str) {
                let err = TracingFormatField::parse(input)
                    .expect_err(&format!("expected input '{input}' to be invalid"));

                match err {
                    DscSettingsError::InvalidTracingFormat { text } => {
                        pretty_assertions::assert_eq!(text, input);
                    }
                    _ => panic!("expected InvalidTracingFormat error but got: {err:?}"),
                }
            }
        }
    }

    crate::macros::test_dsc_repo_schema! {
        define_statics: { dsc_lib::settings::fields::TracingFormatField },
        test_meta_schema: {},
        test_docs: {},
        test_validation: {
            default_string_value_is_valid => {
                input_json: json!("default"),
                expected_valid: true,
            },
            plaintext_string_value_is_valid => {
                input_json: json!("plaintext"),
                expected_valid: true,
            },
            json_string_value_is_valid => {
                input_json: json!("json"),
                expected_valid: true,
            },
            unknown_string_value_is_invalid => {
                input_json: json!("unknown"),
                expected_valid: false,
            },
            empty_string_value_is_invalid => {
                input_json: json!(""),
                expected_valid: false,
            },
            boolean_value_is_invalid => {
                input_json: json!(true),
                expected_valid: false,
            },
            integer_value_is_invalid => {
                input_json: json!(42),
                expected_valid: false,
            },
            number_value_is_invalid => {
                input_json: json!(3.14),
                expected_valid: false,
            },
            null_value_is_invalid => {
                input_json: json!(null),
                expected_valid: false,
            },
        }
    }

    mod serde {
        use dsc_lib::settings::fields::TracingFormatField;
        use serde_json::{json, Value};
        use test_case::test_case;

        #[test_case(&TracingFormatField::Default => json!("default"); "default variant")]
        #[test_case(&TracingFormatField::Plaintext => json!("plaintext"); "plaintext variant")]
        #[test_case(&TracingFormatField::Json => json!("json"); "json variant")]
        fn serializing(input: &TracingFormatField) -> serde_json::Value {
            serde_json::to_value(input)
                .expect("serialization should never fail")
        }

        #[test_case(&json!("invalid"); "unrecognized variant string value")]
        #[test_case(&json!(123); "integer value")]
        #[test_case(&json!(true); "boolean value")]
        #[test_case(&json!(null); "null value")]
        #[test_case(&json!(["info"]); "array value")]
        #[test_case(&json!({"format": "json"}); "object value")]
        fn deserializing_invalid(input_value: &Value) {
            serde_json::from_value::<TracingFormatField>(input_value.clone())
                .expect_err(&format!("json value '{input_value}' should be invalid"));
        }

        #[test_case(&json!("default") => TracingFormatField::Default; "default variant")]
        #[test_case(&json!("plaintext") => TracingFormatField::Plaintext; "plaintext variant")]
        #[test_case(&json!("json") => TracingFormatField::Json; "json variant")]
        fn deserializing_valid(input_value: &Value) -> TracingFormatField {
            serde_json::from_value::<TracingFormatField>(input_value.clone())
                .expect(&format!(
                    "expected input to serialize but failed for input: {}",
                    serde_json::to_string_pretty(input_value).unwrap(),
                ))
        }
    }
    mod traits {
        mod display {
            use dsc_lib::settings::fields::TracingFormatField;
            use test_case::test_case;

            #[test_case("default"; "default variant")]
            #[test_case("plaintext"; "plaintext variant")]
            #[test_case("json"; "json variant")]
            fn format(variant: &str) {
                pretty_assertions::assert_eq!(
                    format!("format: {}", TracingFormatField::parse(variant).unwrap()),
                    format!("format: {variant}"),
                )
            }

            #[test_case("default"; "default variant")]
            #[test_case("plaintext"; "plaintext variant")]
            #[test_case("json"; "json variant")]
            fn to_string(variant: &str) {
                pretty_assertions::assert_eq!(
                    TracingFormatField::parse(variant).unwrap().to_string(),
                    variant.to_string(),
                );
            }
        }
        mod from_str {
            use dsc_lib::settings::{DscSettingsError, fields::TracingFormatField};
            // use std::str::FromStr;
            use test_case::test_case;

            #[test_case("default" => matches Ok(_); "default variant")]
            #[test_case("plaintext" => matches Ok(_); "plaintext variant")]
            #[test_case("json" => matches Ok(_); "json variant")]
            #[test_case("invalid" => matches Err(_); "invalid variant string")]
            #[test_case("" => matches Err(_); "empty string")]
            #[test_case(" " => matches Err(_); "space only string")]
            fn parse(input: &str) -> Result<TracingFormatField, DscSettingsError> {
                input.parse()
            }
        }
        mod try_from {
            use dsc_lib::settings::{DscSettingsError, fields::TracingFormatField};
            use std::convert::TryFrom;
            use test_case::test_case;

            #[test_case("default" => matches Ok(_); "default variant")]
            #[test_case("plaintext" => matches Ok(_); "plaintext variant")]
            #[test_case("json" => matches Ok(_); "json variant")]
            #[test_case("invalid" => matches Err(_); "invalid variant string")]
            #[test_case("" => matches Err(_); "empty string")]
            #[test_case(" " => matches Err(_); "space only string")]
            fn string(input: &str) -> Result<TracingFormatField, DscSettingsError> {
                TracingFormatField::try_from(input.to_string())
            }
        }
        mod into {
            use dsc_lib::settings::fields::TracingFormatField;
            use test_case::test_case;

            #[test_case(TracingFormatField::Default => "default".to_string(); "default variant")]
            #[test_case(TracingFormatField::Plaintext => "plaintext".to_string(); "plaintext variant")]
            #[test_case(TracingFormatField::Json => "json".to_string(); "json variant")]
            fn string(input: TracingFormatField) -> String {
                input.into()
            }
        }
    }
}
