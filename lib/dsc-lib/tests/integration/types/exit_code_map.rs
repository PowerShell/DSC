// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Concisely defines a static map for use in tests. For example, create a map with exit codes `0`
/// and `1` as:
///
/// ```ignore
/// define_static_map!(CUSTOM_MAP: {
///     0 => "successful operation",
///     1 => "unhandled failure",
///     2 => "unauthorized operation"
/// });
/// ```
///
/// Which expands to:
///
/// ```ignore
/// static CUSTOM_MAP: std::sync::LazyLock<dsc_lib::types::ExitCodesMap> = std::sync::LazyLock::new(|| {
///     let mut map = dsc_lib::types::ExitCodesMap::new();
///     map.insert(dsc_lib::types::ExitCode::new(0), "successful operation".to_string());
///     map.insert(dsc_lib::types::ExitCode::new(1), "unhandled failure".to_string());
///     map.insert(dsc_lib::types::ExitCode::new(2), "unauthorized operation".to_string());
/// });
/// ```
///
/// This macro is only intended for use in these integration tests.
macro_rules! define_static_map {
    ( $name:ident: {$( $key:expr => $value:expr ),* $(,)?} ) => {
        static $name: std::sync::LazyLock<dsc_lib::types::ExitCodesMap> = std::sync::LazyLock::new(|| {
            let mut map = dsc_lib::types::ExitCodesMap::new();
            $(
                map.insert(dsc_lib::types::ExitCode::new($key), $value.to_string());
            )*
            map
        });
    }
}

#[cfg(test)]
mod methods {
    use std::sync::LazyLock;

    use dsc_lib::types::ExitCodesMap;
    use test_case::test_case;

    #[test]
    fn new() {
        let _ = ExitCodesMap::new();
    }

    #[test]
    fn with_capacity() {
        let _ = ExitCodesMap::with_capacity(1);
    }

    define_static_map!(CUSTOM_MAP: {
        0 => "okay",
        10 => "oops"
    });

    static DEFAULT_FAILURE_DESC: LazyLock<String> = LazyLock::new(|| ExitCodesMap::default().get_code(1).cloned().unwrap());

    #[test_case(0 => matches Some(_); "returns description for a defined exit code")]
    #[test_case(100 => matches None; "returns none for an undefined exit code")]
    fn get_code(code: i32) -> Option<String> {
        ExitCodesMap::default().get_code(code).cloned()
    }

    #[test_case(&*CUSTOM_MAP, 0, "okay"; "returns description for success from non-default map")]
    #[test_case(&*CUSTOM_MAP, 10, "oops"; "returns description for failure from non-default map")]
    #[test_case(&*CUSTOM_MAP, 5, DEFAULT_FAILURE_DESC.as_str())]
    fn get_code_or_default(map: &ExitCodesMap, code: i32, expected: &str) {
        pretty_assertions::assert_eq!(
            map.get_code_or_default(code).as_str(),
            expected
        )
    }

    #[test_case(ExitCodesMap::new() => true; "map without codes returns true")]
    #[test_case(ExitCodesMap::default() => false; "map with any codes returns false")]
    fn is_empty(map: ExitCodesMap) -> bool {
        map.is_empty()
    }

    #[test_case(&ExitCodesMap::default() => true; "default map returns true")]
    #[test_case(&ExitCodesMap::new() => false; "empty map returns false")]
    #[test_case(&*CUSTOM_MAP => false; "non-default map returns false")]
    fn is_default(map: &ExitCodesMap) -> bool {
        map.is_default()
    }

    #[test_case(&ExitCodesMap::default() => true; "default map returns true")]
    #[test_case(&ExitCodesMap::new() => true; "empty map returns true")]
    #[test_case(&*CUSTOM_MAP => false; "non-default map returns false")]
    fn is_empty_or_default(map: &ExitCodesMap) -> bool {
        map.is_empty_or_default()
    }
}

#[cfg(test)]
mod schema {
    use std::sync::LazyLock;

    use dsc_lib::types::ExitCodesMap;
    use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    use jsonschema::Validator;
    use regex::Regex;
    use schemars::{schema_for, Schema};
    use serde_json::{json, Value};
    use test_case::test_case;

    static SCHEMA: LazyLock<Schema> = LazyLock::new(|| schema_for!(ExitCodesMap));
    static PROPERTY_NAMES_SUBSCHEMA: LazyLock<Schema> = LazyLock::new(|| {
        (&*SCHEMA).get_keyword_as_subschema("propertyNames").unwrap().clone()
    });
    static VALIDATOR: LazyLock<Validator> =
        LazyLock::new(|| Validator::new((&*SCHEMA).as_value()).unwrap());
    static KEYWORD_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\w+(\.\w+)+$").expect("pattern is valid"));

    #[test_case("title", &*SCHEMA)]
    #[test_case("description", &*SCHEMA)]
    #[test_case("markdownDescription", &*SCHEMA)]
    #[test_case("patternErrorMessage", &*PROPERTY_NAMES_SUBSCHEMA)]
    fn has_documentation_keyword(keyword: &str, schema: &Schema) {
        let value = schema
            .get_keyword_as_str(keyword)
            .expect(format!("expected keyword '{keyword}' to be defined").as_str());

        assert!(
            !(&*KEYWORD_PATTERN).is_match(value),
            "Expected keyword '{keyword}' to be defined in translation, but was set to i18n key '{value}'"
        );
    }

    #[test_case(&json!({ "0": "okay"}) => true; "object with zero exit code and description is valid")]
    #[test_case(&json!({ "1": "oops"}) => true; "object with positive integer exit code and description is valid")]
    #[test_case(&json!({ "-1": "oops"}) => true; "object with negative integer exit code and description is valid")]
    #[test_case(&json!({ "0": "okay", "-1": "oops"}) => true; "object with multiple exit code and description pairs is valid")]
    #[test_case(&json!({}) => false; "empty object value is invalid")]
    #[test_case(&json!({"invalid": "map"}) => false; "object with non-parseable key is invalid")]
    #[test_case(&json!("0") => false; "string value is invalid")]
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
    use dsc_lib::types::ExitCodesMap;
    use serde_json::{json, Value};
    use test_case::test_case;

    define_static_map!(CUSTOM_MAP: {
        0 => "okay",
        1 => "oops",
    });

    #[test_case(&ExitCodesMap::new(), json!({}); "can serialize empty map")]
    #[test_case(&ExitCodesMap::default(), json!({"0": "Success", "1": "Error"}); "can serialize default map")]
    #[test_case(&*CUSTOM_MAP, json!({"0": "okay", "1": "oops"}); "can serialize custom map")]
    fn serializing(map: &ExitCodesMap, expected: Value) {
        let actual = serde_json::to_value(map.clone())
            .expect("serialization should never fail");

        pretty_assertions::assert_eq!(
            actual,
            expected
        )
    }

    #[test_case(json!({}), Some(&ExitCodesMap::new()); "can deserialize empty object")]
    #[test_case(json!({"0": "Success", "1": "Error"}), Some(&ExitCodesMap::default()); "can deserialize object containing default map")]
    #[test_case(json!({"0": "okay", "1": "oops"}), Some(&*CUSTOM_MAP); "can deserialize valid object")]
    #[test_case(json!({"0": "okay", "foo": "fails"}), None; "object with invalid key fails to deserialize")]
    #[test_case(json!({"0": "okay", "1": false}), None; "object with invalid value fails to deserialize")]
    #[test_case(json!(true), None; "boolean value is invalid")]
    #[test_case(json!(1), None; "integer value is invalid")]
    #[test_case(json!(1.2), None; "float value is invalid")]
    #[test_case(json!("okay"), None; "string value is invalid")]
    #[test_case(json!(["okay"]), None; "array value is invalid")]
    #[test_case(serde_json::Value::Null, None; "null value is invalid")]
    fn deserializing(value: Value, expected: Option<&ExitCodesMap>) {
        match serde_json::from_value::<ExitCodesMap>(value.clone()) {
            Ok(actual_map) => match expected {
                None => panic!(
                    "expected value {value:?} to fail deserializing but is {actual_map:?}"
                ),
                Some(expected_map) => {
                    pretty_assertions::assert_eq!(&actual_map, expected_map);
                }
            },
            Err(_) => match expected {
                None => {},
                Some(expected_map) => panic!(
                    "expected value {value:?} to deserialize to {expected_map:?}"
                ),
            },
        }
    }
}

#[cfg(test)]
mod traits {
    #[cfg(test)]
    mod default {
        use dsc_lib::types::ExitCodesMap;

        #[test]
        fn default() {
            let actual = ExitCodesMap::default();

            pretty_assertions::assert_eq!(actual.len(), 2);

            assert!(actual.get_code(0).is_some());
            assert!(actual.get_code(1).is_some());
        }
    }

    #[cfg(test)]
    mod as_ref {
        use dsc_lib::types::ExitCodesMap;


        #[test]
        fn exit_codes_map() {
            let _: &ExitCodesMap = ExitCodesMap::default().as_ref();
        }
    }

    #[cfg(test)]
    mod deref {
        use dsc_lib::types::ExitCodesMap;

        #[test]
        fn hashmap_exit_code_string() {
            let map = ExitCodesMap::default();

            pretty_assertions::assert_eq!(map.len(), 2);
        }
    }

    #[cfg(test)]
    mod deref_mut {
        use dsc_lib::types::{ExitCode, ExitCodesMap};

        #[test]
        fn hashmap_exit_code_string() {
            let mut map = ExitCodesMap::new();

            map.insert(ExitCode::new(2), "new error".to_string());
        }
    }
}
