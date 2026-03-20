// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    use dsc_lib::types::{
        DateVersion,
        ResourceVersion,
        ResourceVersionError,
        SemanticVersion,
        SemanticVersionReq
    };
    use test_case::test_case;

    #[test_case("1.2.3" => matches Ok(_); "valid semantic version parses")]
    #[test_case("1.2.3a" => matches Err(_); "invalid semantic version fails")]
    #[test_case("2026-01-15" => matches Ok(_); "full ISO8601 date parses")]
    #[test_case("2026-01-15-rc" => matches Ok(_); "full ISO8601 date with preview segment parses")]
    #[test_case("2026-01" => matches Err(_); "partial ISO8601 date fails")]
    #[test_case("arbitrary_string" => matches Err(_); "arbitrary string fails")]
    fn parse(version_string: &str) -> Result<ResourceVersion, ResourceVersionError> {
        ResourceVersion::parse(version_string)
    }

    #[test_case("1.2.3" => true; "semantic version is semantic")]
    #[test_case("2026-01-15" => false; "stable date version is not semantic")]
    #[test_case("2026-01-15-rc" => false; "preview date version is not semantic")]
    fn is_semver(version_string: &str) -> bool {
        ResourceVersion::parse(version_string).unwrap().is_semver()
    }

    #[test_case("1.2.3" => false; "semantic version is not date")]
    #[test_case("2026-01-15" => true; "stable date version is date")]
    #[test_case("2026-01-15-rc" => true; "preview date version is date")]
    fn is_date_version(version_string: &str) -> bool {
        ResourceVersion::parse(version_string).unwrap().is_date_version()
    }

    #[test_case("1.2.3" => matches Some(_); "semantic version returns some")]
    #[test_case("2026-01-15" => matches None; "stable date version returns none")]
    #[test_case("2026-01-15-rc" => matches None; "preview date version returns none")]
    fn as_semver(version: &str) -> Option<SemanticVersion> {
        ResourceVersion::parse(version).unwrap().as_semver().cloned()
    }

    #[test_case("1.2.3" => matches None; "semantic version returns none")]
    #[test_case("2026-01-15" => matches Some(_); "stable date version returns some")]
    #[test_case("2026-01-15-rc" => matches Some(_); "preview date version returns some")]
    fn as_date_version(version: &str) -> Option<DateVersion> {
        ResourceVersion::parse(version).unwrap().as_date_version().cloned()
    }

    #[test_case("1.2.3", ">1.0" => true; "semantic version matches gt req")]
    #[test_case("1.2.3", "<=1.2.2" => false; "semantic version not matches le req")]
    #[test_case("1.2.3", "~1" => true; "semantic version matches tilde req")]
    #[test_case("2026-01-15", "^1" => false; "date version never matches")]
    fn matches_semver_req(version_string: &str, requirement_string: &str) -> bool {
        ResourceVersion::parse(version_string)
            .unwrap()
            .matches_semver_req(&SemanticVersionReq::parse(requirement_string).unwrap())
    }

    #[test_case("2026-01-15", "2026-01-15" => true; "stable date version matches identical date req")]
    #[test_case("2026-01-15", "2026-02-15" => false; "stable date version does not match different stable date req")]
    #[test_case("2026-01-15", "2026-01-15-rc" => false; "stable date version does not match preview date req")]
    #[test_case("2026-01-15-rc", "2026-01-15-rc" => true; "preview date version matches identical date req")]
    #[test_case("2026-01-15-rc", "2026-02-15-rc" => false; "preview date version does not match different preview date req")]
    #[test_case("2026-01-15-rc", "2026-02-15-preview" => false; "preview date version does not match preview date req with different prerelease segment")]
    #[test_case("2026-01-15-rc", "2026-01-15-RC" => false; "preview date version does not match preview date req with different casing")]
    #[test_case("1.2.3", "2026-01-15" => false; "semantic version does not match date req")]
    fn matches_date_version(version_string: &str, requirement_string: &str) -> bool {
        ResourceVersion::parse(version_string)
            .unwrap()
            .matches_date_req(&DateVersion::parse(requirement_string).unwrap())
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
    static DATE_VARIANT_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
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
    #[test_case("title", &*DATE_VARIANT_SCHEMA; "date.title")]
    #[test_case("description", &*DATE_VARIANT_SCHEMA; "date.description")]
    #[test_case("markdownDescription", &*DATE_VARIANT_SCHEMA; "date.markdownDescription")]
    #[test_case("deprecationMessage", &*DATE_VARIANT_SCHEMA; "date.deprecationMessage")]
    fn has_documentation_keyword(keyword: &str, schema: &Schema) {
        let value = schema
            .get_keyword_as_str(keyword)
            .expect(&format!("expected keyword '{keyword}' to be defined"));

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

    #[test]
    fn date_version_subschema_is_reference() {
        assert!(
            (&*DATE_VARIANT_SCHEMA).get_keyword_as_string("$ref").is_some_and(|kv| !kv.is_empty())
        )
    }

    #[test_case(&json!("1.2.3") => true ; "valid semantic version string value is valid")]
    #[test_case(&json!("1.2.a") => false ; "invalid semantic version string value is invalid")]
    #[test_case(&json!("2026-01-15") => true ; "iso8601 full date string value is valid")]
    #[test_case(&json!("2026-01-15-rc") => true ; "iso8601 full date with prerelease segment string value is valid")]
    #[test_case(&json!("2026-01") => false ; "iso8601 partial date is invalid")]
    #[test_case(&json!("arbitrary_string") => false ; "arbitrary string value is invalid")]
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
    use dsc_lib::types::{ResourceVersion, ResourceVersion::*};
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case("1.2.3"; "semantic version")]
    #[test_case("2026-01-15"; "stable date version")]
    #[test_case("2026-01-15-rc"; "preview date version")]
    fn serializing_resource_version_to_string(version_string: &str) {
        let actual = serde_json::to_string(&ResourceVersion::parse(version_string).unwrap())
            .expect("serialization should never fail");
        let expected = format!(r#""{version_string}""#);

        pretty_assertions::assert_eq!(actual, expected);
    }

    #[test_case("1.2.3"; "semantic version")]
    #[test_case("2026-01-15"; "stable date version")]
    #[test_case("2026-01-15-rc"; "preview date version")]
    fn serializing_to_json_value_returns_string(version_string: &str) {
        let expected = Value::String(version_string.to_string());
        let actual = serde_json::to_value(&ResourceVersion::parse(version_string).unwrap())
            .expect("serialization should never fail");

        pretty_assertions::assert_eq!(actual, expected);
    }


    #[test_case(json!("1.2.c"); "invalid semantic version fails")]
    #[test_case(json!("2026-02-29"); "invalid date version fails")]
    #[test_case(json!(true); "boolean value fails")]
    #[test_case(json!(1); "integer value fails")]
    #[test_case(json!(1.2); "float value fails")]
    #[test_case(json!({"version": "1.2.3"}); "object value fails")]
    #[test_case(json!(["1.2.3"]); "array value fails")]
    #[test_case(serde_json::Value::Null; "null value fails")]
    fn deserializing_invalid(input_value: Value) {
        serde_json::from_value::<ResourceVersion>(input_value.clone())
            .expect_err(&format!("json value '{input_value}' should be invalid"));
    }

    #[test_case(json!("1.2.3") => matches Semantic(_); "valid semantic version string value returns semantic version")]
    #[test_case(json!("2026-01-15") => matches Date(_) ; "iso8601 date string value returns date version")]
    #[test_case(json!("2026-01-15-rc") => matches Date(_) ; "iso8601 date with preview segment string value returns date version")]
    fn deserializing_valid(input_value: Value) -> ResourceVersion {
        serde_json::from_value::<ResourceVersion>(input_value)
            .expect("deserializing shouldn't fail")
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
                ResourceVersion::parse("0.0.0").unwrap()
            );
        }
    }

    #[cfg(test)]
    mod display {
        use dsc_lib::types::ResourceVersion;
        use test_case::test_case;

        #[test_case("1.2.3"; "semantic version")]
        #[test_case("2026-01-15"; "stable date version")]
        #[test_case("2026-01-15-rc"; "preview date version")]
        fn format(version_string: &str) {
            pretty_assertions::assert_eq!(
                format!("version: {}", ResourceVersion::parse(version_string).unwrap()),
                format!("version: {version_string}")
            )
        }

        #[test_case("1.2.3"; "semantic version")]
        #[test_case("2026-01-15"; "stable date version")]
        #[test_case("2026-01-15-rc"; "preview date version")]
        fn to_string(version_string: &str) {
            pretty_assertions::assert_eq!(
                ResourceVersion::parse(version_string).unwrap().to_string(),
                version_string.to_string()
            )
        }
    }

    #[cfg(test)]
    mod from_str {
        use dsc_lib::types::{ResourceVersion, ResourceVersionError};
        use test_case::test_case;

        #[test_case("1.2.3" => matches Ok(_); "valid semantic version parses")]
        #[test_case("1.2.3a" => matches Err(_); "invalid semantic version fails")]
        #[test_case("2026-01-15" => matches Ok(_); "ISO8601 date parses")]
        #[test_case("2026-01-15-rc" => matches Ok(_); "ISO8601 date with preview segment parses")]
        #[test_case("2026-01" => matches Err(_); "ISO8601 date year and month only fails")]
        #[test_case("arbitrary_string" => matches Err(_); "arbitrary string fails")]
        fn parse(input: &str) -> Result<ResourceVersion, ResourceVersionError> {
            input.parse()
        }
    }

    #[cfg(test)]
    mod from {
        use dsc_lib::types::{DateVersion, ResourceVersion, SemanticVersion};

        #[test]
        fn semantic_version() {
            let semantic_version = SemanticVersion::parse("1.2.3").unwrap();
            match ResourceVersion::from(semantic_version.clone()) {
                ResourceVersion::Semantic(v) => pretty_assertions::assert_eq!(v, semantic_version),
                _ => {
                    panic!("should never fail to convert as Semantic version")
                }
            }
        }

        #[test]
        fn date_version() {
            let date_version = DateVersion::parse("2026-01-15").unwrap();
            match ResourceVersion::from(date_version.clone()) {
                ResourceVersion::Date(v) => pretty_assertions::assert_eq!(v, date_version),
                _ => {
                    panic!("should never fail to convert as date version")
                }
            }
        }
    }

    #[cfg(test)]
    mod try_from {
        use dsc_lib::types::{ResourceVersion, ResourceVersionError};
        use test_case::test_case;

        #[test_case("1.2.3" => matches Ok(_); "valid semantic version converts")]
        #[test_case("1.2.3a" => matches Err(_); "invalid semantic version fails")]
        #[test_case("2026-01-15" => matches Ok(_); "valid ISO8601 date converts")]
        #[test_case("2026-01-15-rc" => matches Ok(_); "valid ISO8601 date with preview segment converts")]
        #[test_case("2026-01" => matches Err(_); "partial ISO8601 date fails")]
        #[test_case("2026-02-29" => matches Err(_); "invalid ISO8601 date fails")]
        #[test_case("arbitrary_string" => matches Err(_); "arbitrary string fails")]
        fn string(version_string: &str) -> Result<ResourceVersion, ResourceVersionError> {
            ResourceVersion::try_from(version_string.to_string())
        }

        #[test_case("1.2.3" => matches Ok(_); "valid semantic version converts")]
        #[test_case("1.2.3a" => matches Err(_); "invalid semantic version fails")]
        #[test_case("2026-01-15" => matches Ok(_); "valid ISO8601 date converts")]
        #[test_case("2026-01-15-rc" => matches Ok(_); "valid ISO8601 date with preview segment converts")]
        #[test_case("2026-01" => matches Err(_); "partial ISO8601 date fails")]
        #[test_case("2026-02-29" => matches Err(_); "invalid ISO8601 date fails")]
        #[test_case("arbitrary_string" => matches Err(_); "arbitrary string fails")]
        fn str(version_string: &str) -> Result<ResourceVersion, ResourceVersionError> {
            ResourceVersion::try_from(version_string)
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
        #[test_case("2026-01-15"; "stable date version")]
        #[test_case("2026-01-15-rc"; "preview date version")]
        fn string(version_string: &str) {
            let actual: String = ResourceVersion::parse(version_string).unwrap().into();
            let expected = version_string.to_string();

            pretty_assertions::assert_eq!(actual, expected)
        }
    }

    #[cfg(test)]
    mod try_into {
        use dsc_lib::types::{DateVersion, ResourceVersion, ResourceVersionError, SemanticVersion};
        use test_case::test_case;

        #[test_case("1.2.3" => matches Ok(_); "semantic version converts")]
        #[test_case("2026-01-15" => matches Err(_); "stable date version fails")]
        #[test_case("2026-01-15-rc" => matches Err(_); "preview date version fails")]
        fn semantic_version(version_string: &str) -> Result<SemanticVersion, ResourceVersionError> {
            TryInto::<SemanticVersion>::try_into(ResourceVersion::parse(version_string).unwrap())
        }

        #[test_case("1.2.3" => matches Err(_); "semantic version fails")]
        #[test_case("2026-01-15" => matches Ok(_); "stable date version converts")]
        #[test_case("2026-01-15-rc" => matches Ok(_); "preview date version converts")]
        fn date_version(version_string: &str) -> Result<DateVersion, ResourceVersionError> {
            TryInto::<DateVersion>::try_into(ResourceVersion::parse(version_string).unwrap())
        }
    }

    #[cfg(test)]
    mod partial_eq {
        use dsc_lib::types::{DateVersion, ResourceVersion, SemanticVersion};
        use test_case::test_case;

        #[test_case("1.2.3", "1.2.3", true; "equal semantic versions")]
        #[test_case("1.2.3", "3.2.1", false; "unequal semantic versions")]
        #[test_case("2026-01-15", "2026-01-15", true; "identical stable date versions")]
        #[test_case("2026-01-15", "2026-03-15", false; "different stable date versions")]
        #[test_case("2026-01-15-rc", "2026-01-15-rc", true; "identical preview date versions")]
        #[test_case("2026-01-15-rc", "2026-01-15-preview", false; "different preview date versions")]
        #[test_case("2026-01-15-rc", "2026-01-15-RC", false; "differently cased preview date versions")]
        fn resource_version(lhs: &str, rhs: &str, should_be_equal: bool) {
            if should_be_equal {
                pretty_assertions::assert_eq!(
                    ResourceVersion::parse(lhs).unwrap(),
                    ResourceVersion::parse(rhs).unwrap()
                )
            } else {
                pretty_assertions::assert_ne!(
                    ResourceVersion::parse(lhs).unwrap(),
                    ResourceVersion::parse(rhs).unwrap()
                )
            }
        }

        #[test_case("1.2.3", "1.2.3", true; "equal semantic versions")]
        #[test_case("1.2.3", "3.2.1", false; "unequal semantic versions")]
        #[test_case("2026-01-15", "3.2.1", false; "date version with semantic version")]
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
        
        #[test_case("2026-01-15", "2026-01-15", true; "identical stable date versions")]
        #[test_case("2026-01-15", "2026-03-15", false; "different stable date versions")]
        #[test_case("2026-01-15-rc", "2026-01-15-rc", true; "identical preview date versions")]
        #[test_case("2026-01-15-rc", "2026-01-15-preview", false; "different preview date versions")]
        #[test_case("2026-01-15-rc", "2026-01-15-RC", false; "differently cased preview date versions")]
        #[test_case("1.2.3", "2026-01-15-RC", false; "semantic version with date version")]
        fn date_version(
            resource_version_string: &str,
            date_version_string: &str,
            should_be_equal: bool,
        ) {
            let version: ResourceVersion = resource_version_string.parse().unwrap();
            let date: DateVersion = date_version_string.parse().unwrap();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                version == date,
                should_be_equal,
                "expected comparison of {version} and {date} to be #{should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                date == version,
                should_be_equal,
                "expected comparison of {date} and {version} to be #{should_be_equal}"
            );
        }

        #[test_case("1.2.3", "1.2.3", true; "equal semantic versions")]
        #[test_case("1.2.3", "3.2.1", false; "unequal semantic versions")]
        #[test_case("2026-01-15", "2026-01-15", true; "identical stable date versions")]
        #[test_case("2026-01-15", "2026-03-15", false; "different stable date versions")]
        #[test_case("2026-01-15-rc", "2026-01-15-rc", true; "identical preview date versions")]
        #[test_case("2026-01-15-rc", "2026-01-15-preview", false; "different preview date versions")]
        #[test_case("2026-01-15-rc", "2026-01-15-RC", false; "differently cased preview date versions")]
        #[test_case("1.2.3", "arbitrary", false; "semantic version and arbitrary string")]
        #[test_case("2026-01-15", "arbitrary", false; "date version and arbitrary string")]
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

        #[test_case("1.2.3", "1.2.3", true; "equal semantic versions")]
        #[test_case("1.2.3", "3.2.1", false; "unequal semantic versions")]
        #[test_case("2026-01-15", "2026-01-15", true; "identical stable date versions")]
        #[test_case("2026-01-15", "2026-03-15", false; "different stable date versions")]
        #[test_case("2026-01-15-rc", "2026-01-15-rc", true; "identical preview date versions")]
        #[test_case("2026-01-15-rc", "2026-01-15-preview", false; "different preview date versions")]
        #[test_case("2026-01-15-rc", "2026-01-15-RC", false; "differently cased preview date versions")]
        #[test_case("1.2.3", "arbitrary", false; "semantic version and arbitrary string")]
        #[test_case("2026-01-15", "arbitrary", false; "date version and arbitrary string")]
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

        use dsc_lib::types::{DateVersion, ResourceVersion, SemanticVersion};
        use test_case::test_case;

        #[test_case("1.2.3", "1.2.3", Ordering::Equal; "equal semantic versions")]
        #[test_case("3.2.1", "1.2.3", Ordering::Greater; "semantic versions with newer lhs")]
        #[test_case("1.2.3", "3.2.1", Ordering::Less; "semantic versions with newer rhs")]
        #[test_case("2026-01-15", "2026-01-15", Ordering::Equal; "identical stable date versions")]
        #[test_case("2026-02-15", "2026-01-15", Ordering::Greater; "stable date versions with newer lhs")]
        #[test_case("2026-01-15", "2026-02-15", Ordering::Less; "stable date versions with newer rhs")]
        #[test_case("2026-01-15-rc", "2026-01-15-rc", Ordering::Equal; "identical preview date versions")]
        #[test_case("2026-02-15-rc", "2026-01-15-rc", Ordering::Greater; "preview date versions with newer lhs")]
        #[test_case("2026-01-15-rc", "2026-02-15-rc", Ordering::Less; "preview date versions with newer rhs")]
        #[test_case("2026-01-15-alpha", "2026-01-15-bravo", Ordering::Less; "preview date versions with lexicographically later rhs preview segment")]
        #[test_case("2026-01-15-bravo", "2026-01-15-alpha", Ordering::Greater; "preview date versions with lexicographically earlier rhs preview segment")]
        #[test_case("1.2.3", "2026-01-15", Ordering::Greater; "semantic version to date version")]
        #[test_case("2026-01-15", "1.2.3", Ordering::Less; "date version to semantic version")]
        fn resource_version(lhs: &str, rhs: &str, expected_order: Ordering) {
            pretty_assertions::assert_eq!(
                ResourceVersion::parse(lhs)
                    .unwrap()
                    .partial_cmp(&ResourceVersion::parse(rhs).unwrap())
                    .unwrap(),
                expected_order,
                "expected '{lhs}' compared to '{rhs}' to be {expected_order:#?}"
            )
        }

        #[test_case("1.2.3", "1.2.3", Ordering::Equal; "equal semantic versions")]
        #[test_case("3.2.1", "1.2.3", Ordering::Greater; "semantic versions with newer lhs")]
        #[test_case("1.2.3", "3.2.1", Ordering::Less; "semantic versions with newer rhs")]
        #[test_case("2026-01-15", "1.2.3", Ordering::Less; "date version to semantic version")]
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
        
        #[test_case("2026-01-15", "2026-01-15", Ordering::Equal; "identical stable date versions")]
        #[test_case("2026-02-15", "2026-01-15", Ordering::Greater; "stable date versions with newer lhs")]
        #[test_case("2026-01-15", "2026-02-15", Ordering::Less; "stable date versions with newer rhs")]
        #[test_case("2026-01-15-rc", "2026-01-15-rc", Ordering::Equal; "identical preview date versions")]
        #[test_case("2026-02-15-rc", "2026-01-15-rc", Ordering::Greater; "preview date versions with newer lhs")]
        #[test_case("2026-01-15-rc", "2026-02-15-rc", Ordering::Less; "preview date versions with newer rhs")]
        #[test_case("2026-01-15-alpha", "2026-01-15-bravo", Ordering::Less; "preview date versions with lexicographically later rhs preview segment")]
        #[test_case("2026-01-15-bravo", "2026-01-15-alpha", Ordering::Greater; "preview date versions with lexicographically earlier rhs preview segment")]
        #[test_case("1.2.3", "2026-01-15", Ordering::Greater; "semantic version to date version")]
        fn date_version(
            resource_version_string: &str,
            date_version_string: &str,
            expected_order: Ordering,
        ) {
            let version: ResourceVersion = resource_version_string.parse().unwrap();
            let date: DateVersion = date_version_string.parse().unwrap();

            // Test comparison bidirectionally
            pretty_assertions::assert_eq!(
                version.partial_cmp(&date).unwrap(),
                expected_order,
                "expected comparison of {version} and {date} to be #{expected_order:#?}"
            );

            let expected_inverted_order = match expected_order {
                Ordering::Equal => Ordering::Equal,
                Ordering::Greater => Ordering::Less,
                Ordering::Less => Ordering::Greater,
            };

            pretty_assertions::assert_eq!(
                date.partial_cmp(&version).unwrap(),
                expected_inverted_order,
                "expected comparison of {date} and {version} to be #{expected_inverted_order:#?}"
            );
        }

        #[test_case("1.2.3", "1.2.3", Some(Ordering::Equal); "equal semantic versions")]
        #[test_case("3.2.1", "1.2.3", Some(Ordering::Greater); "semantic versions with newer lhs")]
        #[test_case("1.2.3", "3.2.1", Some(Ordering::Less); "semantic versions with newer rhs")]
        #[test_case("2026-01-15", "2026-01-15", Some(Ordering::Equal); "identical stable date versions")]
        #[test_case("2026-02-15", "2026-01-15", Some(Ordering::Greater); "stable date versions with newer lhs")]
        #[test_case("2026-01-15", "2026-02-15", Some(Ordering::Less); "stable date versions with newer rhs")]
        #[test_case("2026-01-15-rc", "2026-01-15-rc", Some(Ordering::Equal); "identical preview date versions")]
        #[test_case("2026-02-15-rc", "2026-01-15-rc", Some(Ordering::Greater); "preview date versions with newer lhs")]
        #[test_case("2026-01-15-rc", "2026-02-15-rc", Some(Ordering::Less); "preview date versions with newer rhs")]
        #[test_case("2026-01-15-alpha", "2026-01-15-bravo", Some(Ordering::Less); "preview date versions with lexicographically later rhs preview segment")]
        #[test_case("2026-01-15-bravo", "2026-01-15-alpha", Some(Ordering::Greater); "preview date versions with lexicographically earlier rhs preview segment")]
        #[test_case("1.2.3", "2026-01-15", Some(Ordering::Greater); "semantic version to date version")]
        #[test_case("1.2.3", "arbitrary", None; "semantic version to arbitrary string")]
        #[test_case("2026-01-15", "1.2.3", Some(Ordering::Less); "date version to semantic version")]
        #[test_case("2026-01-15", "arbitrary", None; "date version to arbitrary string")]
        fn string(resource_version_string: &str, string_slice: &str, expected_order: Option<Ordering>) {
            let version: ResourceVersion = resource_version_string.parse().unwrap();
            let string = string_slice.to_string();

            // Test comparison bidirectionally
            pretty_assertions::assert_eq!(
                version.partial_cmp(&string),
                expected_order,
                "expected comparison of {version} and {string} to be #{expected_order:#?}"
            );

            let expected_inverted_order = match expected_order {
                Some(order) => match order {
                    Ordering::Equal => Some(Ordering::Equal),
                    Ordering::Greater => Some(Ordering::Less),
                    Ordering::Less => Some(Ordering::Greater),
                },
                None => None,
            };

            pretty_assertions::assert_eq!(
                string.partial_cmp(&version),
                expected_inverted_order,
                "expected comparison of {string} and {version} to be #{expected_inverted_order:#?}"
            );
        }

        #[test_case("1.2.3", "1.2.3", Some(Ordering::Equal); "equal semantic versions")]
        #[test_case("3.2.1", "1.2.3", Some(Ordering::Greater); "semantic versions with newer lhs")]
        #[test_case("1.2.3", "3.2.1", Some(Ordering::Less); "semantic versions with newer rhs")]
        #[test_case("2026-01-15", "2026-01-15", Some(Ordering::Equal); "identical stable date versions")]
        #[test_case("2026-02-15", "2026-01-15", Some(Ordering::Greater); "stable date versions with newer lhs")]
        #[test_case("2026-01-15", "2026-02-15", Some(Ordering::Less); "stable date versions with newer rhs")]
        #[test_case("2026-01-15-rc", "2026-01-15-rc", Some(Ordering::Equal); "identical preview date versions")]
        #[test_case("2026-02-15-rc", "2026-01-15-rc", Some(Ordering::Greater); "preview date versions with newer lhs")]
        #[test_case("2026-01-15-rc", "2026-02-15-rc", Some(Ordering::Less); "preview date versions with newer rhs")]
        #[test_case("2026-01-15-alpha", "2026-01-15-bravo", Some(Ordering::Less); "preview date versions with lexicographically later rhs preview segment")]
        #[test_case("2026-01-15-bravo", "2026-01-15-alpha", Some(Ordering::Greater); "preview date versions with lexicographically earlier rhs preview segment")]
        #[test_case("1.2.3", "2026-01-15", Some(Ordering::Greater); "semantic version to date version")]
        #[test_case("1.2.3", "arbitrary", None; "semantic version to arbitrary string")]
        #[test_case("2026-01-15", "1.2.3", Some(Ordering::Less); "date version to semantic version")]
        #[test_case("2026-01-15", "arbitrary", None; "date version to arbitrary string")]
        fn str(resource_version_string: &str, string_slice: &str, expected_order: Option<Ordering>) {
            let version: ResourceVersion = resource_version_string.parse().unwrap();

            // Test comparison bidirectionally
            pretty_assertions::assert_eq!(
                version.partial_cmp(&string_slice),
                expected_order,
                "expected comparison of {version} and {string_slice} to be #{expected_order:#?}"
            );

            let expected_inverted_order = match expected_order {
                Some(order) => match order {
                    Ordering::Equal => Some(Ordering::Equal),
                    Ordering::Greater => Some(Ordering::Less),
                    Ordering::Less => Some(Ordering::Greater),
                },
                None => None,
            };

            pretty_assertions::assert_eq!(
                string_slice.partial_cmp(&version),
                expected_inverted_order,
                "expected comparison of {string_slice} and {version} to be #{expected_inverted_order:#?}"
            );
        }
    }
}
