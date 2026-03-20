// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    use dsc_lib::types::{ResourceVersionReq, SemanticVersionReq};
    use test_case::test_case;

    #[cfg(test)]
    mod parse {
        use dsc_lib::types::{ResourceVersionReq, ResourceVersionReqError};
        use test_case::test_case;

        #[test_case("^1" => matches Ok(_); "major is valid")]
        #[test_case("^1.2" => matches Ok(_); "major.minor is valid")]
        #[test_case("^1.2.3" => matches Ok(_); "major.minor.patch is valid")]
        #[test_case("^1.2.3-pre" => matches Ok(_); "major.minor.patch-pre is valid")]
        #[test_case("^1-pre" => matches Err(_); "major-pre is invalid")]
        #[test_case("^1.2-pre" => matches Err(_); "major.minor-pre is invalid")]
        #[test_case("^1.2.3+build" => matches Err(_); "major.minor.patch+build is invalid")]
        #[test_case("^1.2.3-pre+build" => matches Err(_); "major.minor.patch-pre+build is invalid")]
        #[test_case("^a" => matches Err(_); "invalid_char is invalid")]
        #[test_case("^1.b" => matches Err(_); "major.invalid_char is invalid")]
        #[test_case("^1.2.c" => matches Err(_); "major.minor.invalid_char is invalid")]
        fn literal_version_req(requirement_string: &str) -> Result<ResourceVersionReq, ResourceVersionReqError> {
            ResourceVersionReq::parse(requirement_string)
        }

        #[test_case("^*" => matches Err(_); "wildcard is invalid")]
        #[test_case("^1.*" => matches Ok(_); "major.wildcard is valid")]
        #[test_case("^1.*.*" => matches Ok(_); "major.wildcard.wildcard is valid")]
        #[test_case("^1.2.*" => matches Ok(_); "major.minor.wildcard is valid")]
        #[test_case("^1.*.3" => matches Err(_); "major.wildcard.patch is invalid")]
        #[test_case("^1.2.*-pre" => matches Err(_); "major.minor.wildcard-pre is invalid")]
        #[test_case("^1.*.*-pre" => matches Err(_); "major.wildcard.wildcard-pre is invalid")]
        #[test_case("^1.2.3-*" => matches Err(_); "major.minor.patch-wildcard is invalid")]
        #[test_case("^1.2.3-pre.*" => matches Err(_); "major.minor.patch-pre.wildcard is invalid")]
        #[test_case("^1.x" => matches Err(_); "major.lowercase_x is invalid")]
        #[test_case("^1.X" => matches Err(_); "major.uppercase_x is invalid")]
        #[test_case("^1.2.x" => matches Err(_); "major.minor.lowercase_x is invalid")]
        #[test_case("^1.2.X" => matches Err(_); "major.minor.uppercase_x is invalid")]
        fn wildcard_version_req(requirement_string: &str) -> Result<ResourceVersionReq, ResourceVersionReqError> {
            ResourceVersionReq::parse(requirement_string)
        }

        #[test_case("^ 1.2.3" => matches Ok(_); "caret operator is valid")]
        #[test_case("~ 1.2.3" => matches Ok(_); "tilde operator is valid")]
        #[test_case("= 1.2.3" => matches Ok(_); "exact operator is valid")]
        #[test_case("> 1.2.3" => matches Ok(_); "greater than operator is valid")]
        #[test_case(">= 1.2.3" => matches Ok(_); "greater than or equal to operator is valid")]
        #[test_case("< 1.2.3" => matches Ok(_); "less than operator is valid")]
        #[test_case("<= 1.2.3" => matches Ok(_); "less than or equal to operator is valid")]
        #[test_case("1.2.3" => matches Err(_); "implicit operator is invalid")]
        #[test_case("== 1.2.3" => matches Err(_); "unknown operator is invalid")]
        fn operators_in_version_req(requirement_string: &str) -> Result<ResourceVersionReq, ResourceVersionReqError> {
            ResourceVersionReq::parse(requirement_string)
        }

        #[test_case("^1.2.3, < 1.5" => matches Ok(_); "pair with separating comma is valid")]
        #[test_case("^1, ^1.2, ^1.2.3" => matches Ok(_); "triple with separating comma is valid")]
        #[test_case("<= 1, >= 2" => matches Ok(_); "incompatible pair is valid")]
        #[test_case(", ^1, ^1.2" => matches Err(_); "leading comma is invalid")]
        #[test_case("^1, ^1.2," => matches Err(_); "trailing comma is invalid")]
        #[test_case("^1 ^1.2" => matches Err(_); "omitted separating comma is invalid")]
        #[test_case("^1.*, <1.3.*" => matches Ok(_); "multiple comparators with wildcard is valid")]
        fn multiple_comparator_version_req(requirement_string: &str) -> Result<ResourceVersionReq, ResourceVersionReqError> {
            ResourceVersionReq::parse(requirement_string)
        }

        #[test_case("^1.2" => matches Ok(_); "operator and version without spacing is valid")]
        #[test_case("^   1.2" => matches Ok(_); "operator and version with extra spacing is valid")]
        #[test_case("  ^ 1.2" => matches Ok(_); "leading space is valid")]
        #[test_case("^ 1.2  " => matches Ok(_); "trailing space is valid")]
        #[test_case("^1.2,<1.5" => matches Ok(_); "pair of comparators without spacing is valid")]
        #[test_case("  ^  1.2  ,  <  1.5  " => matches Ok(_); "pair of comparators with extra spacing is valid")]
        fn spacing_in_version_req(requirement_string: &str) -> Result<ResourceVersionReq, ResourceVersionReqError> {
            ResourceVersionReq::parse(requirement_string)
        }
    }

    #[test_case("^1.2.3" => true; "single comparator is semver")]
    #[test_case("^1.2, >1.5" => true; "multi comparator is semver")]
    #[test_case("2026-02-01" => false; "stable date version is not semver")]
    #[test_case("2026-02-01-rc" => false; "preview date version is not semver")]
    fn is_semver(requirement_string: &str) -> bool {
        ResourceVersionReq::parse(requirement_string).unwrap().is_semver()
    }

    #[test_case("^1.2.3" => false; "single comparator is not date")]
    #[test_case("^1.2, >1.5" => false; "multi comparator is not date")]
    #[test_case("2026-02-01" => true; "stable date version is date")]
    #[test_case("2026-02-01-rc" => true; "preview date version is date")]
    fn is_date_version(requirement_string: &str) -> bool {
        ResourceVersionReq::parse(requirement_string).unwrap().is_date_version()
    }

    #[test_case("^1.2.3" => matches Some(_); "single comparator returns some")]
    #[test_case("^1.2, >1.5" => matches Some(_); "multi comparator returns some")]
    #[test_case("2026-02-01" => matches None; "stable date version returns none")]
    #[test_case("2026-02-01-rc" => matches None; "preview date version returns none")]
    fn as_semver_req(requirement_string: &str) -> Option<SemanticVersionReq> {
        ResourceVersionReq::parse(requirement_string).unwrap().as_semver_req().cloned()
    }

    #[cfg(test)]
    mod matches {
        use dsc_lib::types::{ResourceVersion, ResourceVersionReq};
        use test_case::test_case;

        fn check(requirement: &str, versions: Vec<&str>, should_match: bool) {
            let req = ResourceVersionReq::parse(requirement).unwrap();
            let expected = if should_match { "match" } else { "not match" };
            for version in versions {
                pretty_assertions::assert_eq!(
                    req.matches(&ResourceVersion::parse(version).unwrap()),
                    should_match,
                    "expected version '{version}' to {expected} requirement '{requirement}'"
                );
            }
        }

        // Only test a subset of valid semantic reqs since the matches method for SemanticVersionReq
        // more thoroughly covers these cases
        #[test_case("^1", vec!["1.0.0", "1.2.0", "1.3.0"], true; "matching major")]
        #[test_case("^1", vec!["0.1.0", "2.0.0", "1.2.3-rc.1", "2026-02-01"], false; "not matching major")]
        #[test_case("^1.2", vec!["1.2.0", "1.2.3", "1.3.0"], true; "matching major.minor")]
        #[test_case("^1.2", vec!["1.0.0", "2.0.0", "1.2.3-rc.1", "2026-02-01"], false; "not matching major.minor")]
        #[test_case("^1.2.3", vec!["1.2.3", "1.2.4", "1.3.0"], true; "matching major.minor.patch")]
        #[test_case("^1.2.3", vec!["1.2.0", "2.0.0", "1.2.3-rc.1", "2026-02-01"], false; "not matching major.minor.patch")]
        #[test_case("^1.2.3-rc.2", vec!["1.2.3", "1.3.0", "1.2.3-rc.2", "1.2.3-rc.3"], true; "matching major.minor.patch-pre")]
        #[test_case("^1.2.3-rc.2", vec!["1.2.0", "2.0.0", "1.2.3-rc.1", "1.3.0-rc.2", "2026-02-01"], false; "not matching major.minor.patch-pre")]
        fn semantic(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }

        #[test_case("2026-02-01", vec!["2026-02-01"], true; "matching version as date")]
        #[test_case("2026-02-01", vec!["2026-02-02", "1.2.3"], false; "not matching version as date")]
        fn date(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }
    }
}

#[cfg(test)]
mod schema {
    use std::{ops::Index, sync::LazyLock};

    use dsc_lib::types::ResourceVersionReq;
    use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    use jsonschema::Validator;
    use regex::Regex;
    use schemars::{schema_for, Schema};
    use serde_json::{json, Value};
    use test_case::test_case;

    static ROOT_SCHEMA: LazyLock<Schema> = LazyLock::new(|| schema_for!(ResourceVersionReq));
    static SEMANTIC_VARIANT_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
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
    #[test_case("title", &*SEMANTIC_VARIANT_SCHEMA; "semver.title")]
    #[test_case("description", &*SEMANTIC_VARIANT_SCHEMA; "semver.description")]
    #[test_case("markdownDescription", &*SEMANTIC_VARIANT_SCHEMA; "semver.markdownDescription")]
    #[test_case("title", &*DATE_VARIANT_SCHEMA; "dateVersion.title")]
    #[test_case("description", &*DATE_VARIANT_SCHEMA; "dateVersion.description")]
    #[test_case("deprecationMessage", &*DATE_VARIANT_SCHEMA; "dateVersion.deprecationMessage")]
    #[test_case("markdownDescription", &*DATE_VARIANT_SCHEMA; "dateVersion.markdownDescription")]
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
            (&*SEMANTIC_VARIANT_SCHEMA).get_keyword_as_string("$ref").is_some_and(|kv| !kv.is_empty())
        )
    }

    #[test]
    fn date_subschema_is_reference() {
        assert!(
            (&*DATE_VARIANT_SCHEMA).get_keyword_as_string("$ref").is_some_and(|kv| !kv.is_empty())
        )
    }

    #[test_case(&json!("^1.2.3") => true ; "single comparator semantic version req string value is valid")]
    #[test_case(&json!("^1.2.3, <1.5") => true ; "multi comparator semantic version req string value is valid")]
    #[test_case(&json!("=1.2.3a") => false ; "invalid semantic version req string value is invalid")]
    #[test_case(&json!("2026-01-15") => true ; "iso8601 full date string value is valid")]
    #[test_case(&json!("2026-01-15-rc") => true ; "iso8601 full date with prerelease segment string value is valid")]
    #[test_case(&json!("2026-01") => false ; "iso8601 date year month string value is invalid")]
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
    use dsc_lib::types::ResourceVersionReq;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case("^1.2.3"; "single comparator semantic req string serializes to string")]
    #[test_case("^1.2.3, <1.4"; "multi comparator semantic req serializes to string")]
    #[test_case("2026-02-01"; "stable date req serializes to string")]
    #[test_case("2026-02-01-rc"; "preview date req serializes to string")]
    fn serializing(requirement: &str) {
        let actual = serde_json::to_string(
            &ResourceVersionReq::parse(requirement).unwrap()
        ).expect("serialization should never fail");

        let expected = format!(r#""{requirement}""#);

        pretty_assertions::assert_eq!(actual, expected);
    }

    #[test_case(json!("^1.2.3") => matches Ok(_); "valid req string value succeeds")]
    #[test_case(json!("2026-02-01") => matches Ok(_); "valid stable date version string value succeeds")]
    #[test_case(json!("2026-02-01-rc") => matches Ok(_); "valid preview date version string value succeeds")]
    #[test_case(json!("2026-02-29") => matches Err(_); "invalid date version string value fails")]
    #[test_case(json!("1.*.3") => matches Err(_); "invalid req string value fails")]
    #[test_case(json!(true) => matches Err(_); "boolean value fails")]
    #[test_case(json!(1) => matches Err(_); "integer value fails")]
    #[test_case(json!(1.2) => matches Err(_); "float value fails")]
    #[test_case(json!({"req": "^1.2.3"}) => matches Err(_); "object value fails")]
    #[test_case(json!(["^1.2.3"]) => matches Err(_); "array value fails")]
    #[test_case(serde_json::Value::Null => matches Err(_); "null value fails")]
    fn deserializing(value: Value) -> Result<ResourceVersionReq, serde_json::Error> {
        serde_json::from_value::<ResourceVersionReq>(value)
    }
}

#[cfg(test)]
mod traits {
    #[cfg(test)]
    mod default {
        use dsc_lib::types::{ResourceVersionReq, SemanticVersionReq};

        #[test]
        fn default() {
            pretty_assertions::assert_eq!(
                ResourceVersionReq::default(),
                SemanticVersionReq::default(),
            )
        }
    }

    #[cfg(test)]
    mod display {
        use dsc_lib::types::ResourceVersionReq;
        use test_case::test_case;

        #[test_case("  ^ 1.2  ", "^1.2"; "semantic req with single comparator")]
        #[test_case("^1.2, < 1.4", "^1.2, <1.4"; "semantic req with multiple comparators")]
        #[test_case("^1.*", "^1"; "semantic req with a wildcard")]
        #[test_case("2020-02-01", "2020-02-01"; "stable date req")]
        #[test_case("2020-02-01-rc", "2020-02-01-rc"; "preview date req")]
        fn format(requirement: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                format!("req: '{}'", ResourceVersionReq::parse(requirement).unwrap()),
                format!("req: '{}'", expected)
            )
        }

        #[test_case("  ^ 1.2  ", "^1.2"; "semantic req with single comparator")]
        #[test_case("^1.2, < 1.4", "^1.2, <1.4"; "semantic req with multiple comparators")]
        #[test_case("^1.*", "^1"; "semantic req with a wildcard")]
        #[test_case("2020-02-01", "2020-02-01"; "stable date req")]
        #[test_case("2020-02-01-rc", "2020-02-01-rc"; "preview date req")]
        fn to_string(requirement: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                ResourceVersionReq::parse(requirement).unwrap().to_string(),
                expected.to_string()
            )
        }
    }

    #[cfg(test)]
    mod from {
        use dsc_lib::types::{
            DateVersion,
            ResourceVersionReq,
            ResourceVersionReq::*,
            SemanticVersionReq
        };

        #[test]
        fn semantic_version_req() {
            let semantic = SemanticVersionReq::parse("^1.2.3").unwrap();
            match ResourceVersionReq::from(semantic.clone()) {
                Semantic(req) => pretty_assertions::assert_eq!(req, semantic),
                _ => panic!("should never fail to convert as Semantic version requirement"),
            }
        }

        #[test]
        fn date_version() {
            let date = DateVersion::parse("2026-02-01").unwrap();
            match ResourceVersionReq::from(date.clone()) {
                Date(req) => pretty_assertions::assert_eq!(req, date),
                _ => panic!("should never fail to convert as date version requirement"),
            }
        }
    }
    #[cfg(test)]
    mod try_from {
        use dsc_lib::types::{ResourceVersionReq, ResourceVersionReqError};
        use test_case::test_case;

        #[test_case("^1.2.3" => matches Ok(_); "single comparator semantic req is valid")]
        #[test_case("^1.2, <1.5" => matches Ok(_); "multi comparator semantic req is valid")]
        #[test_case("2020-02-01" => matches Ok(_); "stable date req is valid")]
        #[test_case("2020-02-01-rc" => matches Ok(_); "preview date req is valid")]
        #[test_case("arbitrary" => matches Err(_); "arbitrary string is invalid")]
        fn string(requirement_string: &str) -> Result<ResourceVersionReq, ResourceVersionReqError> {
            ResourceVersionReq::try_from(requirement_string.to_string())
        }

        #[test_case("^1.2.3" => matches Ok(_); "single comparator semantic req is valid")]
        #[test_case("^1.2, <1.5" => matches Ok(_); "multi comparator semantic req is valid")]
        #[test_case("2020-02-01" => matches Ok(_); "stable date req is valid")]
        #[test_case("2020-02-01-rc" => matches Ok(_); "preview date req is valid")]
        #[test_case("arbitrary" => matches Err(_); "arbitrary string is invalid")]
        fn str(string_slice: &str) -> Result<ResourceVersionReq, ResourceVersionReqError> {
            ResourceVersionReq::try_from(string_slice)
        }
    }

    #[cfg(test)]
    mod from_str {
        use dsc_lib::types::{ResourceVersionReq, ResourceVersionReqError};
        use test_case::test_case;

        #[test_case("^1.2.3" => matches Ok(_); "single comparator semantic req is valid")]
        #[test_case("^1.2, <1.5" => matches Ok(_); "multi comparator semantic req is valid")]
        #[test_case("2020-02-01" => matches Ok(_); "stable date req is valid")]
        #[test_case("2020-02-01-rc" => matches Ok(_); "preview date req is valid")]
        #[test_case("arbitrary" => matches Err(_); "arbitrary string is invalid")]
        fn parse(input: &str) -> Result<ResourceVersionReq, ResourceVersionReqError> {
            input.parse()
        }
    }

    #[cfg(test)]
    mod into {
        use dsc_lib::types::ResourceVersionReq;
        use test_case::test_case;

        #[test_case("^1.2.3"; "single comparator semantic req")]
        #[test_case("^1.2, <1.5"; "multi comparator semantic req")]
        #[test_case("2020-02-01"; "stable date req")]
        #[test_case("2020-02-01-rc"; "preview date req")]
        fn string(requirement_string: &str) {
            let actual: String = ResourceVersionReq::parse(requirement_string).unwrap().into();
            let expected = requirement_string.to_string();

            pretty_assertions::assert_eq!(actual, expected)
        }
    }

    #[cfg(test)]
    mod try_into {
        use dsc_lib::types::{
            DateVersion,
            ResourceVersionReq,
            ResourceVersionReqError,
            SemanticVersionReq
        };
        use test_case::test_case;

        #[test_case("^1.2.3" => matches Ok(_); "single comparator semantic req converts")]
        #[test_case("^1.2, <1.5" => matches Ok(_); "multi comparator semantic req converts")]
        #[test_case("2020-02-01" => matches Err(_); "stable date req fails")]
        #[test_case("2020-02-01-rc" => matches Err(_); "preview date req fails")]
        fn semantic_version_req(requirement: &str) -> Result<SemanticVersionReq, ResourceVersionReqError> {
            TryInto::<SemanticVersionReq>::try_into(ResourceVersionReq::parse(requirement).unwrap())
        }

        #[test_case("^1.2.3" => matches Err(_); "single comparator semantic req fails")]
        #[test_case("^1.2, <1.5" => matches Err(_); "multi comparator semantic req fails")]
        #[test_case("2020-02-01" => matches Ok(_); "stable date req converts")]
        #[test_case("2020-02-01-rc" => matches Ok(_); "preview date req converts")]
        fn date_version(requirement: &str) -> Result<DateVersion, ResourceVersionReqError> {
            TryInto::<DateVersion>::try_into(ResourceVersionReq::parse(requirement).unwrap())
        }
    }

    #[cfg(test)]
    mod partial_eq {
        use dsc_lib::types::{DateVersion, ResourceVersionReq, SemanticVersionReq};
        use test_case::test_case;

        #[test_case("^1.2", "^1.2.*", true; "equivalent semantic reqs")]
        #[test_case("^1.2.3", "^1.2.3", true; "identical semantic reqs")]
        #[test_case(">1.2.3", "<1.2.3", false; "different semantic reqs")]
        #[test_case("2026-02-01", "2026-02-01", true; "identical stable date reqs")]
        #[test_case("2026-02-01-rc", "2026-02-01-rc", true; "identical preview date reqs")]
        #[test_case("2026-02-01-rc", "2026-02-01-RC", false; "differently cased preview date reqs")]
        fn resource_version_req(lhs: &str, rhs: &str, should_be_equal: bool) {
            if should_be_equal {
                pretty_assertions::assert_eq!(
                    ResourceVersionReq::parse(lhs).unwrap(),
                    ResourceVersionReq::parse(rhs).unwrap()
                )
            } else {
                pretty_assertions::assert_ne!(
                    ResourceVersionReq::parse(lhs).unwrap(),
                    ResourceVersionReq::parse(rhs).unwrap()
                )
            }
        }

        #[test_case("^1.2", "^1.2.*", true; "equivalent semantic reqs")]
        #[test_case("^1.2.3", "^1.2.3", true; "identical semantic reqs")]
        #[test_case(">1.2.3", "<1.2.3", false; "different semantic reqs")]
        #[test_case("2026-02-01", "^1.2.3", false; "date req and semantic req")]
        fn semantic_version_req(
            resource_version_req_string: &str,
            semantic_version_req_string: &str,
            should_be_equal: bool,
        ) {
            let req = ResourceVersionReq::parse(resource_version_req_string).unwrap();
            let semantic = SemanticVersionReq::parse(semantic_version_req_string).unwrap();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                req == semantic,
                should_be_equal,
                "expected comparison of {req} and {semantic} to be #{should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                semantic == req,
                should_be_equal,
                "expected comparison of {semantic} and {req} to be #{should_be_equal}"
            );
        }

        #[test_case("2026-02-01", "2026-02-01", true; "identical stable date reqs")]
        #[test_case("2026-02-01-rc", "2026-02-01-rc", true; "identical preview date reqs")]
        #[test_case("2026-02-01-rc", "2026-02-01-RC", false; "differently cased preview date reqs")]
        #[test_case("^1.2.3", "2026-02-01", false; "date req and semantic req")]
        fn date_version(
            resource_version_req_string: &str,
            date_version_string: &str,
            should_be_equal: bool,
        ) {
            let req = ResourceVersionReq::parse(resource_version_req_string).unwrap();
            let date = DateVersion::parse(date_version_string).unwrap();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                req == date,
                should_be_equal,
                "expected comparison of {req} and {date} to be #{should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                date == req,
                should_be_equal,
                "expected comparison of {date} and {req} to be #{should_be_equal}"
            );
        }

        #[test_case("^1.2", "^1.2.*", true; "equivalent semantic reqs")]
        #[test_case("^1.2.3", "^1.2.3", true; "identical semantic reqs")]
        #[test_case(">1.2.3", "<1.2.3", false; "different semantic reqs")]
        #[test_case("2026-02-01", "2026-02-01", true; "identical stable date reqs")]
        #[test_case("2026-02-01-rc", "2026-02-01-rc", true; "identical preview date reqs")]
        #[test_case("2026-02-01-rc", "2026-02-01-RC", false; "differently cased preview date reqs")]
        #[test_case("^1.2.3", "arbitrary", false; "semantic req and arbitrary string")]
        #[test_case("2026-02-01", "arbitrary", false; "date req and arbitrary string")]
        fn str(resource_version_req_string: &str, string_slice: &str, should_be_equal: bool) {
            let req: ResourceVersionReq = resource_version_req_string.parse().unwrap();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                req == string_slice,
                should_be_equal,
                "expected comparison of {req} and {string_slice} to be #{should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string_slice == req,
                should_be_equal,
                "expected comparison of {string_slice} and {req} to be #{should_be_equal}"
            );
        }

        #[test_case("^1.2", "^1.2.*", true; "equivalent semantic reqs")]
        #[test_case("^1.2.3", "^1.2.3", true; "identical semantic reqs")]
        #[test_case(">1.2.3", "<1.2.3", false; "different semantic reqs")]
        #[test_case("2026-02-01", "2026-02-01", true; "identical stable date reqs")]
        #[test_case("2026-02-01-rc", "2026-02-01-rc", true; "identical preview date reqs")]
        #[test_case("2026-02-01-rc", "2026-02-01-RC", false; "differently cased preview date reqs")]
        #[test_case("^1.2.3", "arbitrary", false; "semantic req and arbitrary string")]
        #[test_case("2026-02-01", "arbitrary", false; "date req and arbitrary string")]
        fn string(resource_version_req_string: &str, string_slice: &str, should_be_equal: bool) {
            let req: ResourceVersionReq = resource_version_req_string.parse().unwrap();
            let string = string_slice.to_string();
            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                req == string,
                should_be_equal,
                "expected comparison of {req} and {string} to be #{should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string == req,
                should_be_equal,
                "expected comparison of {string} and {req} to be #{should_be_equal}"
            );
        }
    }
}
