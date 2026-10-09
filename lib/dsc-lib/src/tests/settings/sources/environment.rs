// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::env::VarError;
use std::path::PathBuf;

use crate::settings::DscSettingsError;
use crate::settings::fields::{IgnoreSettingsFileField, TracingFormatField, TracingLevelField};
use crate::settings::sources::EnvironmentData;
use crate::tests::stubs::env::*;

/// Tests for [`EnvironmentData::get_env_trace_level()`] with stubbed
/// environment variables.
mod get_env_trace_level {
    use test_case::test_case;
    use serial_test::serial;

    use super::*;

    #[serial]
    #[test] fn when_not_defined() {
        stubbed_env_context_map! {
            "TRACE_LEVEL" => Err(VarError::NotPresent),
        };

        let result = EnvironmentData::get_env_trace_level();
        assert!(matches!(result, Ok(None)));
    }

    #[test_case("trace" => TracingLevelField::Trace; "trace lower case")]
    #[test_case("TRACE" => TracingLevelField::Trace; "trace upper case")]
    #[test_case("tRaCe" => TracingLevelField::Trace; "trace mixed case")]
    #[test_case("debug" => TracingLevelField::Debug; "debug lower case")]
    #[test_case("DEBUG" => TracingLevelField::Debug; "debug upper case")]
    #[test_case("dEbUg" => TracingLevelField::Debug; "debug mixed case")]
    #[test_case("info" => TracingLevelField::Info; "info lower case")]
    #[test_case("INFO" => TracingLevelField::Info; "info upper case")]
    #[test_case("iNfO" => TracingLevelField::Info; "info mixed case")]
    #[test_case("warn" => TracingLevelField::Warn; "warn lower case")]
    #[test_case("WARN" => TracingLevelField::Warn; "warn upper case")]
    #[test_case("wArN" => TracingLevelField::Warn; "warn mixed case")]
    #[test_case("error" => TracingLevelField::Error; "error lower case")]
    #[test_case("ERROR" => TracingLevelField::Error; "error upper case")]
    #[test_case("eRrOr" => TracingLevelField::Error; "error mixed case")]
    #[serial]
    fn when_defined_with_valid_value(value: &str) -> TracingLevelField {
        stubbed_env_context_map! {
            "DSC_TRACE_LEVEL" => Ok(value.to_string()),
        };

        EnvironmentData::get_env_trace_level()
            .expect("TRACE_LEVEL should be defined and parseable")
            .expect("TRACE_LEVEL should be defined and parseable")
    }

    #[test_case("invalid"; "invalid value")]
    #[test_case(" "; "space-only value")]
    #[test_case(" debug"; "leading space before valid value")]
    #[test_case("debug "; "trailing space after valid value")]
    #[serial]
    fn when_defined_with_invalid_value(value: &str) {
        stubbed_env_context_map! {
            "DSC_TRACE_LEVEL" => Ok(value.to_string()),
        };

        let err = EnvironmentData::get_env_trace_level()
            .expect_err("DSC_TRACE_LEVEL should be defined with invalid value");

        match err {
            DscSettingsError::LoadEnvironmentError{env_var, source} => {
                assert_eq!(env_var, "DSC_TRACE_LEVEL");

                match *source {
                    DscSettingsError::InvalidTracingLevel{text} => {
                        assert_eq!(text, value);
                    },
                    _ => panic!("Expected InvalidTracingLevel as source but was {source:?}"),
                }
            }
            _ => panic!("Expected LoadEnvironmentError with InvalidTracingLevel source but was {err:?}"),
        }
    }
}

/// Tests for [`EnvironmentData::get_env_trace_format()`] with stubbed
/// environment variables.
mod get_env_trace_format {
    use test_case::test_case;
    use serial_test::serial;

    use super::*;

    #[serial]
    #[test]
    fn when_not_defined() {
        stubbed_env_context_map! {
            "DSC_TRACE_FORMAT" => Err(std::env::VarError::NotPresent),
        };

        let result = EnvironmentData::get_env_trace_format();
        assert!(matches!(result, Ok(None)));
    }

    #[test_case("default" => TracingFormatField::Default; "default lower case")]
    #[test_case("DEFAULT" => TracingFormatField::Default; "default upper case")]
    #[test_case("dEfAuLt" => TracingFormatField::Default; "default mixed case")]
    #[test_case("json" => TracingFormatField::Json; "json lower case")]
    #[test_case("JSON" => TracingFormatField::Json; "json upper case")]
    #[test_case("jSoN" => TracingFormatField::Json; "json mixed case")]
    #[test_case("plaintext" => TracingFormatField::Plaintext; "plaintext lower case")]
    #[test_case("PLAINTEXT" => TracingFormatField::Plaintext; "plaintext upper case")]
    #[test_case("pLaInTeXt" => TracingFormatField::Plaintext; "plaintext mixed case")]
    #[serial]
    fn when_defined_with_valid_value(value: &str) -> TracingFormatField {
        stubbed_env_context_map! {
            "DSC_TRACE_FORMAT" => Ok(value.to_string()),
        };

        EnvironmentData::get_env_trace_format()
            .expect("TRACE_FORMAT should be defined and parseable")
            .expect("TRACE_FORMAT should be defined and parseable")
    }

    #[test_case("invalid"; "invalid value")]
    #[test_case(" "; "space-only value")]
    #[test_case(" json"; "leading space before valid value")]
    #[test_case("json "; "trailing space after valid value")]
    #[serial]
    fn when_defined_with_invalid_value(value: &str) {
        stubbed_env_context_map! {
            "DSC_TRACE_FORMAT" => Ok(value.to_string()),
        };

        let err = EnvironmentData::get_env_trace_format()
            .expect_err("DSC_TRACE_FORMAT should be defined with invalid value");

        match err {
            DscSettingsError::LoadEnvironmentError{env_var, source} => {
                assert_eq!(env_var, "DSC_TRACE_FORMAT");

                match *source {
                    DscSettingsError::InvalidTracingFormat{text} => {
                        assert_eq!(text, value);
                    },
                    _ => panic!("Expected InvalidTracingFormat as source but was {source:?}"),
                }
            }
            _ => panic!("Expected LoadEnvironmentError with InvalidTracingFormat source but was {err:?}"),
        }
    }
}

/// Tests for [`EnvironmentData::get_env_resource_path()`] with stubbed
/// environment variables.
mod get_env_resource_path {
    use test_case::test_case;
    use serial_test::serial;

    use super::*;

    #[serial]
    #[test]
    fn when_not_defined() {
        stubbed_env_context_map! {
            "DSC_RESOURCE_PATH" => Err(std::env::VarError::NotPresent),
        };

        let result = EnvironmentData::get_env_resource_path();
        assert!(matches!(result, None));
    }

    #[test_case(vec!["/test/example"]; "single valid path")]
    #[test_case(vec!["/test/example", "/another/path"]; "multiple valid paths")]
    #[serial]
    fn when_defined(paths: Vec<&str>) {
        let paths = std::env::join_paths(paths.iter()).unwrap();
        let expected = std::env::split_paths(&paths)
            .collect::<Vec<PathBuf>>();
        stubbed_env_context_map! {
            "DSC_RESOURCE_PATH" => Ok(paths.to_string_lossy().into_owned()),
        };

        let result = EnvironmentData::get_env_resource_path()
            .expect("DSC_RESOURCE_PATH should be populated");
        pretty_assertions::assert_eq!(result, expected);
    }
}

/// Tests for [`EnvironmentData::get_env_restricted_path()`] with stubbed
/// environment variables.
mod get_env_restricted_path {
    use test_case::test_case;
    use serial_test::serial;

    use super::*;

    #[serial]
    #[test]
    fn when_not_defined() {
        stubbed_env_context_map! {
            "DSC_RESTRICTED_PATH" => Err(std::env::VarError::NotPresent),
        };

        let result = EnvironmentData::get_env_restricted_path();
        assert!(matches!(result, None));
    }

    #[test_case(vec!["/test/example"]; "single valid path")]
    #[test_case(vec!["/test/example", "/another/path"]; "multiple valid paths")]
    #[serial]
    fn when_defined(paths: Vec<&str>) {
        let paths = std::env::join_paths(paths.iter()).unwrap();
        let expected = std::env::split_paths(&paths)
            .collect::<Vec<PathBuf>>();
        stubbed_env_context_map! {
            "DSC_RESTRICTED_PATH" => Ok(paths.to_string_lossy().into_owned()),
        };

        let result = EnvironmentData::get_env_restricted_path()
            .expect("DSC_RESTRICTED_PATH should be populated");
        pretty_assertions::assert_eq!(result, expected);
    }
}

/// Tests for [`EnvironmentData::get_env_ignore_settings_file()`] with stubbed
/// environment variables.
mod get_env_ignore_settings_file {
    use super::*;

    use test_case::test_case;
    use serial_test::serial;

    #[serial]
    #[test]
    fn when_not_defined() {
        stubbed_env_context_map! {
            "DSC_IGNORE_SETTINGS_FILE" => Err(std::env::VarError::NotPresent),
        };

        let result = EnvironmentData::get_env_ignore_settings_file();
        assert!(matches!(result, Ok(None)));
    }

    #[test_case("true" => true; "true lower case")]
    #[test_case("TRUE" => true; "true upper case")]
    #[test_case("tRuE" => true; "true mixed case")]
    #[test_case("1" => true; "true as numeral 1")]
    #[test_case("false" => false; "false lower case")]
    #[test_case("FALSE" => false; "false upper case")]
    #[test_case("fAlSe" => false; "false mixed case")]
    #[test_case("0" => false; "false as numeral 0")]
    #[serial]
    fn when_defined_with_valid_value(value: &str) -> bool {
        stubbed_env_context_map! {
            "DSC_IGNORE_SETTINGS_FILE" => Ok(value.to_string()),
        };

        EnvironmentData::get_env_ignore_settings_file()
            .expect("DSC_IGNORE_SETTINGS_FILE should be defined and valid")
            .expect("DSC_IGNORE_SETTINGS_FILE should be defined and valid")
            .into()
    }

    #[test_case("invalid" ; "non-boolean string")]
    #[test_case("2" ; "numerical value greater than 1")]
    #[test_case("-1" ; "numerical value less than 0")]
    #[test_case("0.5"; "non-integer numerical value")]
    #[test_case(" " ; "whitespace only value")]
    #[test_case(" true" ; "valid value with leading whitespace")]
    #[test_case("true " ; "valid value with trailing whitespace")]
    #[serial]
    fn when_defined_with_invalid_value(env_var_value: &str) {
        stubbed_env_context_map! {
            "DSC_IGNORE_SETTINGS_FILE" => Ok(env_var_value.to_string()),
        };

        let err = EnvironmentData::get_env_ignore_settings_file()
            .expect_err("DSC_IGNORE_SETTINGS_FILE should be defined but invalid");

        match err {
            DscSettingsError::LoadEnvironmentError { env_var, source } => {
                assert_eq!(env_var, "DSC_IGNORE_SETTINGS_FILE");
                match *source {
                    DscSettingsError::EnvVarUnparseableBoolean { value } => {
                        assert_eq!(value, env_var_value);
                    }
                    _ => panic!("expected EnvVarUnparseableBoolean"),
                }
            },
            _ => panic!("expected LoadEnvironmentError error"),
        }
    }
}

/// Tests for [`EnvironmentData::from_env()`] with stubbed environment
/// variables.
mod from_env {
    use serial_test::serial;
    use super::*;

    #[serial]
    #[test]
    fn with_no_env_var_defined() {
        StubbedEnvContext::reset();

        let env_data = EnvironmentData::from_env();
        pretty_assertions::assert_eq!(env_data.dsc_resource_path, None);
        pretty_assertions::assert_eq!(env_data.dsc_restricted_path, None);
        pretty_assertions::assert_eq!(env_data.dsc_trace_format, None);
        pretty_assertions::assert_eq!(env_data.dsc_trace_level, None);
        pretty_assertions::assert_eq!(env_data.dsc_ignore_settings_file, None);
    }

    #[serial]
    #[test]
    fn with_all_env_var_defined_with_valid_value() {
        stubbed_env_context_map! {
            "DSC_IGNORE_SETTINGS_FILE" => Ok("true".to_string()),
            "DSC_RESOURCE_PATH" => Ok("/some/resource/path".to_string()),
            "DSC_RESTRICTED_PATH" => Ok("/some/restricted/path".to_string()),
            "DSC_TRACE_FORMAT" => Ok("json".to_string()),
            "DSC_TRACE_LEVEL" => Ok("info".to_string()),
        };

        let env_data = EnvironmentData::from_env();
        pretty_assertions::assert_eq!(
            env_data.dsc_ignore_settings_file,
            Some(IgnoreSettingsFileField::new(true))
        );
        pretty_assertions::assert_eq!(
            env_data.dsc_resource_path,
            Some(vec![PathBuf::from("/some/resource/path")])
        );
        pretty_assertions::assert_eq!(
            env_data.dsc_restricted_path,
            Some(vec![PathBuf::from("/some/restricted/path")])
        );
        pretty_assertions::assert_eq!(
            env_data.dsc_trace_format,
            Some(TracingFormatField::Json)
        );
        pretty_assertions::assert_eq!(
            env_data.dsc_trace_level,
            Some(TracingLevelField::Info)
        );
    }

    #[serial]
    #[test]
    fn with_some_env_var_defined_with_invalid_value() {
        stubbed_env_context_map! {
            "DSC_IGNORE_SETTINGS_FILE" => Ok("not_a_boolean".to_string()),
            "DSC_RESOURCE_PATH" => Ok("/some/resource/path".to_string()),
            "DSC_RESTRICTED_PATH" => Ok("/some/restricted/path".to_string()),
            "DSC_TRACE_FORMAT" => Ok("json".to_string()),
            "DSC_TRACE_LEVEL" => Ok("invalid".to_string()),
        };

        let env_data = EnvironmentData::from_env();
        pretty_assertions::assert_eq!(env_data.dsc_ignore_settings_file, None);
        pretty_assertions::assert_eq!(env_data.dsc_trace_level, None);
        pretty_assertions::assert_eq!(
            env_data.dsc_resource_path,
            Some(vec![PathBuf::from("/some/resource/path")])
        );
        pretty_assertions::assert_eq!(
            env_data.dsc_restricted_path,
            Some(vec![PathBuf::from("/some/restricted/path")])
        );
        pretty_assertions::assert_eq!(
            env_data.dsc_trace_format,
            Some(TracingFormatField::Json)
        );
    }
}

/// Tests for [`EnvironmentData::try_from_env()`] with stubbed environment
/// variables.
mod try_from_env {
    use serial_test::serial;
    use super::*;

    #[serial]
    #[test]
    fn with_no_env_var_defined() {
        StubbedEnvContext::reset();

        let env_data = EnvironmentData::try_from_env()
            .expect("EnvironmentData should be successfully created even with no environment variables defined");
        pretty_assertions::assert_eq!(env_data.dsc_resource_path, None);
        pretty_assertions::assert_eq!(env_data.dsc_restricted_path, None);
        pretty_assertions::assert_eq!(env_data.dsc_trace_format, None);
        pretty_assertions::assert_eq!(env_data.dsc_trace_level, None);
        pretty_assertions::assert_eq!(env_data.dsc_ignore_settings_file, None);
    }

    #[serial]
    #[test]
    fn with_all_env_var_defined_with_valid_value() {
        stubbed_env_context_map! {
            "DSC_IGNORE_SETTINGS_FILE" => Ok("true".to_string()),
            "DSC_RESOURCE_PATH" => Ok("/some/resource/path".to_string()),
            "DSC_RESTRICTED_PATH" => Ok("/some/restricted/path".to_string()),
            "DSC_TRACE_FORMAT" => Ok("json".to_string()),
            "DSC_TRACE_LEVEL" => Ok("info".to_string()),
        };

        let env_data = EnvironmentData::try_from_env()
            .expect("All DSC variables should be validly defined");
        pretty_assertions::assert_eq!(
            env_data.dsc_ignore_settings_file,
            Some(IgnoreSettingsFileField::new(true))
        );
        pretty_assertions::assert_eq!(
            env_data.dsc_resource_path,
            Some(vec![PathBuf::from("/some/resource/path")])
        );
        pretty_assertions::assert_eq!(
            env_data.dsc_restricted_path,
            Some(vec![PathBuf::from("/some/restricted/path")])
        );
        pretty_assertions::assert_eq!(
            env_data.dsc_trace_format,
            Some(TracingFormatField::Json)
        );
        pretty_assertions::assert_eq!(
            env_data.dsc_trace_level,
            Some(TracingLevelField::Info)
        );
    }

    #[serial]
    #[test]
    fn with_some_env_var_defined_with_invalid_value() {
        stubbed_env_context_map! {
            "DSC_IGNORE_SETTINGS_FILE" => Ok("not_a_boolean".to_string()),
            "DSC_RESOURCE_PATH" => Ok("/some/resource/path".to_string()),
            "DSC_RESTRICTED_PATH" => Ok("/some/restricted/path".to_string()),
            "DSC_TRACE_FORMAT" => Ok("json".to_string()),
            "DSC_TRACE_LEVEL" => Ok("invalid".to_string()),
        };

        let err = EnvironmentData::try_from_env()
            .expect_err("At least one DSC variable should raise an error");
        match err {
            DscSettingsError::LoadEnvironmentMultipleErrors{errors} => {
                assert_eq!(errors.len(), 2);
                assert!(errors.iter().any(|e| matches!(e,
                        DscSettingsError::LoadEnvironmentError{env_var, ..} if *env_var == "DSC_IGNORE_SETTINGS_FILE"
                )));
                assert!(errors.iter().any(|e| matches!(e,
                    DscSettingsError::LoadEnvironmentError{env_var, ..} if *env_var == "DSC_TRACE_LEVEL"
                )));
            },
            _ => panic!("Expected LoadEnvironmentMultipleErrors"),
        }
    }
}
