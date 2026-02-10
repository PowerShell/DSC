// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    use dsc_lib::types::{ResourceVersionReq, SemanticVersionReq};
    use test_case::test_case;

    #[cfg(test)]
    mod new {
        use dsc_lib::types::ResourceVersionReq;
        use dsc_lib::types::ResourceVersionReq::*;
        use test_case::test_case;

        #[test_case("1" => matches Semantic(_); "major is semantic")]
        #[test_case("1.2" => matches Semantic(_); "major.minor is semantic")]
        #[test_case("1.2.3" => matches Semantic(_); "major.minor.patch is semantic")]
        #[test_case("1.2.3-pre" => matches Semantic(_); "major.minor.patch-pre is semantic")]
        #[test_case("1-pre" => matches Arbitrary(_); "major-pre is arbitrary")]
        #[test_case("1.2-pre" => matches Arbitrary(_); "major.minor-pre is arbitrary")]
        #[test_case("1.2.3+build" => matches Arbitrary(_); "major.minor.patch+build is arbitrary")]
        #[test_case("1.2.3-pre+build" => matches Arbitrary(_); "major.minor.patch-pre+build is arbitrary")]
        #[test_case("a" => matches Arbitrary(_); "invalid_char is arbitrary")]
        #[test_case("1.b" => matches Arbitrary(_); "major.invalid_char is arbitrary")]
        #[test_case("1.2.c" => matches Arbitrary(_); "major.minor.invalid_char is arbitrary")]
        fn literal_version_req(requirement_string: &str) -> ResourceVersionReq {
            ResourceVersionReq::new(requirement_string)
        }

        #[test_case("1.*" => matches Semantic(_); "major.wildcard is semantic")]
        #[test_case("1.*.*" => matches Semantic(_); "major.wildcard.wildcard is semantic")]
        #[test_case("1.2.*" => matches Semantic(_); "major.minor.wildcard is semantic")]
        #[test_case("1.*.3" => matches Arbitrary(_); "major.wildcard.patch is arbitrary")]
        #[test_case("1.2.*-pre" => matches Arbitrary(_); "major.minor.wildcard-pre is arbitrary")]
        #[test_case("1.*.*-pre" => matches Arbitrary(_); "major.wildcard.wildcard-pre is arbitrary")]
        #[test_case("1.2.3-*" => matches Arbitrary(_); "major.minor.patch-wildcard is arbitrary")]
        #[test_case("1.2.3-pre.*" => matches Arbitrary(_); "major.minor.patch-pre.wildcard is arbitrary")]
        fn wildcard_version_req(requirement_string: &str) -> ResourceVersionReq {
            ResourceVersionReq::new(requirement_string)
        }

        #[test_case("1.2.3" => matches Semantic(_); "implicit operator is semantic")]
        #[test_case("^ 1.2.3" => matches Semantic(_); "caret operator is semantic")]
        #[test_case("~ 1.2.3" => matches Semantic(_); "tilde operator is semantic")]
        #[test_case("= 1.2.3" => matches Semantic(_); "exact operator is semantic")]
        #[test_case("> 1.2.3" => matches Semantic(_); "greater than operator is semantic")]
        #[test_case(">= 1.2.3" => matches Semantic(_); "greater than or equal to operator is semantic")]
        #[test_case("< 1.2.3" => matches Semantic(_); "less than operator is semantic")]
        #[test_case("<= 1.2.3" => matches Semantic(_); "less than or equal to operator is semantic")]
        #[test_case("== 1.2.3" => matches Arbitrary(_); "invalid operator is arbitrary")]
        fn operators_in_version_req(requirement_string: &str) -> ResourceVersionReq {
            ResourceVersionReq::new(requirement_string)
        }

        #[test_case("1.2.3, < 1.5" => matches Semantic(_); "pair with separating comma is semantic")]
        #[test_case("1, 1.2, 1.2.3" => matches Semantic(_); "triple with separating comma is semantic")]
        #[test_case("<= 1, >= 2" => matches Semantic(_); "incompatible pair is semantic")]
        #[test_case(", 1, 1.2" => matches Arbitrary(_); "leading comma is arbitrary")]
        #[test_case("1, 1.2," => matches Arbitrary(_); "trailing comma is arbitrary")]
        #[test_case("1 1.2" => matches Arbitrary(_); "omitted separating comma is arbitrary")]
        #[test_case("1.*, < 1.3.*" => matches Semantic(_); "multiple comparators with wildcard is semantic")]
        fn multiple_comparator_version_req(requirement_string: &str) -> ResourceVersionReq {
            ResourceVersionReq::new(requirement_string)
        }

        #[test_case("^1.2" => matches Semantic(_); "operator and version without spacing is semantic")]
        #[test_case("^   1.2" => matches Semantic(_); "operator and version with extra spacing is semantic")]
        #[test_case("  ^ 1.2" => matches Semantic(_); "leading space is semantic")]
        #[test_case("^ 1.2  " => matches Semantic(_); "trailing space is semantic")]
        #[test_case("^1.2,<1.5" => matches Semantic(_); "pair of comparators without spacing is semantic")]
        #[test_case("  ^  1.2  ,  <  1.5  " => matches Semantic(_); "pair of comparators with extra spacing is semantic")]
        fn spacing_in_version_req(requirement_string: &str) -> ResourceVersionReq {
            ResourceVersionReq::new(requirement_string)
        }
    }

    #[test_case("1.2.3" => true; "single comparator is semver")]
    #[test_case("^1.2, >1.5" => true; "multi comparator is semver")]
    #[test_case("2026-02-01" => false; "date string is not semver")]
    #[test_case("arbitrary" => false; "arbitrary string is not semver")]
    fn is_semver(requirement_string: &str) -> bool {
        ResourceVersionReq::new(requirement_string).is_semver()
    }

    #[test_case("1.2.3" => false; "single comparator is not arbitrary")]
    #[test_case("^1.2, >1.5" => false; "multi comparator is not arbitrary")]
    #[test_case("2026-02-01" => true; "date string is arbitrary")]
    #[test_case("arbitrary" => true; "arbitrary string is arbitrary")]
    fn is_arbitrary(requirement_string: &str) -> bool {
        ResourceVersionReq::new(requirement_string).is_arbitrary()
    }

    #[test_case("1.2.3" => matches Some(_); "single comparator returns some")]
    #[test_case("^1.2, >1.5" => matches Some(_); "multi comparator returns some")]
    #[test_case("2026-02-01" => matches None; "date string returns none")]
    #[test_case("arbitrary" => matches None; "arbitrary string returns none")]
    fn as_semver_req(requirement_string: &str) -> Option<SemanticVersionReq> {
        ResourceVersionReq::new(requirement_string).as_semver_req().cloned()
    }

    #[cfg(test)]
    mod matches {
        use dsc_lib::types::{ResourceVersion, ResourceVersionReq};
        use test_case::test_case;

        fn check(requirement: &str, versions: Vec<&str>, should_match: bool) {
            let req = ResourceVersionReq::new(requirement);
            let expected = if should_match { "match" } else { "not match" };
            for version in versions {
                pretty_assertions::assert_eq!(
                    req.matches(&ResourceVersion::new(version)),
                    should_match,
                    "expected version '{version}' to {expected} requirement '{requirement}'"
                );
            }
        }

        // Only test a subset of valid semantic reqs since the matches method for SemanticVersionReq
        // more thoroughly covers these cases
        #[test_case("1", vec!["1.0.0", "1.2.0", "1.3.0"], true; "matching major")]
        #[test_case("1", vec!["0.1.0", "2.0.0", "1.2.3-rc.1", "2026-02-01", "arbitrary"], false; "not matching major")]
        #[test_case("1.2", vec!["1.2.0", "1.2.3", "1.3.0"], true; "matching major.minor")]
        #[test_case("1.2", vec!["1.0.0", "2.0.0", "1.2.3-rc.1", "2026-02-01", "arbitrary"], false; "not matching major.minor")]
        #[test_case("1.2.3", vec!["1.2.3", "1.2.4", "1.3.0"], true; "matching major.minor.patch")]
        #[test_case("1.2.3", vec!["1.2.0", "2.0.0", "1.2.3-rc.1", "2026-02-01", "arbitrary"], false; "not matching major.minor.patch")]
        #[test_case("1.2.3-rc.2", vec!["1.2.3", "1.3.0", "1.2.3-rc.2", "1.2.3-rc.3"], true; "matching major.minor.patch-pre")]
        #[test_case("1.2.3-rc.2", vec!["1.2.0", "2.0.0", "1.2.3-rc.1", "1.3.0-rc.2", "2026-02-01", "arbitrary"], false; "not matching major.minor.patch-pre")]
        fn semantic(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }

        #[test_case("2026-02-01", vec!["2026-02-01"], true; "matching version as date")]
        #[test_case("2026-02-01", vec!["2026-02-02", "2026-02", "arbitrary", "2026.02.01", "1.2.3"], false; "not matching version as date")]
        #[test_case("Arbitrary", vec!["Arbitrary"], true; "matching version as arbitrary string")]
        #[test_case("Arbitrary", vec!["arbitrary", " Arbitrary", "Arbitrary ", "2026-02-01", "1.2.3"], false; "not matching version as arbitrary string")]
        fn arbitrary(requirement: &str, versions: Vec<&str>, should_match: bool) {
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
    static ARBITRARY_VARIANT_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
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
    #[test_case("title", &*ARBITRARY_VARIANT_SCHEMA; "arbitrary.title")]
    #[test_case("description", &*ARBITRARY_VARIANT_SCHEMA; "arbitrary.description")]
    #[test_case("deprecationMessage", &*ARBITRARY_VARIANT_SCHEMA; "arbitrary.deprecationMessage")]
    #[test_case("markdownDescription", &*ARBITRARY_VARIANT_SCHEMA; "arbitrary.markdownDescription")]
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

    #[test_case(&json!("^1.2.3") => true ; "single comparator semantic version req string value is valid")]
    #[test_case(&json!("^1.2.3, <1.5") => true ; "multi comparator semantic version req string value is valid")]
    #[test_case(&json!("=1.2.3a") => true ; "invalid semantic version req string value is valid")]
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
    use dsc_lib::types::ResourceVersionReq;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case("^1.2.3"; "single comparator semantic req string serializes to string")]
    #[test_case("^1.2.3, <1.4"; "multi comparator semantic req serializes to string")]
    #[test_case("2026-02-1"; "arbitrary req formatted as date serializes to string")]
    #[test_case("arbitrary"; "arbitrary req serializes to string")]
    fn serializing(requirement: &str) {
        let actual = serde_json::to_string(
            &ResourceVersionReq::new(requirement)
        ).expect("serialization should never fail");

        let expected = format!(r#""{requirement}""#);

        pretty_assertions::assert_eq!(actual, expected);
    }

    #[test_case(json!("1.2.3") => matches Ok(_); "valid req string value succeeds")]
    #[test_case(json!("a.b") => matches Ok(_); "invalid req string value succeeds")]
    #[test_case(json!(true) => matches Err(_); "boolean value is invalid")]
    #[test_case(json!(1) => matches Err(_); "integer value is invalid")]
    #[test_case(json!(1.2) => matches Err(_); "float value is invalid")]
    #[test_case(json!({"req": "1.2.3"}) => matches Err(_); "object value is invalid")]
    #[test_case(json!(["1.2.3"]) => matches Err(_); "array value is invalid")]
    #[test_case(serde_json::Value::Null => matches Err(_); "null value is invalid")]
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

        #[test_case("1.2", "^1.2"; "semantic req with single comparator")]
        #[test_case("1.2, < 1.4", "^1.2, <1.4"; "semantic req with multiple comparators")]
        #[test_case("1.*", "1.*"; "semantic req with a wildcard")]
        #[test_case("2020-02-01", "2020-02-01"; "arbitrary req as date")]
        #[test_case("Arbitrary", "Arbitrary"; "arbitrary req as string")]
        fn format(requirement: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                format!("req: '{}'", ResourceVersionReq::new(requirement)),
                format!("req: '{}'", expected)
            )
        }

        #[test_case("1.2", "^1.2"; "semantic req with single comparator")]
        #[test_case("1.2, < 1.4", "^1.2, <1.4"; "semantic req with multiple comparators")]
        #[test_case("1.*", "1.*"; "semantic req with a wildcard")]
        #[test_case("2020-02-01", "2020-02-01"; "arbitrary req as date")]
        #[test_case("Arbitrary", "Arbitrary"; "arbitrary req as string")]
        fn to_string(requirement: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                ResourceVersionReq::new(requirement).to_string(),
                expected.to_string()
            )
        }
    }

    #[cfg(test)]
    mod from {
        use dsc_lib::types::{ResourceVersionReq, SemanticVersionReq};
        use dsc_lib::types::ResourceVersionReq::*;
        use test_case::test_case;

        #[test]
        fn semantic_version_req() {
            let semantic = SemanticVersionReq::parse("^1.2.3").unwrap();
            match ResourceVersionReq::from(semantic.clone()) {
                Semantic(req) => pretty_assertions::assert_eq!(req, semantic),
                Arbitrary(_) => {
                    panic!("should never fail to convert as Semantic version requirement")
                }
            }
         }
         #[test_case("^1.2.3" => matches Semantic(_); "single comparator semantic req")]
         #[test_case("^1.2, <1.5" => matches Semantic(_); "multi comparator semantic req")]
         #[test_case("2020-02-01" => matches Arbitrary(_); "date-formatted arbitrary req")]
         #[test_case("arbitrary" => matches Arbitrary(_); "arbitrary string req")]
         fn string(requirement_string: &str) -> ResourceVersionReq {
            ResourceVersionReq::from(requirement_string.to_string())
         }

         #[test_case("^1.2.3" => matches Semantic(_); "single comparator semantic req")]
         #[test_case("^1.2, <1.5" => matches Semantic(_); "multi comparator semantic req")]
         #[test_case("2020-02-01" => matches Arbitrary(_); "date-formatted arbitrary req")]
         #[test_case("arbitrary" => matches Arbitrary(_); "arbitrary string req")]
         fn str(string_slice: &str) -> ResourceVersionReq {
            ResourceVersionReq::from(string_slice)
         }
    }

    #[cfg(test)]
    mod from_str {
        use dsc_lib::types::ResourceVersionReq;
        use dsc_lib::types::ResourceVersionReq::*;
        use test_case::test_case;

        #[test_case("^1.2.3" => matches Semantic(_); "single comparator semantic req")]
        #[test_case("^1.2, <1.5" => matches Semantic(_); "multi comparator semantic req")]
        #[test_case("2020-02-01" => matches Arbitrary(_); "date-formatted arbitrary req")]
        #[test_case("arbitrary" => matches Arbitrary(_); "arbitrary string req")]
        fn parse(input: &str) -> ResourceVersionReq {
            input.parse().expect("parse should be infallible")
        }
    }

    #[cfg(test)]
    mod into {
        use dsc_lib::types::ResourceVersionReq;
        use test_case::test_case;

        #[test_case("^1.2.3"; "single comparator semantic req")]
        #[test_case("^1.2, <1.5"; "multi comparator semantic req")]
        #[test_case("2020-02-01"; "date-formatted arbitrary req")]
        #[test_case("arbitrary"; "arbitrary string req")]
        fn string(requirement_string: &str) {
            let actual: String = ResourceVersionReq::new(requirement_string).into();
            let expected = requirement_string.to_string();

            pretty_assertions::assert_eq!(actual, expected)
        }
    }

    #[cfg(test)]
    mod try_into {
        use dsc_lib::{dscerror::DscError, types::{ResourceVersionReq, SemanticVersionReq}};
        use test_case::test_case;

        #[test_case("^1.2.3" => matches Ok(_); "single comparator semantic req converts")]
        #[test_case("^1.2, <1.5" => matches Ok(_); "multi comparator semantic req converts")]
        #[test_case("2020-02-01" => matches Err(_); "date-formatted arbitrary req fails")]
        #[test_case("arbitrary" => matches Err(_); "arbitrary string req fails")]
        fn semantic_version_req(requirement: &str) -> Result<SemanticVersionReq, DscError> {
            TryInto::<SemanticVersionReq>::try_into(ResourceVersionReq::new(requirement))
        }
    }

    #[cfg(test)]
    mod partial_eq {
        use dsc_lib::types::{ResourceVersionReq, SemanticVersionReq};
        use test_case::test_case;

        #[test_case("1.2.3", "^1.2.3", true; "equivalent semantic reqs")]
        #[test_case("^1.2.3", "^1.2.3", true; "identical semantic reqs")]
        #[test_case(">1.2.3", "<1.2.3", false; "different semantic reqs")]
        #[test_case("Arbitrary", "Arbitrary", true; "identical arbitrary reqs")]
        #[test_case("Arbitrary", "arbitrary", false; "differently cased arbitrary reqs")]
        #[test_case("foo", "bar", false; "different arbitrary reqs")]
        fn resource_version_req(lhs: &str, rhs: &str, should_be_equal: bool) {
            if should_be_equal {
                pretty_assertions::assert_eq!(ResourceVersionReq::new(lhs), ResourceVersionReq::new(rhs))
            } else {
                pretty_assertions::assert_ne!(ResourceVersionReq::new(lhs), ResourceVersionReq::new(rhs))
            }
        }

        #[test_case("1.2.3", "^1.2.3", true; "equivalent semantic reqs")]
        #[test_case("^1.2.3", "^1.2.3", true; "identical semantic reqs")]
        #[test_case(">1.2.3", "<1.2.3", false; "different semantic reqs")]
        #[test_case("Arbitrary", "1.2.3", false; "arbitrary req and semantic req")]
        fn semantic_version_req(
            resource_version_req_string: &str,
            semantic_version_req_string: &str,
            should_be_equal: bool,
        ) {
            let req = ResourceVersionReq::new(resource_version_req_string);
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

        #[test_case("1.2.3", "^1.2.3", true; "equivalent semantic reqs")]
        #[test_case("^1.2.3", "^1.2.3", true; "identical semantic reqs")]
        #[test_case(">1.2.3", "<1.2.3", false; "different semantic reqs")]
        #[test_case("Arbitrary", "1.2.3", false; "arbitrary req and semantic req")]
        #[test_case("Arbitrary", "Arbitrary", true; "identical arbitrary reqs")]
        #[test_case("Arbitrary", "arbitrary", false; "differently cased arbitrary reqs")]
        #[test_case("foo", "bar", false; "different arbitrary reqs")]
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

        #[test_case("1.2.3", "^1.2.3", true; "equivalent semantic reqs")]
        #[test_case("^1.2.3", "^1.2.3", true; "identical semantic reqs")]
        #[test_case(">1.2.3", "<1.2.3", false; "different semantic reqs")]
        #[test_case("Arbitrary", "1.2.3", false; "arbitrary req and semantic req")]
        #[test_case("Arbitrary", "Arbitrary", true; "identical arbitrary reqs")]
        #[test_case("Arbitrary", "arbitrary", false; "differently cased arbitrary reqs")]
        #[test_case("foo", "bar", false; "different arbitrary reqs")]
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
