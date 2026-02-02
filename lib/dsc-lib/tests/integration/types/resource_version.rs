// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    use dsc_lib::types::{ResourceVersion, SemanticVersion, SemanticVersionReq};
    use test_case::test_case;

    #[test_case("1.2.3" => matches ResourceVersion::Semantic(_); "for valid semantic version")]
    #[test_case("1.2.3a" => matches ResourceVersion::Arbitrary(_); "for invalid semantic version")]
    #[test_case("2026-01-15" => matches ResourceVersion::Arbitrary(_); "for full ISO8601 date")]
    #[test_case("2026-01" => matches ResourceVersion::Arbitrary(_); "for partial ISO8601 date")]
    #[test_case("arbitrary_string" => matches ResourceVersion::Arbitrary(_); "for arbitrary string")]
    fn new(version_string: &str) -> ResourceVersion {
        ResourceVersion::new(version_string)
    }

    #[test_case("1.2.3" => true; "for valid semantic version")]
    #[test_case("1.2.3a" => false; "for invalid semantic version")]
    #[test_case("2026-01-15" => false; "for full ISO8601 date")]
    #[test_case("2026-01" => false; "for partial ISO8601 date")]
    #[test_case("arbitrary_string" => false; "for arbitrary string")]
    fn is_semver(version_string: &str) -> bool {
        ResourceVersion::new(version_string).is_semver()
    }

    #[test_case("1.2.3" => false; "for valid semantic version")]
    #[test_case("1.2.3a" => true; "for invalid semantic version")]
    #[test_case("2026-01-15" => true; "for full ISO8601 date")]
    #[test_case("2026-01" => true; "for partial ISO8601 date")]
    #[test_case("arbitrary_string" => true; "for arbitrary string")]
    fn is_arbitrary(version_string: &str) -> bool {
        ResourceVersion::new(version_string).is_arbitrary()
    }

    #[test_case(ResourceVersion::new("1.2.3") => matches Some(_); "for valid semantic version")]
    #[test_case(ResourceVersion::new("1.2.3a") => matches None; "for invalid semantic version")]
    #[test_case(ResourceVersion::new("2026-01-15") => matches None; "for full ISO8601 date")]
    #[test_case(ResourceVersion::new("2026-01") => matches None; "for partial ISO8601 date")]
    #[test_case(ResourceVersion::new("arbitrary_string") => matches None; "for arbitrary string")]
    fn as_semver(version: ResourceVersion) -> Option<SemanticVersion> {
        version.as_semver().cloned()
    }

    #[test_case("1.2.3", ">1.0" => true; "semantic version matches gt req")]
    #[test_case("1.2.3", "<=1.2.2" => false; "semantic version not matches le req")]
    #[test_case("1.2.3", "~1" => true; "semantic version matches tilde req")]
    #[test_case("arbitrary", "*" => false; "arbitrary string version never matches")]
    fn matches_semver_req(version_string: &str, requirement_string: &str) -> bool {
        ResourceVersion::new(version_string)
            .matches_semver_req(&SemanticVersionReq::parse(requirement_string).unwrap())
    }
}

#[cfg(test)]
mod schema {
    use std::{ops::Index, sync::LazyLock};

    use dsc_lib::types::ResourceVersion;
    use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    use jsonschema::Validator;
    use regex::Regex;
    use schemars::{schema_for, Schema};
    use serde_json::{json, Value};
    use test_case::test_case;

    static ROOT_SCHEMA: LazyLock<Schema> = LazyLock::new(|| schema_for!(ResourceVersion));
    static SEMVER_VARIANT_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
        (&*ROOT_SCHEMA)
            .get_keyword_as_array("anyOf")
            .unwrap()
            .index(0)
            .as_object()
            .unwrap()
            .clone()
            .into()
    });
    static STRING_VARIANT_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
        (&*ROOT_SCHEMA)
            .get_keyword_as_array("anyOf")
            .unwrap()
            .index(1)
            .as_object()
            .unwrap()
            .clone()
            .into()
    });

    static VALIDATOR: LazyLock<Validator> =
        LazyLock::new(|| Validator::new((&*ROOT_SCHEMA).as_value()).unwrap());

    static KEYWORD_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\w+(\.\w+)+$").expect("pattern is valid"));

    #[test_case("title", &*ROOT_SCHEMA; "title")]
    #[test_case("description", &*ROOT_SCHEMA; "description")]
    #[test_case("markdownDescription", &*ROOT_SCHEMA; "markdownDescription")]
    #[test_case("title", &*SEMVER_VARIANT_SCHEMA; "semver.title")]
    #[test_case("description", &*SEMVER_VARIANT_SCHEMA; "semver.description")]
    #[test_case("markdownDescription", &*SEMVER_VARIANT_SCHEMA; "semver.markdownDescription")]
    #[test_case("title", &*STRING_VARIANT_SCHEMA; "arbitrary.title")]
    #[test_case("description", &*STRING_VARIANT_SCHEMA; "arbitrary.description")]
    #[test_case("markdownDescription", &*STRING_VARIANT_SCHEMA; "arbitrary.markdownDescription")]
    fn has_documentation_keyword(keyword: &str, schema: &Schema) {
        let value = schema
            .get_keyword_as_str(keyword)
            .expect(format!("expected keyword '{keyword}' to be defined").as_str());

        assert!(
            !(&*KEYWORD_PATTERN).is_match(value),
            "Expected keyword '{keyword}' to be defined in translation, but was set to i18n key '{value}'"
        );
    }

    #[test]
    fn semver_subschema_is_reference() {
        assert!(
            (&*SEMVER_VARIANT_SCHEMA).get_keyword_as_string("$ref").is_some_and(|kv| !kv.is_empty())
        )
    }

    #[test_case(&json!("1.2.3") => true ; "valid semantic version string value is valid")]
    #[test_case(&json!("1.2.3a") => true ; "invalid semantic version string value is valid")]
    #[test_case(&json!("2026-01-15") => true ; "iso8601 date full string value is valid")]
    #[test_case(&json!("2026-01") => true ; "iso8601 date year month string value is valid")]
    #[test_case(&json!("arbitrary_string") => true ; "arbitrary string value is valid")]
    #[test_case(&json!(true) => false; "boolean value is invalid")]
    #[test_case(&json!(1) => false; "integer value is invalid")]
    #[test_case(&json!(1.2) => false; "float value is invalid")]
    #[test_case(&json!({"version": "1.2.3"}) => false; "object value is invalid")]
    #[test_case(&json!(["1.2.3"]) => false; "array value is invalid")]
    #[test_case(&serde_json::Value::Null => false; "null value is invalid")]
    fn validation(input_json: &Value) -> bool {
        (&*VALIDATOR).validate(input_json).is_ok()
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::types::ResourceVersion;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case("1.2.3"; "valid semantic version")]
    #[test_case("1.2.3a"; "invalid semantic version")]
    #[test_case("2026-01-15"; "ISO8601 date full")]
    #[test_case("2026-01"; "ISO8601 date year and month only")]
    #[test_case("arbitrary_string"; "arbitrary string")]
    fn serializing_resource_version_to_string(version_string: &str) {
        let actual = serde_json::to_string(&ResourceVersion::new(version_string))
            .expect("serialization should never fail");
        let expected = format!(r#""{version_string}""#);

        pretty_assertions::assert_eq!(actual, expected);
    }

    #[test_case("1.2.3"; "valid semantic version")]
    #[test_case("1.2.3a"; "invalid semantic version")]
    #[test_case("2026-01-15"; "ISO8601 date full")]
    #[test_case("2026-01"; "ISO8601 date year and month only")]
    #[test_case("arbitrary_string"; "arbitrary string")]
    fn serializing_to_json_value_returns_string(version_string: &str) {
        let expected = Value::String(version_string.to_string());
        let actual = serde_json::to_value(&ResourceVersion::new(version_string))
            .expect("serialization should never fail");

        pretty_assertions::assert_eq!(actual, expected);
    }


    #[test_case(json!(true); "boolean value fails")]
    #[test_case(json!(1); "integer value fails")]
    #[test_case(json!(1.2); "float value fails")]
    #[test_case(json!({"version": "1.2.3"}); "object value fails")]
    #[test_case(json!(["1.2.3"]); "array value fails")]
    #[test_case(serde_json::Value::Null; "null value fails")]
    fn deserializing_invalid(input_value: Value) {
        serde_json::from_value::<ResourceVersion>(input_value)
            .expect_err("json value '{input_value}' should be invalid");
    }

    #[test_case(json!("1.2.3") => matches ResourceVersion::Semantic(_); "valid semantic version string value succeeds")]
    #[test_case(json!("1.2.3a") => matches ResourceVersion::Arbitrary(_) ; "invalid semantic version string value succeeds")]
    #[test_case(json!("2026-01-15") => matches ResourceVersion::Arbitrary(_) ; "iso8601 date full string value succeeds")]
    #[test_case(json!("2026-01") => matches ResourceVersion::Arbitrary(_) ; "iso8601 date year month string value succeeds")]
    #[test_case(json!("arbitrary_string") => matches ResourceVersion::Arbitrary(_) ; "arbitrary string value succeeds")]
    fn deserializing_valid(input_value: Value) -> ResourceVersion {
        serde_json::from_value::<ResourceVersion>(input_value)
            .expect("deserialization for '{input_value}' should never fail")
    }
}

#[cfg(test)]
mod traits {
    #[cfg(test)]
    mod default {
        use dsc_lib::types::ResourceVersion;

        #[test]
        fn default() {
            pretty_assertions::assert_eq!(
                ResourceVersion::default(),
                ResourceVersion::new("0.0.0")
            );
        }
    }

    #[cfg(test)]
    mod display {
        use dsc_lib::types::ResourceVersion;
        use test_case::test_case;

        #[test_case("1.2.3"; "valid semantic version")]
        #[test_case("1.2.3a"; "invalid semantic version")]
        #[test_case("2026-01-15"; "ISO8601 date full")]
        #[test_case("2026-01"; "ISO8601 date year and month only")]
        #[test_case("arbitrary_string"; "arbitrary string")]
        fn format(version_string: &str) {
            pretty_assertions::assert_eq!(
                format!("version: {}", ResourceVersion::new(version_string)),
                format!("version: {version_string}")
            )
        }

        #[test_case("1.2.3"; "valid semantic version")]
        #[test_case("1.2.3a"; "invalid semantic version")]
        #[test_case("2026-01-15"; "ISO8601 date full")]
        #[test_case("2026-01"; "ISO8601 date year and month only")]
        #[test_case("arbitrary_string"; "arbitrary string")]
        fn to_string(version_string: &str) {
            pretty_assertions::assert_eq!(
                ResourceVersion::new(version_string).to_string(),
                version_string.to_string()
            )
        }
    }

    #[cfg(test)]
    mod from_str {
        use dsc_lib::types::ResourceVersion;
        use test_case::test_case;

        #[test_case("1.2.3" => ResourceVersion::new("1.2.3"); "valid semantic version")]
        #[test_case("1.2.3a" => ResourceVersion::new("1.2.3a"); "invalid semantic version")]
        #[test_case("2026-01-15" => ResourceVersion::new("2026-01-15"); "ISO8601 date full")]
        #[test_case("2026-01" => ResourceVersion::new("2026-01"); "ISO8601 date year and month only")]
        #[test_case("arbitrary_string" => ResourceVersion::new("arbitrary_string"); "arbitrary string")]
        fn parse(input: &str) -> ResourceVersion {
            input.parse().expect("parse should be infallible")
        }
    }

    #[cfg(test)]
    mod from {
        use dsc_lib::types::{ResourceVersion, SemanticVersion};
        use test_case::test_case;

        #[test]
        fn semantic_version() {
            let semantic_version = SemanticVersion::parse("1.2.3").unwrap();
            match ResourceVersion::from(semantic_version.clone()) {
                ResourceVersion::Semantic(v) => pretty_assertions::assert_eq!(v, semantic_version),
                ResourceVersion::Arbitrary(_) => {
                    panic!("should never fail to convert as Semantic version")
                }
            }
        }

        #[test_case("1.2.3" => matches ResourceVersion::Semantic(_); "valid semantic version")]
        #[test_case("1.2.3a" => matches ResourceVersion::Arbitrary(_); "invalid semantic version")]
        #[test_case("2026-01-15" => matches ResourceVersion::Arbitrary(_); "ISO8601 date full")]
        #[test_case("2026-01" => matches ResourceVersion::Arbitrary(_); "ISO8601 date year and month only")]
        #[test_case("arbitrary_string" => matches ResourceVersion::Arbitrary(_); "arbitrary string")]
        fn string(version_string: &str) -> ResourceVersion {
            ResourceVersion::from(version_string.to_string())
        }
    }

    // While technically we implemented the traits as `From<TypeVersion> for <T>`, it's easier to
    // reason about what we're converting _into_ - otherwise the functions would have names like
    // `resource_version_for_semver_version`. When you implement `From`, you automatically implement
    // `Into` for the reversing of the type pair.
    #[cfg(test)]
    mod into {
        use dsc_lib::types::ResourceVersion;
        use test_case::test_case;

        #[test_case("1.2.3"; "semantic version")]
        #[test_case("arbitrary_version"; "arbitrary string version")]
        fn string(version_string: &str) {
            let actual: String = ResourceVersion::new(version_string).into();
            let expected = version_string.to_string();

            pretty_assertions::assert_eq!(actual, expected)
        }
    }

    #[cfg(test)]
    mod try_into {
        use dsc_lib::{dscerror::DscError, types::{ResourceVersion, SemanticVersion}};
        use test_case::test_case;

        #[test_case("1.2.3" => matches Ok(_); "valid semantic version converts")]
        #[test_case("1.2.3a" => matches Err(_); "invalid semantic version fails")]
        #[test_case("2026-01-15" => matches Err(_); "ISO8601 date full fails")]
        #[test_case("2026-01" => matches Err(_); "ISO8601 date year and month only fails")]
        #[test_case("arbitrary_string" => matches Err(_); "arbitrary string fails")]
        fn semantic_version(version_string: &str) -> Result<SemanticVersion, DscError> {
            TryInto::<SemanticVersion>::try_into(ResourceVersion::new(version_string))
        }
    }

    #[cfg(test)]
    mod partial_eq {
        use dsc_lib::types::{ResourceVersion, SemanticVersion};
        use test_case::test_case;

        #[test_case("1.2.3", "1.2.3", true; "equal semantic versions")]
        #[test_case("1.2.3", "3.2.1", false; "unequal semantic versions")]
        #[test_case("Arbitrary", "Arbitrary", true; "identical string versions")]
        #[test_case("Arbitrary", "arbitrary", false; "differently cased string versions")]
        #[test_case("foo", "bar", false; "unequal string versions")]
        fn resource_version(lhs: &str, rhs: &str, should_be_equal: bool) {
            if should_be_equal {
                pretty_assertions::assert_eq!(ResourceVersion::new(lhs), ResourceVersion::new(rhs))
            } else {
                pretty_assertions::assert_ne!(ResourceVersion::new(lhs), ResourceVersion::new(rhs))
            }
        }

        #[test_case("1.2.3", "1.2.3", true; "equal semantic versions")]
        #[test_case("1.2.3", "3.2.1", false; "unequal semantic versions")]
        #[test_case("arbitrary_string", "3.2.1", false; "arbitrary string with semantic version")]
        fn semantic_version(
            resource_version_string: &str,
            semantic_version_string: &str,
            should_be_equal: bool,
        ) {
            let version: ResourceVersion = resource_version_string.parse().unwrap();
            let semantic: SemanticVersion = semantic_version_string.parse().unwrap();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                version == semantic,
                should_be_equal,
                "expected comparison of {version} and {semantic} to be #{should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                semantic == version,
                should_be_equal,
                "expected comparison of {semantic} and {version} to be #{should_be_equal}"
            );
        }

        #[test_case("1.2.3", "1.2.3", true; "semantic version and equivalent string")]
        #[test_case("1.2.3", "3.2.1", false; "semantic version and differing string")]
        #[test_case("Arbitrary", "Arbitrary", true; "arbitrary string version and identical string")]
        #[test_case("Arbitrary", "arbitrary", false; "arbitrary string version and string with differing case")]
        #[test_case("foo", "bar", false; "arbitrary string version and different string")]
        fn str(resource_version_string: &str, string_slice: &str, should_be_equal: bool) {
            let version: ResourceVersion = resource_version_string.parse().unwrap();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                version == string_slice,
                should_be_equal,
                "expected comparison of {version} and {string_slice} to be #{should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string_slice == version,
                should_be_equal,
                "expected comparison of {string_slice} and {version} to be #{should_be_equal}"
            );
        }

        #[test_case("1.2.3", "1.2.3", true; "semantic version and equivalent string")]
        #[test_case("1.2.3", "3.2.1", false; "semantic version and differing string")]
        #[test_case("Arbitrary", "Arbitrary", true; "arbitrary string version and identical string")]
        #[test_case("Arbitrary", "arbitrary", false; "arbitrary string version and string with differing case")]
        #[test_case("foo", "bar", false; "arbitrary string version and different string")]
        fn string(resource_version_string: &str, string_slice: &str, should_be_equal: bool) {
            let version: ResourceVersion = resource_version_string.parse().unwrap();
            let string = string_slice.to_string();
            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                version == string,
                should_be_equal,
                "expected comparison of {version} and {string} to be #{should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string == version,
                should_be_equal,
                "expected comparison of {string} and {version} to be #{should_be_equal}"
            );
        }
    }

    #[cfg(test)]
    mod partial_ord {
        use std::cmp::Ordering;

        use dsc_lib::types::{ResourceVersion, SemanticVersion};
        use test_case::test_case;

        #[test_case("1.2.3", "1.2.3", Ordering::Equal; "equal semantic versions")]
        #[test_case("3.2.1", "1.2.3", Ordering::Greater; "semantic versions with newer lhs")]
        #[test_case("1.2.3", "3.2.1", Ordering::Less; "semantic versions with newer rhs")]
        #[test_case("1.2.3", "arbitrary", Ordering::Greater; "semantic version to string version")]
        #[test_case("arbitrary", "1.2.3", Ordering::Less; "string version to semantic version")]
        #[test_case("arbitrary", "arbitrary", Ordering::Equal; "string version to same string version")]
        #[test_case("arbitrary", "ARBITRARY", Ordering::Greater; "lowercased string version to uppercased string version")]
        #[test_case("foo", "bar", Ordering::Greater; "string version to earlier alphabetic string version")]
        #[test_case("a", "b", Ordering::Less; "string version to later alphabetic string version")]
        #[test_case("2026-01-15", "2026-01-15", Ordering::Equal; "full date string version to same string version")]
        #[test_case("2026-01", "2026-01", Ordering::Equal; "partial date string version to same string version")]
        #[test_case("2026-01-15", "2026-02-15", Ordering::Less; "full date string version to later full date")]
        #[test_case("2026-01-15", "2026-02", Ordering::Less; "full date string version to later partial date")]
        #[test_case("2026-01", "2026-02-15", Ordering::Less; "partial date string version to later full date")]
        #[test_case("2026-01", "2026-02", Ordering::Less; "partial date string version to later partial date")]
        fn resource_version(lhs: &str, rhs: &str, expected_order: Ordering) {
            pretty_assertions::assert_eq!(
                ResourceVersion::new(lhs)
                    .partial_cmp(&ResourceVersion::new(rhs))
                    .unwrap(),
                expected_order,
                "expected '{lhs}' compared to '{rhs}' to be {expected_order:#?}"
            )
        }

        #[test_case("1.2.3", "1.2.3", Ordering::Equal; "equal semantic versions")]
        #[test_case("3.2.1", "1.2.3", Ordering::Greater; "semantic versions with newer lhs")]
        #[test_case("1.2.3", "3.2.1", Ordering::Less; "semantic versions with newer rhs")]
        #[test_case("arbitrary", "1.2.3", Ordering::Less; "string version to semantic version")]
        fn semantic_version(
            resource_version_string: &str,
            semantic_version_string: &str,
            expected_order: Ordering,
        ) {
            let version: ResourceVersion = resource_version_string.parse().unwrap();
            let semantic: SemanticVersion = semantic_version_string.parse().unwrap();

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

        #[test_case("1.2.3", "1.2.3", Ordering::Equal; "equal semantic versions")]
        #[test_case("3.2.1", "1.2.3", Ordering::Greater; "semantic versions with newer lhs")]
        #[test_case("1.2.3", "3.2.1", Ordering::Less; "semantic versions with newer rhs")]
        #[test_case("1.2.3", "arbitrary", Ordering::Greater; "semantic version to string version")]
        #[test_case("arbitrary", "1.2.3", Ordering::Less; "string version to semantic version")]
        #[test_case("arbitrary", "arbitrary", Ordering::Equal; "string version to same string version")]
        #[test_case("arbitrary", "ARBITRARY", Ordering::Greater; "lowercased string version to uppercased string version")]
        #[test_case("foo", "bar", Ordering::Greater; "string version to earlier alphabetic string version")]
        #[test_case("a", "b", Ordering::Less; "string version to later alphabetic string version")]
        #[test_case("2026-01-15", "2026-01-15", Ordering::Equal; "full date string version to same string version")]
        #[test_case("2026-01", "2026-01", Ordering::Equal; "partial date string version to same string version")]
        #[test_case("2026-01-15", "2026-02-15", Ordering::Less; "full date string version to later full date")]
        #[test_case("2026-01-15", "2026-02", Ordering::Less; "full date string version to later partial date")]
        #[test_case("2026-01", "2026-02-15", Ordering::Less; "partial date string version to later full date")]
        #[test_case("2026-01", "2026-02", Ordering::Less; "partial date string version to later partial date")]
        fn string(resource_version_string: &str, string_slice: &str, expected_order: Ordering) {
            let version: ResourceVersion = resource_version_string.parse().unwrap();
            let string = string_slice.to_string();

            // Test comparison bidirectionally
            pretty_assertions::assert_eq!(
                version.partial_cmp(&string).unwrap(),
                expected_order,
                "expected comparison of {version} and {string} to be #{expected_order:#?}"
            );

            let expected_inverted_order = match expected_order {
                Ordering::Equal => Ordering::Equal,
                Ordering::Greater => Ordering::Less,
                Ordering::Less => Ordering::Greater,
            };

            pretty_assertions::assert_eq!(
                string.partial_cmp(&version).unwrap(),
                expected_inverted_order,
                "expected comparison of {string} and {version} to be #{expected_inverted_order:#?}"
            );
        }
    }
}
