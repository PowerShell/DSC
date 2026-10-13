// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    use dsc_lib::{dscerror::DscError, types::Tag};
    use test_case::test_case;

    #[test_case("macos" => matches Ok(_); "lowercase ascii string is valid")]
    #[test_case("MACOS" => matches Ok(_); "uppercase ascii string is valid")]
    #[test_case("macOS" => matches Ok(_); "mixed case ascii string is valid")]
    #[test_case("_" => matches Ok(_); "single underscore string is valid")]
    #[test_case("123" => matches Ok(_); "digit string is valid")]
    #[test_case("abc_123_DEF" => matches Ok(_); "mixed valid characters string is valid")]
    #[test_case("a.b" => matches Err(_); "string with invalid character raises error")]
    #[test_case("" => matches Err(_); "empty string raises error")]
    fn new(text: &str) -> Result<Tag, DscError> {
        Tag::new(text)
    }

    #[test_case("macos" => matches Ok(_); "lowercase ascii string is valid")]
    #[test_case("MACOS" => matches Ok(_); "uppercase ascii string is valid")]
    #[test_case("macOS" => matches Ok(_); "mixed case ascii string is valid")]
    #[test_case("_" => matches Ok(_); "single underscore string is valid")]
    #[test_case("123" => matches Ok(_); "digit string is valid")]
    #[test_case("abc_123_DEF" => matches Ok(_); "mixed valid characters string is valid")]
    #[test_case("a.b" => matches Err(_); "string with invalid character raises error")]
    #[test_case("" => matches Err(_); "empty string raises error")]
    fn validate(text: &str) -> Result<(), DscError> {
        Tag::validate(text)
    }
}

#[cfg(test)]
mod schema {
    use std::sync::LazyLock;

    use dsc_lib::types::Tag;
    use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    use jsonschema::Validator;
    use regex::Regex;
    use schemars::{schema_for, Schema};
    use serde_json::{json, Value};
    use test_case::test_case;

    static SCHEMA: LazyLock<Schema> = LazyLock::new(|| schema_for!(Tag));
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

    #[test_case(&json!("macos") => true; "lowercase ascii string value is valid")]
    #[test_case(&json!("MACOS") => true; "uppercase ascii string value is valid")]
    #[test_case(&json!("macOS") => true; "mixed case ascii string value is valid")]
    #[test_case(&json!("_") => true; "single underscore string value is valid")]
    #[test_case(&json!("123") => true; "digit string value is valid")]
    #[test_case(&json!("abc_123_DEF") => true; "mixed valid characters string value is valid")]
    #[test_case(&json!("a.b") => false; "string value with invalid character is invalid")]
    #[test_case(&json!("") => false; "empty string value is invalid")]
    #[test_case(&json!(true) => false; "boolean value is invalid")]
    #[test_case(&json!(1) => false; "integer value is invalid")]
    #[test_case(&json!(1.2) => false; "float value is invalid")]
    #[test_case(&json!({"tag": "macOS"}) => false; "object value is invalid")]
    #[test_case(&json!(["macOS"]) => false; "array value is invalid")]
    #[test_case(&serde_json::Value::Null => false; "null value is invalid")]
    fn validation(input_json: &Value) -> bool {
        (&*VALIDATOR).validate(input_json).is_ok()
    }
}

#[cfg(test)]
mod serde {
    use serde_json::{Value, json};
    use dsc_lib::types::Tag;
    use test_case::test_case;

    #[test_case("macos"; "lowercase ascii tag")]
    #[test_case("MACOS"; "uppercase ascii tag")]
    #[test_case("macOS"; "mixed case ascii tag")]
    #[test_case("_"; "single underscore tag")]
    #[test_case("123"; "digit tag")]
    #[test_case("abc_123_DEF"; "mixed valid characters tag")]
    fn serializing(tag: &str) {
        let actual = serde_json::to_string(
            &Tag::new(tag).expect("new should never fail"),
        )
        .expect("serialization should never fail");

        let expected = format!(r#""{tag}""#);

        pretty_assertions::assert_eq!(actual, expected);
    }

    #[test_case(json!("macos") => matches Ok(_); "lowercase ascii string value is valid")]
    #[test_case(json!("MACOS") => matches Ok(_); "uppercase ascii string value is valid")]
    #[test_case(json!("macOS") => matches Ok(_); "mixed case ascii string value is valid")]
    #[test_case(json!("_") => matches Ok(_); "single underscore string value is valid")]
    #[test_case(json!("123") => matches Ok(_); "digit string value is valid")]
    #[test_case(json!("abc_123_DEF") => matches Ok(_); "mixed valid characters string value is valid")]
    #[test_case(json!("a.b") => matches Err(_); "string value with invalid character raises error")]
    #[test_case(json!("") => matches Err(_); "empty string value raises error")]
    #[test_case(json!(true) => matches Err(_); "boolean value raises error")]
    #[test_case(json!(1) => matches Err(_); "integer value raises error")]
    #[test_case(json!(1.2) => matches Err(_); "float value raises error")]
    #[test_case(json!({"tag": "macOS"}) => matches Err(_); "object value raises error")]
    #[test_case(json!(["macOS"]) => matches Err(_); "array value raises error")]
    #[test_case(serde_json::Value::Null => matches Err(_); "null value raises error")]
    fn deserializing(value: Value) -> Result<Tag, serde_json::Error> {
        serde_json::from_value::<Tag>(value)
    }
}

#[cfg(test)]
mod traits {
    #[cfg(test)]
    mod display {
        use dsc_lib::types::Tag;
        use test_case::test_case;

        #[test_case("macos", "macos"; "lowercase ascii string")]
        #[test_case("MACOS", "MACOS"; "uppercase ascii string")]
        #[test_case("macOS", "macOS"; "mixed case ascii string")]
        #[test_case("_", "_"; "single underscore string")]
        #[test_case("123", "123"; "digit string")]
        #[test_case("abc_123_DEF", "abc_123_DEF"; "mixed valid characters string")]
        fn format(tag: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                format!("tag: '{}'", Tag::new(tag).unwrap()),
                format!("tag: '{}'", expected)
            )
        }

        #[test_case("macos", "macos"; "lowercase ascii string")]
        #[test_case("MACOS", "MACOS"; "uppercase ascii string")]
        #[test_case("macOS", "macOS"; "mixed case ascii string")]
        #[test_case("_", "_"; "single underscore string")]
        #[test_case("123", "123"; "digit string")]
        #[test_case("abc_123_DEF", "abc_123_DEF"; "mixed valid characters string")]
        fn to_string(tag: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                Tag::new(tag).unwrap().to_string(),
                expected.to_string()
            )
        }
    }

    #[cfg(test)]
    mod as_ref {
        use dsc_lib::types::Tag;

        #[test]
        fn str() {
            let _: &str = Tag::new("macOS").unwrap().as_ref();
        }
    }

    #[cfg(test)]
    mod deref {
        use dsc_lib::types::Tag;

        #[test]
        fn str() {
            let tag = Tag::new("macOS").unwrap();
            let t = tag.as_ref();

            pretty_assertions::assert_eq!(t.to_lowercase(), "macos".to_string());
            pretty_assertions::assert_eq!(t.to_uppercase(), "MACOS".to_string());
            assert!(t.is_ascii());
        }
    }

    #[cfg(test)]
    mod from_str {
        use std::str::FromStr;

        use dsc_lib::{dscerror::DscError, types::Tag};
        use test_case::test_case;

        #[test_case("macos" => matches Ok(_); "lowercase ascii string is valid")]
        #[test_case("MACOS" => matches Ok(_); "uppercase ascii string is valid")]
        #[test_case("macOS" => matches Ok(_); "mixed case ascii string is valid")]
        #[test_case("_" => matches Ok(_); "single underscore string is valid")]
        #[test_case("123" => matches Ok(_); "digit string is valid")]
        #[test_case("abc_123_DEF" => matches Ok(_); "mixed valid characters string is valid")]
        #[test_case("a.b" => matches Err(_); "string with invalid character raises error")]
        #[test_case("" => matches Err(_); "empty string raises error")]
        fn from_str(text: &str) -> Result<Tag, DscError> {
            Tag::from_str(text)
        }

        #[test_case("macos" => matches Ok(_); "lowercase ascii string is valid")]
        #[test_case("MACOS" => matches Ok(_); "uppercase ascii string is valid")]
        #[test_case("macOS" => matches Ok(_); "mixed case ascii string is valid")]
        #[test_case("_" => matches Ok(_); "single underscore string is valid")]
        #[test_case("123" => matches Ok(_); "digit string is valid")]
        #[test_case("abc_123_DEF" => matches Ok(_); "mixed valid characters string is valid")]
        #[test_case("a.b" => matches Err(_); "string with invalid character raises error")]
        #[test_case("" => matches Err(_); "empty string raises error")]
        fn parse(text: &str) -> Result<Tag, DscError> {
            text.parse()
        }
    }

    #[cfg(test)]
    mod try_from {
        use dsc_lib::{dscerror::DscError, types::Tag};
        use test_case::test_case;


        #[test_case("macos" => matches Ok(_); "lowercase ascii string is valid")]
        #[test_case("MACOS" => matches Ok(_); "uppercase ascii string is valid")]
        #[test_case("macOS" => matches Ok(_); "mixed case ascii string is valid")]
        #[test_case("_" => matches Ok(_); "single underscore string is valid")]
        #[test_case("123" => matches Ok(_); "digit string is valid")]
        #[test_case("abc_123_DEF" => matches Ok(_); "mixed valid characters string is valid")]
        #[test_case("a.b" => matches Err(_); "string with invalid character raises error")]
        #[test_case("" => matches Err(_); "empty string raises error")]
        fn string(text: &str) -> Result<Tag, DscError> {
            Tag::try_from(text.to_string())
        }

        #[test_case("macos" => matches Ok(_); "lowercase ascii string is valid")]
        #[test_case("MACOS" => matches Ok(_); "uppercase ascii string is valid")]
        #[test_case("macOS" => matches Ok(_); "mixed case ascii string is valid")]
        #[test_case("_" => matches Ok(_); "single underscore string is valid")]
        #[test_case("123" => matches Ok(_); "digit string is valid")]
        #[test_case("abc_123_DEF" => matches Ok(_); "mixed valid characters string is valid")]
        #[test_case("a.b" => matches Err(_); "string with invalid character raises error")]
        #[test_case("" => matches Err(_); "empty string raises error")]
        fn str(text: &str) -> Result<Tag, DscError> {
            Tag::try_from(text)
        }
    }

    #[cfg(test)]
    mod into {
        use dsc_lib::types::Tag;

        #[test]
        fn string() {
            let _: String = Tag::new("tag").unwrap().into();
        }
    }

    #[cfg(test)]
    mod partial_eq {
        use dsc_lib::types::Tag;
        use test_case::test_case;

        #[test_case("macOS", "macOS", true; "identical tags")]
        #[test_case("macOS", "macos", true; "differing case tags")]
        #[test_case("macOS", "Linux", false; "different text tags")]
        fn tag(lhs: &str, rhs: &str, should_be_equal: bool) {
            if should_be_equal {
                pretty_assertions::assert_eq!(
                    Tag::new(lhs).unwrap(),
                    Tag::new(rhs).unwrap()
                )
            } else {
                pretty_assertions::assert_ne!(
                    Tag::new(lhs).unwrap(),
                    Tag::new(rhs).unwrap()
                )
            }
        }

        #[test_case("macOS", "macOS", true; "tag and identical string")]
        #[test_case("macOS", "macos", true; "tag and differing case string")]
        #[test_case("macOS", "Linux", false; "tag and different text string")]
        fn string(tag_string_slice: &str, string_slice: &str, should_be_equal: bool) {
            let tag = Tag::new(tag_string_slice).unwrap();
            let string = string_slice.to_string();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                tag == string,
                should_be_equal,
                "expected comparison of {tag} and {string} to be {should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string == tag,
                should_be_equal,
                "expected comparison of {string} and {tag} to be {should_be_equal}"
            );
        }

        #[test_case("macOS", "macOS", true; "tag and identical string slice")]
        #[test_case("macOS", "macos", true; "tag and differing case string slice")]
        #[test_case("macOS", "Linux", false; "tag and different text string slice")]
        fn str(tag_string_slice: &str, string_slice: &str, should_be_equal: bool) {
            let tag = Tag::new(tag_string_slice).unwrap();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                tag == string_slice,
                should_be_equal,
                "expected comparison of {tag} and {string_slice} to be {should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string_slice == tag,
                should_be_equal,
                "expected comparison of {string_slice} and {tag} to be {should_be_equal}"
            );
        }
    }

    #[cfg(test)]
    mod partial_ord {
        use std::cmp::Ordering;

        use dsc_lib::types::Tag;
        use test_case::test_case;

        #[test_case("macOS", "macOS", Ordering::Equal; "identical tags are equal")]
        #[test_case("MACOS", "macos", Ordering::Equal; "differently cased tags are equal")]
        #[test_case("AAA", "BBB", Ordering::Less; "lexicographically ordered uppercase tags")]
        #[test_case("bbb", "aaa", Ordering::Greater; "lexicographically ordered lowercase tags")]
        #[test_case("BBB", "aaa", Ordering::Greater; "lexicographically ordered differing case tags as lowercase")]
        fn tag(lhs: &str, rhs:&str, expected_order: Ordering) {
            pretty_assertions::assert_eq!(
                Tag::new(lhs)
                    .expect("parsing for lhs should not fail")
                    .partial_cmp(&Tag::new(rhs).expect("parsing for rhs should not fail"))
                    .expect("comparison should always be an ordering"),
                expected_order,
                "expected '{lhs}' compared to '{rhs}' to be {expected_order:#?}"
            )
        }

        #[test_case("macOS", "macOS", Ordering::Equal; "tag and identical string are equal")]
        #[test_case("MACOS", "macos", Ordering::Equal; "tag and differently cased string are equal")]
        #[test_case("AAA", "BBB", Ordering::Less; "lexicographically ordered uppercase tag and string")]
        #[test_case("bbb", "aaa", Ordering::Greater; "lexicographically ordered lowercase tag and string")]
        #[test_case("BBB", "aaa", Ordering::Greater; "lexicographically ordered differing case tag and string")]
        fn string(tag_text: &str, string_text: &str, expected_order: Ordering) {
            let tag = Tag::new(tag_text).unwrap();
            let string = string_text.to_string();

            // Test comparison bidirectionally
            pretty_assertions::assert_eq!(
                tag.partial_cmp(&string).unwrap(),
                expected_order,
                "expected comparison of {tag} and {string} to be #{expected_order:#?}"
            );

            let expected_inverted_order = expected_order.reverse();

            pretty_assertions::assert_eq!(
                string.partial_cmp(&tag).unwrap(),
                expected_inverted_order,
                "expected comparison of {tag} and {string} to be #{expected_inverted_order:#?}"
            )
        }

        #[test_case("macOS", "macOS", Ordering::Equal; "tag and identical string are equal")]
        #[test_case("MACOS", "macos", Ordering::Equal; "tag and differently cased string are equal")]
        #[test_case("AAA", "BBB", Ordering::Less; "lexicographically ordered uppercase tag and string")]
        #[test_case("bbb", "aaa", Ordering::Greater; "lexicographically ordered lowercase tag and string")]
        #[test_case("BBB", "aaa", Ordering::Greater; "lexicographically ordered differing case tag and string")]
        fn str(tag_text: &str, string_slice: &str, expected_order: Ordering) {
            let tag = Tag::new(tag_text).unwrap();

            // Test comparison bidirectionally
            pretty_assertions::assert_eq!(
                tag.partial_cmp(&string_slice).unwrap(),
                expected_order,
                "expected comparison of {tag} and {string_slice} to be #{expected_order:#?}"
            );

            let expected_inverted_order = expected_order.reverse();

            pretty_assertions::assert_eq!(
                string_slice.partial_cmp(&tag).unwrap(),
                expected_inverted_order,
                "expected comparison of {tag} and {string_slice} to be #{expected_inverted_order:#?}"
            )
        }
    }
}
