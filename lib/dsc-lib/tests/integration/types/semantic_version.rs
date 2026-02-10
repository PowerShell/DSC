// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    use dsc_lib::{dscerror::DscError, types::SemanticVersion};
    use test_case::test_case;

    #[test]
    fn new() {
        let actual = SemanticVersion::new(1, 0, 0);

        pretty_assertions::assert_eq!(actual.to_string(), "1.0.0".to_string());
    }

    #[test_case("1.0.0" => matches Ok(_); "valid stable semantic version")]
    #[test_case("1.0.0-rc.1" => matches Ok(_); "valid prerelease semantic version")]
    #[test_case("1.0.0+ci.123" => matches Ok(_); "valid stable semantic version with build metadata")]
    #[test_case("1.0.0-rc.1+ci.123" => matches Ok(_); "valid prerelease semantic version with build metadata")]
    #[test_case("1" => matches Err(_); "major version only is invalid")]
    #[test_case("1.0" => matches Err(_); "missing patch version is invalid")]
    #[test_case("1.2.c" => matches Err(_); "version segment as non-digit is invalid")]
    fn parse(value: &str) -> Result<SemanticVersion, DscError> {
        SemanticVersion::parse(value)
    }
}

#[cfg(test)]
mod schema {
    use std::sync::LazyLock;

    use dsc_lib::types::SemanticVersion;
    use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    use jsonschema::Validator;
    use regex::Regex;
    use schemars::{schema_for, Schema};
    use serde_json::{json, Value};
    use test_case::test_case;

    static SCHEMA: LazyLock<Schema> = LazyLock::new(|| schema_for!(SemanticVersion));
    static VALIDATOR: LazyLock<Validator> =
        LazyLock::new(|| Validator::new((&*SCHEMA).as_value()).unwrap());
    static KEYWORD_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\w+(\.\w+)+$").expect("pattern is valid"));

    #[test_case("title")]
    #[test_case("description")]
    #[test_case("markdownDescription")]
    #[test_case("patternErrorMessage")]
    fn has_documentation_keyword(keyword: &str) {
        let schema = &*SCHEMA;
        let value = schema
            .get_keyword_as_str(keyword)
            .expect(format!("expected keyword '{keyword}' to be defined").as_str());

        assert!(
            !(&*KEYWORD_PATTERN).is_match(value),
            "Expected keyword '{keyword}' to be defined in translation, but was set to i18n key '{value}'"
        );
    }

    #[test_case(&json!("1.0.0") => true; "valid stable semantic version string value is valid")]
    #[test_case(&json!("1.0.0-rc.1") => true; "valid prerelease semantic version string value is valid")]
    #[test_case(&json!("1.0.0+ci.123") => true; "valid stable semantic version with build metadata string value is valid")]
    #[test_case(&json!("1.0.0-rc.1+ci.123") => true; "valid prerelease semantic version with build metadata string value is valid")]
    #[test_case(&json!("1") => false; "major version only string value is invalid")]
    #[test_case(&json!("1.0") => false; "missing patch version string value is invalid")]
    #[test_case(&json!("1.2.c") => false; "version segment as non-digit string value is invalid")]
    #[test_case(&json!(true) => false; "boolean value is invalid")]
    #[test_case(&json!(1) => false; "integer value is invalid")]
    #[test_case(&json!(1.2) => false; "float value is invalid")]
    #[test_case(&json!({"req": "1.2.3"}) => false; "object value is invalid")]
    #[test_case(&json!(["1.2.3"]) => false; "array value is invalid")]
    #[test_case(&serde_json::Value::Null => false; "null value is invalid")]
    fn validation(input_json: &Value) -> bool {
        (&*VALIDATOR).validate(input_json).is_ok()
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::types::SemanticVersion;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case("1.0.0"; "stable semantic version")]
    #[test_case("1.0.0-rc.1"; "prerelease semantic version")]
    #[test_case("1.0.0+ci.123"; "stable semantic version with build metadata")]
    #[test_case("1.0.0-rc.1+ci.123"; "prerelease semantic version with build metadata")]
    fn serializing(version: &str) {
        let actual = serde_json::to_string(
            &SemanticVersion::parse(version).expect("parse should never fail"),
        )
        .expect("serialization should never fail");

        let expected = format!(r#""{version}""#);

        pretty_assertions::assert_eq!(actual, expected);
    }

    #[test_case(json!("1.0.0") => matches Ok(_); "stable semantic version string value is valid")]
    #[test_case(json!("1.0.0-rc.1") => matches Ok(_); "prerelease semantic version string value is valid")]
    #[test_case(json!("1.0.0+ci.123") => matches Ok(_); "stable semantic version with build metadata string value is valid")]
    #[test_case(json!("1.0.0-rc.1+ci.123") => matches Ok(_); "prerelease semantic version with build metadata string value is valid")]
    #[test_case(json!("1") => matches Err(_); "major version only string value is invalid")]
    #[test_case(json!("1.0") => matches Err(_); "missing patch version string value is invalid")]
    #[test_case(json!("1.2.c") => matches Err(_); "version segment as non-digit string value is invalid")]
    #[test_case(json!(true) => matches Err(_); "boolean value is invalid")]
    #[test_case(json!(1) => matches Err(_); "integer value is invalid")]
    #[test_case(json!(1.2) => matches Err(_); "float value is invalid")]
    #[test_case(json!({"req": "1.2.3"}) => matches Err(_); "object value is invalid")]
    #[test_case(json!(["1.2.3"]) => matches Err(_); "array value is invalid")]
    #[test_case(serde_json::Value::Null => matches Err(_); "null value is invalid")]
    fn deserializing(value: Value) -> Result<SemanticVersion, serde_json::Error> {
        serde_json::from_value::<SemanticVersion>(value)
    }
}

#[cfg(test)]
mod traits {
    #[cfg(test)]
    mod default {
        use dsc_lib::types::SemanticVersion;

        #[test]
        fn default() {
            let actual = SemanticVersion::default();
            let expected = SemanticVersion::new(0, 0, 0);

            pretty_assertions::assert_eq!(actual, expected);
        }
    }

    #[cfg(test)]
    mod display {
        use dsc_lib::types::SemanticVersion;
        use test_case::test_case;

        #[test_case("1.0.0", "1.0.0"; "valid stable semantic version")]
        #[test_case("1.0.0-rc.1", "1.0.0-rc.1"; "valid prerelease semantic version")]
        #[test_case("1.0.0+ci.123", "1.0.0+ci.123"; "valid stable semantic version with build metadata")]
        #[test_case("1.0.0-rc.1+ci.123", "1.0.0-rc.1+ci.123"; "valid prerelease semantic version with build metadata")]
        fn format(version: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                format!("req: '{}'", SemanticVersion::parse(version).unwrap()),
                format!("req: '{}'", expected)
            )
        }

        #[test_case("1.0.0", "1.0.0"; "valid stable semantic version")]
        #[test_case("1.0.0-rc.1", "1.0.0-rc.1"; "valid prerelease semantic version")]
        #[test_case("1.0.0+ci.123", "1.0.0+ci.123"; "valid stable semantic version with build metadata")]
        #[test_case("1.0.0-rc.1+ci.123", "1.0.0-rc.1+ci.123"; "valid prerelease semantic version with build metadata")]
        fn to_string(requirement: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                SemanticVersion::parse(requirement).unwrap().to_string(),
                expected.to_string()
            )
        }
    }

    #[cfg(test)]
    mod from {
        use dsc_lib::types::SemanticVersion;
        use semver::Version;

        #[test]
        fn semver_version() {
            let _ = SemanticVersion::from(Version::new(1, 0, 0));
        }
    }

    #[cfg(test)]
    mod from_str {
        use std::str::FromStr;

        use dsc_lib::{dscerror::DscError, types::SemanticVersion};
        use test_case::test_case;

        #[test_case("1.0.0" => matches Ok(_); "valid stable semantic version")]
        #[test_case("1.0.0-rc.1" => matches Ok(_); "valid prerelease semantic version")]
        #[test_case("1.0.0+ci.123" => matches Ok(_); "valid stable semantic version with build metadata")]
        #[test_case("1.0.0-rc.1+ci.123" => matches Ok(_); "valid prerelease semantic version with build metadata")]
        #[test_case("1" => matches Err(_); "major version only string value is invalid")]
        #[test_case("1.0" => matches Err(_); "missing patch version string value is invalid")]
        #[test_case("1.2.c" => matches Err(_); "version segment as non-digit string value is invalid")]
        fn from_str(text: &str) -> Result<SemanticVersion, DscError> {
            SemanticVersion::from_str(text)
        }

        #[test_case("1.0.0" => matches Ok(_); "valid stable semantic version")]
        #[test_case("1.0.0-rc.1" => matches Ok(_); "valid prerelease semantic version")]
        #[test_case("1.0.0+ci.123" => matches Ok(_); "valid stable semantic version with build metadata")]
        #[test_case("1.0.0-rc.1+ci.123" => matches Ok(_); "valid prerelease semantic version with build metadata")]
        #[test_case("1" => matches Err(_); "major version only string value is invalid")]
        #[test_case("1.0" => matches Err(_); "missing patch version string value is invalid")]
        #[test_case("1.2.c" => matches Err(_); "version segment as non-digit string value is invalid")]
        fn parse(text: &str) -> Result<SemanticVersion, DscError> {
            text.parse()
        }
    }

    #[cfg(test)]
    mod try_from {
        use dsc_lib::{dscerror::DscError, types::SemanticVersion};
        use test_case::test_case;

        #[test_case("1.0.0" => matches Ok(_); "valid stable semantic version")]
        #[test_case("1.0.0-rc.1" => matches Ok(_); "valid prerelease semantic version")]
        #[test_case("1.0.0+ci.123" => matches Ok(_); "valid stable semantic version with build metadata")]
        #[test_case("1.0.0-rc.1+ci.123" => matches Ok(_); "valid prerelease semantic version with build metadata")]
        #[test_case("1" => matches Err(_); "major version only string value is invalid")]
        #[test_case("1.0" => matches Err(_); "missing patch version string value is invalid")]
        #[test_case("1.2.c" => matches Err(_); "version segment as non-digit string value is invalid")]
        fn string(text: &str) -> Result<SemanticVersion, DscError> {
            SemanticVersion::try_from(text.to_string())
        }

        #[test_case("1.0.0" => matches Ok(_); "valid stable semantic version")]
        #[test_case("1.0.0-rc.1" => matches Ok(_); "valid prerelease semantic version")]
        #[test_case("1.0.0+ci.123" => matches Ok(_); "valid stable semantic version with build metadata")]
        #[test_case("1.0.0-rc.1+ci.123" => matches Ok(_); "valid prerelease semantic version with build metadata")]
        #[test_case("1" => matches Err(_); "major version only string value is invalid")]
        #[test_case("1.0" => matches Err(_); "missing patch version string value is invalid")]
        #[test_case("1.2.c" => matches Err(_); "version segment as non-digit string value is invalid")]
        fn str(text: &str) -> Result<SemanticVersion, DscError> {
            SemanticVersion::try_from(text)
        }
    }

    #[cfg(test)]
    mod into {
        use dsc_lib::types::SemanticVersion;
        use semver::Version;

        #[test]
        fn semver_version() {
            let _: Version = SemanticVersion::new(1, 0, 0).into();
        }

        #[test]
        fn string() {
            let _: String = SemanticVersion::new(1, 0, 0).into();
        }
    }

    #[cfg(test)]
    mod as_ref {
        use dsc_lib::types::SemanticVersion;
        use semver::Version;

        #[test]
        fn semver_version() {
            let _: &Version = SemanticVersion::new(1, 0, 0).as_ref();
        }
    }

    #[cfg(test)]
    mod deref {
        use dsc_lib::types::SemanticVersion;

        #[test]
        fn semver_version() {
            let v = SemanticVersion::new(1, 2, 3);

            pretty_assertions::assert_eq!(v.major, 1u64);
            pretty_assertions::assert_eq!(v.minor, 2u64);
            pretty_assertions::assert_eq!(v.patch, 3u64);
            pretty_assertions::assert_eq!(v.pre, semver::Prerelease::EMPTY);
            pretty_assertions::assert_eq!(v.build, semver::BuildMetadata::EMPTY);
        }
    }

    #[cfg(test)]
    mod partial_eq {
        use dsc_lib::types::SemanticVersion;
        use test_case::test_case;

        #[test_case("1.2.3", "1.2.3", true; "identical semantic versions")]
        #[test_case("1.2.3", "3.2.1", false; "different semantic versions")]
        #[test_case("1.2.3", "1.2.3-rc.1", false; "semantic version and prerelease")]
        #[test_case("1.2.3", "1.2.3+ci.123", false; "semantic version and build metadata")]
        fn semantic_version(lhs: &str, rhs: &str, should_be_equal: bool) {
            if should_be_equal {
                pretty_assertions::assert_eq!(
                    SemanticVersion::parse(lhs).unwrap(),
                    SemanticVersion::parse(rhs).unwrap()
                )
            } else {
                pretty_assertions::assert_ne!(
                    SemanticVersion::parse(lhs).unwrap(),
                    SemanticVersion::parse(rhs).unwrap()
                )
            }
        }

        #[test_case("1.2.3", "1.2.3", true; "SemanticVersion and identical semver::Version")]
        #[test_case("1.2.3", "3.2.1", false; "different versions")]
        #[test_case("1.2.3", "1.2.3-rc.1", false; "SemanticVersion and semver::Version with prerelease")]
        #[test_case("1.2.3", "1.2.3+ci.123", false; "SemanticVersion and semver::Version with build metadata")]
        fn semver_version(
            semantic_version_string: &str,
            semver_version_string: &str,
            should_be_equal: bool,
        ) {
            let semantic_version = SemanticVersion::parse(semantic_version_string).unwrap();
            let semver_version = semver::Version::parse(semver_version_string).unwrap();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                semantic_version == semver_version,
                should_be_equal,
                "expected comparison of {semantic_version} and {semver_version} to be {should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                semver_version == semantic_version,
                should_be_equal,
                "expected comparison of {semver_version} and {semantic_version} to be {should_be_equal}"
            );
        }

        #[test_case("1.2.3", "1.2.3", true; "SemanticVersion and identical string")]
        #[test_case("1.2.3", "3.2.1", false; "SemanticVersion and different version string")]
        #[test_case("1.2.3", "1.2.3-rc.1", false; "SemanticVersion and version string with prerelease")]
        #[test_case("1.2.3", "1.2.3+ci.123", false; "SemanticVersion and version string with build metadata")]
        fn string(semantic_version_string: &str, string_slice: &str, should_be_equal: bool) {
            let semantic_version = SemanticVersion::parse(semantic_version_string).unwrap();
            let string = string_slice.to_string();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                semantic_version == string,
                should_be_equal,
                "expected comparison of {semantic_version} and {string} to be {should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string == semantic_version,
                should_be_equal,
                "expected comparison of {string} and {semantic_version} to be {should_be_equal}"
            );
        }

        #[test_case("1.2.3", "1.2.3", true; "SemanticVersion and identical string slice")]
        #[test_case("1.2.3", "3.2.1", false; "SemanticVersion and different version string slice")]
        #[test_case("1.2.3", "1.2.3-rc.1", false; "SemanticVersion and version string slice with prerelease")]
        #[test_case("1.2.3", "1.2.3+ci.123", false; "SemanticVersion and version string slice with build metadata")]
        fn str(semantic_version_string: &str, string_slice: &str, should_be_equal: bool) {
            let semantic_version = SemanticVersion::parse(semantic_version_string).unwrap();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                semantic_version == string_slice,
                should_be_equal,
                "expected comparison of {semantic_version} and {string_slice} to be {should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string_slice == semantic_version,
                should_be_equal,
                "expected comparison of {string_slice} and {semantic_version} to be {should_be_equal}"
            );
        }
    }

    #[cfg(test)]
    mod partial_ord {
        use std::cmp::Ordering;

        use dsc_lib::types::SemanticVersion;
        use test_case::test_case;

        #[test_case("1.2.3", "1.2.3", Ordering::Equal; "equal stable versions")]
        #[test_case("1.2.3-rc.1", "1.2.3-rc.1", Ordering::Equal; "equal prerelease versions")]
        #[test_case("1.2.3+ci.1", "1.2.3+ci.1", Ordering::Equal; "equal stable versions with build metadata")]
        #[test_case("1.2.3-rc.1+ci.1", "1.2.3-rc.1+ci.1", Ordering::Equal; "equal prerelease versions with build metadata")]
        #[test_case("3.2.1", "1.2.3", Ordering::Greater; "newer stable is greater than older stable")]
        #[test_case("1.2.3", "3.2.1", Ordering::Less; "older stable is less than newer stable")]
        #[test_case("1.2.3", "1.2.3-rc.1", Ordering::Greater; "stable is greater than prerelease")]
        #[test_case("1.2.3", "1.2.3+ci.1", Ordering::Less; "stable without build is less than stable with build")]
        #[test_case("1.2.3-rc.1", "1.2.3-rc.1+ci.1", Ordering::Less; "prerelease without build is less than prerelease with build")]
        #[test_case("1.2.3-A-0", "1.2.3-A00", Ordering::Less; "prerelease hyphen is less than digit")]
        #[test_case("1.2.3-A0A", "1.2.3-AAA", Ordering::Less; "prerelease digit is less than uppercase")]
        #[test_case("1.2.3-AAA", "1.2.3-AaA", Ordering::Less; "prerelease uppercase is less than lowercase")]
        #[test_case("1.2.3+A-0", "1.2.3+A00", Ordering::Less; "build metadata hyphen is less than digit")]
        #[test_case("1.2.3+A0A", "1.2.3+AAA", Ordering::Less; "build metadata digit is less than uppercase")]
        #[test_case("1.2.3+AAA", "1.2.3+AaA", Ordering::Less; "build metadata uppercase is less than lowercase")]
        fn semantic_version(lhs: &str, rhs: &str, expected_order: Ordering) {
            pretty_assertions::assert_eq!(
                SemanticVersion::parse(lhs)
                    .expect("parsing for lhs should not fail")
                    .partial_cmp(&SemanticVersion::parse(rhs).expect("parsing for rhs should not fail"))
                    .expect("comparison should always be an ordering"),
                expected_order,
                "expected '{lhs}' compared to '{rhs}' to be {expected_order:#?}"
            )
        }

        #[test_case("1.2.3", "1.2.3", Ordering::Equal; "equal versions")]
        #[test_case("3.2.1", "1.2.3", Ordering::Greater; "newer lhs")]
        #[test_case("1.2.3", "3.2.1", Ordering::Less; "newer rhs")]
        #[test_case("1.2.3", "1.2.3-rc.1", Ordering::Greater; "stable lhs and prerelease rhs")]
        #[test_case("1.2.3", "1.2.3+ci.1", Ordering::Less; "stable lhs and rhs with build metadata")]
        fn semver_version(
            resource_version_string: &str,
            semver_string: &str,
            expected_order: Ordering,
        ) {
            let version: SemanticVersion = resource_version_string.parse().unwrap();
            let semantic: semver::Version = semver_string.parse().unwrap();

            // Test comparison bidirectionally
            pretty_assertions::assert_eq!(
                version.partial_cmp(&semantic).unwrap(),
                expected_order,
                "expected comparison of {version} and {semantic} to be #{expected_order:#?}"
            );

            let expected_inverted_order = match expected_order {
                Ordering::Equal => Ordering::Equal,
                Ordering::Greater => Ordering::Less,
                Ordering::Less => Ordering::Greater,
            };

            pretty_assertions::assert_eq!(
                semantic.partial_cmp(&version).unwrap(),
                expected_inverted_order,
                "expected comparison of {semantic} and {version} to be #{expected_inverted_order:#?}"
            );
        }

        #[test_case("1.2.3", "1.2.3", Some(Ordering::Equal); "equal version and string")]
        #[test_case("3.2.1", "1.2.3", Some(Ordering::Greater); "newer version and older string")]
        #[test_case("1.2.3", "3.2.1", Some(Ordering::Less); "older version and newer string")]
        #[test_case("1.2.3", "1.2.3-rc.1", Some(Ordering::Greater); "stable version and prerelease string")]
        #[test_case("1.2.3", "1.2.3+ci.1", Some(Ordering::Less); "stable version and string with build metadata")]
        #[test_case("1.2.3", "not a version", None; "stable version and non-version string")]
        fn string(
            resource_version_string: &str,
            string_slice: &str,
            expected_order: Option<Ordering>,
        ) {
            let version: SemanticVersion = resource_version_string.parse().unwrap();
            let string = string_slice.to_string();

            // Test comparison bidirectionally
            pretty_assertions::assert_eq!(
                version.partial_cmp(&string),
                expected_order,
                "expected comparison of {version} and {string} to be #{expected_order:#?}"
            );

            let expected_inverted_order = match expected_order {
                None => None,
                Some(o) => match o {
                    Ordering::Equal => Some(Ordering::Equal),
                    Ordering::Greater => Some(Ordering::Less),
                    Ordering::Less => Some(Ordering::Greater),
                },
            };

            pretty_assertions::assert_eq!(
                string.partial_cmp(&version),
                expected_inverted_order,
                "expected comparison of {string} and {version} to be #{expected_inverted_order:#?}"
            );
        }

        #[test_case("1.2.3", "1.2.3", Some(Ordering::Equal); "equal version and string")]
        #[test_case("3.2.1", "1.2.3", Some(Ordering::Greater); "newer version and older string")]
        #[test_case("1.2.3", "3.2.1", Some(Ordering::Less); "older version and newer string")]
        #[test_case("1.2.3", "1.2.3-rc.1", Some(Ordering::Greater); "stable version and prerelease string")]
        #[test_case("1.2.3", "1.2.3+ci.1", Some(Ordering::Less); "stable version and string with build metadata")]
        #[test_case("1.2.3", "not a version", None; "stable version and non-version string")]
        fn str(
            resource_version_string: &str,
            string_slice: &str,
            expected_order: Option<Ordering>,
        ) {
            let version: SemanticVersion = resource_version_string.parse().unwrap();

            // Test comparison bidirectionally
            pretty_assertions::assert_eq!(
                version.partial_cmp(string_slice),
                expected_order,
                "expected comparison of {version} and {string_slice} to be #{expected_order:#?}"
            );

            let expected_inverted_order = match expected_order {
                None => None,
                Some(o) => match o {
                    Ordering::Equal => Some(Ordering::Equal),
                    Ordering::Greater => Some(Ordering::Less),
                    Ordering::Less => Some(Ordering::Greater),
                },
            };

            pretty_assertions::assert_eq!(
                string_slice.partial_cmp(&version),
                expected_inverted_order,
                "expected comparison of {string_slice} and {version} to be #{expected_inverted_order:#?}"
            );
        }
    }

    mod ord {
        use dsc_lib::types::SemanticVersion;

        #[test]
        fn semantic_version() {
            let v1_0_0 = SemanticVersion::parse("1.0.0").unwrap();
            let v1_2_3 = SemanticVersion::parse("1.2.3").unwrap();
            let v2_0_0 = SemanticVersion::parse("2.0.0").unwrap();
            let v1_2_3_rc_1 = SemanticVersion::parse("1.2.3-rc.1").unwrap();
            let v1_2_3_ci_1 = SemanticVersion::parse("1.2.3+ci.1").unwrap();

            let mut versions = vec![
                v1_0_0.clone(),
                v1_2_3.clone(),
                v2_0_0.clone(),
                v1_2_3_rc_1.clone(),
                v1_2_3_ci_1.clone()
            ];
            versions.sort();

            pretty_assertions::assert_eq!(
                versions,
                vec![
                    v1_0_0,
                    v1_2_3_rc_1,
                    v1_2_3,
                    v1_2_3_ci_1,
                    v2_0_0
                ]
            );
        }
    }
}
