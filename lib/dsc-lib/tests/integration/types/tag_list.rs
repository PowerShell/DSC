// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    use dsc_lib::types::{Tag, TagList};
    use test_case::test_case;

    #[test]
    fn new() {
        assert_eq!(TagList::new().len(), 0);
    }

    #[test]
    fn with_capacity() {
        assert!(TagList::with_capacity(10).capacity() >= 10)
    }

    #[test_case(vec![] => true; "empty list returns true")]
    fn is_empty(tags: Vec<Tag>) -> bool {
        let mut list = TagList::with_capacity(tags.len());

        for tag in tags {
            list.insert(tag);
        }

        list.is_empty()
    }
}

#[cfg(test)]
mod schema {
    use std::sync::LazyLock;

    use dsc_lib::types::TagList;
    use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    use jsonschema::Validator;
    use regex::Regex;
    use schemars::{schema_for, Schema};
    use serde_json::{json, Value};
    use test_case::test_case;

    static SCHEMA: LazyLock<Schema> = LazyLock::new(|| schema_for!(TagList));
    static VALIDATOR: LazyLock<Validator> =
        LazyLock::new(|| Validator::new((&*SCHEMA).as_value()).unwrap());
    static KEYWORD_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\w+(\.\w+)+$").expect("pattern is valid"));

    #[test_case("title")]
    #[test_case("description")]
    #[test_case("markdownDescription")]
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

    #[test_case(&json!(["macos"]) => true; "array containing single lowercase ascii string value is valid")]
    #[test_case(&json!(["MACOS"]) => true; "array containing single uppercase ascii string value is valid")]
    #[test_case(&json!(["macOS"]) => true; "array containing single mixed case ascii string value is valid")]
    #[test_case(&json!(["_"]) => true; "array containing single underscore string value is valid")]
    #[test_case(&json!(["123"]) => true; "array containing single digit string value is valid")]
    #[test_case(&json!(["abc_123_DEF"]) => true; "array containing single mixed valid characters string value is valid")]
    #[test_case(&json!(["a.b"]) => false; "array containing single string value with invalid character raises error")]
    #[test_case(&json!([""]) => false; "array containing single empty string value raises error")]
    #[test_case(&json!(["Linux", "macOS"]) => true; "array containing multiple unique string values is valid")]
    #[test_case(&json!(["macOS", "a.b"]) => false; "array containing valid string values and invalid string values is invalid")]
    #[test_case(&json!(["macOS", "macOS"]) => false; "array containing identical string values is invalid")]
    #[test_case(&json!(["MACOS", "macos"]) => true; "array containing equal string values with different casing values is valid")]
    #[test_case(&json!("macOS") => false; "string value is invalid")]
    #[test_case(&json!(true) => false; "boolean value is invalid")]
    #[test_case(&json!(1) => false; "integer value is invalid")]
    #[test_case(&json!(1.2) => false; "float value is invalid")]
    #[test_case(&json!({"tag": "macOS"}) => false; "object value is invalid")]
    #[test_case(&serde_json::Value::Null => false; "null value is invalid")]
    fn validation(input_json: &Value) -> bool {
        (&*VALIDATOR).validate(input_json).is_ok()
    }
}

#[cfg(test)]
mod serde {
    use serde_json::{Value, json};
    use dsc_lib::types::{Tag, TagList};
    use test_case::test_case;

    #[test_case(&vec!["macos"], json!(["macos"]); "list containing single lowercase ascii tag")]
    #[test_case(&vec!["MACOS"], json!(["MACOS"]); "list containing single uppercase ascii tag")]
    #[test_case(&vec!["macOS"], json!(["macOS"]); "list containing single mixed case ascii tag")]
    #[test_case(&vec!["_"], json!(["_"]); "list containing single underscore tag")]
    #[test_case(&vec!["123"], json!(["123"]); "list containing single digit tag")]
    #[test_case(&vec!["abc_123_DEF"], json!(["abc_123_DEF"]); "list containing single mixed valid characters tag")]
    #[test_case(&vec!["Linux", "macOS"], json!(["Linux", "macOS"]); "list containing multiple unique tags")]
    fn serializing(tags: &Vec<&str>, expected: Value) {
        let mut set = TagList::new();
        for tag in tags {
            set.insert(Tag::new(tag).unwrap());
        }

        let actual = serde_json::to_value(&set).expect("serialization should never fail");
        pretty_assertions::assert_eq!(actual, expected);
    }

    #[test_case(json!(["macos"]) => matches Ok(_); "array containing single lowercase ascii string value is valid")]
    #[test_case(json!(["MACOS"]) => matches Ok(_); "array containing single uppercase ascii string value is valid")]
    #[test_case(json!(["macOS"]) => matches Ok(_); "array containing single mixed case ascii string value is valid")]
    #[test_case(json!(["_"]) => matches Ok(_); "array containing single underscore string value is valid")]
    #[test_case(json!(["123"]) => matches Ok(_); "array containing single digit string value is valid")]
    #[test_case(json!(["abc_123_DEF"]) => matches Ok(_); "array containing single mixed valid characters string value is valid")]
    #[test_case(json!(["a.b"]) => matches Err(_); "array containing single string value with invalid character raises error")]
    #[test_case(json!([""]) => matches Err(_); "array containing single empty string value raises error")]
    #[test_case(json!(["Linux", "macOS"]) => matches Ok(_); "array containing multiple unique string values is valid")]
    #[test_case(json!(["macOS", "a.b"]) => matches Err(_); "array containing valid string values and invalid string values raises error")]
    // This test case shows a different behavior for deserialization vs JSON Schema validation;
    // Without implementing custom deserialization logic, we can't forbid (instead of ignore)
    // duplicate values. Since all data in DSC is validated against the JSON Schema prior to
    // deserialization and we have documented this behavior, this test shows how the behavior
    // differs so we don't lose track of it.
    #[test_case(json!(["macOS", "macOS"]) => matches Ok(_); "array containing identical string values is valid")]
    #[test_case(json!(["MACOS", "macos"]) => matches Ok(_); "array containing equal string values with different casing values is valid")]
    #[test_case(json!("macOS") => matches Err(_); "string value raises error")]
    #[test_case(json!(true) => matches Err(_); "boolean value raises error")]
    #[test_case(json!(1) => matches Err(_); "integer value raises error")]
    #[test_case(json!(1.2) => matches Err(_); "float value raises error")]
    #[test_case(json!({"tag": "macOS"}) => matches Err(_); "object value raises error")]
    #[test_case(serde_json::Value::Null => matches Err(_); "null value raises error")]
    fn deserializing(value: Value) -> Result<TagList, serde_json::Error> {
        serde_json::from_value::<TagList>(value)
    }
}

#[cfg(test)]
mod traits {
    #[cfg(test)]
    mod as_ref {
        use std::collections::HashSet;

        use dsc_lib::types::{Tag, TagList};

        #[test]
        fn hashset_tag() {
            let _: &HashSet<Tag> = TagList::new().as_ref();
        }
    }

    #[cfg(test)]
    mod borrow {
        use std::{borrow::Borrow, collections::HashSet};

        use dsc_lib::types::{Tag, TagList};

        #[test]
        fn hashset_tag() {
            let _: &HashSet<Tag> = TagList::new().borrow();
        }
    }

    #[cfg(test)]
    mod deref {
        use dsc_lib::types::TagList;

        #[test]
        fn hashset_tag() {
            let tag_list = TagList::new();

            assert!(tag_list.is_empty());
        }
    }

    #[cfg(test)]
    mod deref_mut {
        use dsc_lib::types::{Tag, TagList};

        #[test]
        fn hashset_tag() {
            let mut tag_list = TagList::new();

            tag_list.insert(Tag::new("macOS").unwrap());
        }
    }

    #[cfg(test)]
    mod into_iterator {
        use dsc_lib::types::{Tag, TagList};

        #[test]
        fn tag() {
            let mut list = TagList::new();
            list.insert(Tag::new("macOS").unwrap());

            for tag in list {
                assert!(tag != "");
            }
        }
    }

    mod from_iterator {
        use dsc_lib::types::{Tag, TagList};

        #[test]
        fn tag() {
            let _: TagList = vec![Tag::new("macOS").unwrap()].into_iter().collect();
        }
    }
}
