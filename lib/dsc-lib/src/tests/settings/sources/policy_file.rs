// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::settings::DscSettingsError;
use crate::settings::fields::{
    ForbidIgnoreSettingsFileField,
    IgnoreSettingsFileField,
    TracingFileData,
    TracingLevelField,
    TracingFormatField,
    ResourcePathFileData
};
use crate::settings::sources::PolicyFileData;
use crate::tests::stubs::path::*;

/// Tests for [`PolicyFileData::from_file()`] with stubbed file system context.
mod from_file {
    use serial_test::serial;

    use crate::settings::DscSettingsScope;

use super::*;

    #[test]
    #[serial]
    fn when_file_not_exist() {
        stubbed_path_context_map!(
            "/test/file.json" => StubbedPathContext {
                should_exist: false,
                ..Default::default()
            }
        );

        let err = PolicyFileData::from_file(Path::new("/test/file.json"))
            .expect_err("should raise an error");

        match err {
            DscSettingsError::DataFileReadError{file_path, scope, source} => {
                assert_eq!(file_path, String::from("/test/file.json"));
                assert_eq!(scope, DscSettingsScope::Policy);
                assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
            },
            _ => panic!("expected FileReadError but got {err:?}"),
        }
    }

    #[serial]
    #[test]
    fn when_file_empty() {
        stubbed_path_context_map!(
            "/test/file.json" => StubbedPathContext {
                should_exist: true,
                read_result: Ok(String::new()),
                ..Default::default()
            }
        );

        let err = PolicyFileData::from_file(Path::new("/test/file.json"))
            .expect_err("should raise an error");

        match err {
            DscSettingsError::DataFileEmpty{file_path, scope} => {
                assert_eq!(file_path, String::from("/test/file.json"));
                assert_eq!(scope, DscSettingsScope::Policy);
            },
            _ => panic!("expected DataFileEmpty but got {err:?}"),
        }
    }

    #[serial]
    #[test]
    fn when_file_only_whitespace() {
        stubbed_path_context_map!(
            "/test/file.json" => StubbedPathContext {
                should_exist: true,
                read_result: Ok(String::from("   ")),
                ..Default::default()
            }
        );

        let err = PolicyFileData::from_file(Path::new("/test/file.json"))
            .expect_err("should raise an error");

        match err {
            DscSettingsError::DataFileEmpty{file_path, scope} => {
                assert_eq!(file_path, String::from("/test/file.json"));
                assert_eq!(scope, DscSettingsScope::Policy);
            },
            _ => panic!("expected DataFileEmpty but got {err:?}"),
        }
    }

    #[serial]
    #[test]
    fn when_file_has_unparseable_json() {
        stubbed_path_context_map!(
            "/test/file.json" => StubbedPathContext {
                should_exist: true,
                read_result: Ok(String::from("{ invalid json: true }")),
                ..Default::default()
            }
        );

        let err = PolicyFileData::from_file(Path::new("/test/file.json"))
            .expect_err("should raise an error");

        match err {
            DscSettingsError::DataFileUnparseable{file_path, scope, source} => {
                assert_eq!(file_path, String::from("/test/file.json"));
                assert_eq!(scope, DscSettingsScope::Policy);
                assert!(source.is_syntax());
            },
            _ => panic!("expected ParseDataFileError but got {err:?}"),
        }
    }

    #[serial]
    #[test]
    fn when_file_is_empty_object() {
        stubbed_path_context_map!(
            "/test/file.json" => StubbedPathContext {
                should_exist: true,
                read_result: Ok(String::from("{}")),
                ..Default::default()
            }
        );

        let data = PolicyFileData::from_file(Path::new("/test/file.json"))
            .expect("should parse successfully");

        pretty_assertions::assert_eq!(data, PolicyFileData::default());
    }

    #[serial]
    #[test]
    fn when_file_defines_unknown_field() {
        stubbed_path_context_map!(
            "/test/file.json" => StubbedPathContext {
                should_exist: true,
                read_result: Ok(String::from("{ \"unknown_field\": true }")),
                ..Default::default()
            }
        );

        let data = PolicyFileData::from_file(Path::new("/test/file.json"))
            .expect("should parse successfully even with unknown fields");

        pretty_assertions::assert_eq!(data, PolicyFileData::default());
    }

    #[serial]
    #[test]
    fn when_file_has_all_valid_data() {
        let json = serde_json::json!({
            "forbidIgnoreSettingsFile": false,
            "ignoreSettingsFile": true,
            "tracing": {
                "level": "info",
                "format": "json"
            },
            "resourcePath": {
                "appendEnvPath": true,
                "directories": ["dir1", "dir2"],
                "restricted": false,
            }
        });
        let json_str = serde_json::to_string_pretty(&json).unwrap();
        stubbed_path_context_map!(
            "/test/file.json" => StubbedPathContext {
                should_exist: true,
                read_result: Ok(json_str),
                ..Default::default()
            }
        );

        let data = PolicyFileData::from_file(Path::new("/test/file.json"))
            .expect("should parse successfully");

        // Replace with the actual expected data structure
        let expected_data = PolicyFileData {
            forbid_ignore_settings_file: Some(ForbidIgnoreSettingsFileField::new(false)),
            ignore_settings_file: Some(IgnoreSettingsFileField::new(true)),
            tracing: Some(TracingFileData {
                level: Some(TracingLevelField::Info),
                format: Some(TracingFormatField::Json),
            }),
            resource_path: Some(ResourcePathFileData {
                append_env_path: Some(true),
                directories: Some(vec![
                    PathBuf::from("dir1").into(),
                    PathBuf::from("dir2").into()
                ]),
                restricted: Some(false),
            }),
            ..Default::default()
        };

        pretty_assertions::assert_eq!(data, expected_data);
    }

    #[serial]
    #[test]
    fn when_file_has_invalid_data() {
        let json_data = serde_json::json!({
            "forbidIgnoreSettingsFile": "not_a_boolean",
            "ignoreSettingsFile": "not_a_boolean",
            "tracing": {
                "level": "invalid_level",
                "format": "invalid_format"
            },
            "resourcePath": {
                "appendEnvPath": "not_a_boolean",
                "directories": "not_an_array",
                "restricted": "not_a_boolean"
            }
        });
        let json_str = &serde_json::to_string_pretty(&json_data).unwrap();
        stubbed_path_context_map!(
            "/test/file.json" => StubbedPathContext {
                should_exist: true,
                read_result: Ok(json_str.clone()),
                ..Default::default()
            }
        );

        match PolicyFileData::from_file(Path::new("/test/file.json")) {
            Ok(data) => panic!("expected {json_str} to fail parsing, but got {data:?}"),
            Err(e) => match e {
                DscSettingsError::DataFileUnparseable { file_path, scope, source } => {
                    assert_eq!(file_path, String::from("/test/file.json"));
                    assert_eq!(scope, DscSettingsScope::Policy.to_string());
                    assert!(source.is_data());
                    // Only the first failing field reports invalid
                    assert!(
                        source.to_string().contains("not_a_boolean"),
                        "expected error message to contain 'not_a_boolean': {source}"
                    );
                },
                _ => panic!("expected ParseDataFileError, but got {e:?}"),
            }
        }
    }
}
