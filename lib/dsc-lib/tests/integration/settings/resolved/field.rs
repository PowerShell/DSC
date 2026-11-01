// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(super) mod fixture {
    use std::fmt::Display;
    use std::ops::Deref;

    #[derive(
        Debug, Clone, PartialEq, Eq,
        serde::Serialize, serde::Deserialize,
        schemars::JsonSchema
    )]
    pub(crate) struct TestingField(pub String);
    pub(crate) type Resolved = dsc_lib::settings::DscSettingsResolvedField<TestingField>;

    impl AsRef<str> for TestingField {
        fn as_ref(&self) -> &str {
            self.0.as_ref()
        }
    }
    impl Deref for TestingField {
        type Target = str;
        fn deref(&self) -> &Self::Target {
            self.0.deref()
        }
    }
    impl Display for TestingField {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    macro_rules! field {
        () => {
            TestingField(String::from("test"))
        };
        ($text:literal) => {
            TestingField(String::from($text))
        };
    }
    pub(crate) use field;
    macro_rules! resolved_field {
        () => {
            Resolved::for_code_default(field!())
        };
        ($text:literal) => {
            Resolved::for_code_default(field!($text))
        };
        ($text:literal, $scope:ident) => {
            Resolved::new(field!($text), dsc_lib::settings::DscSettingsScope::$scope)
        };
        ($scope:ident) => {
            Resolved::new(field!(), dsc_lib::settings::DscSettingsScope::$scope)
        }
    }
    pub(crate) use resolved_field;
}


#[cfg(test)]
mod methods {
    use test_case::test_case;

    use dsc_lib::settings::DscSettingsScope;
    use super::fixture::*;

    #[test]
    fn new() {
        Resolved::new(TestingField(String::from("test")), DscSettingsScope::Default);
    }

    #[test]
    fn for_code_default() {
        let resolved = Resolved::for_code_default(
            TestingField(String::from("test"))
        );
        pretty_assertions::assert_eq!(resolved.scope, DscSettingsScope::Default);
        pretty_assertions::assert_eq!(resolved.value.as_ref(), "test");
    }

    #[test]
    fn for_environment() {
        let resolved = Resolved::for_environment(
            TestingField(String::from("test"))
        );
        pretty_assertions::assert_eq!(resolved.scope, DscSettingsScope::Environment);
        pretty_assertions::assert_eq!(resolved.value.as_ref(), "test");
    }

    #[test]
    fn for_command_line() {
        let resolved = Resolved::for_command_line(
            TestingField(String::from("test"))
        );
        pretty_assertions::assert_eq!(resolved.scope, DscSettingsScope::CommandLine);
        pretty_assertions::assert_eq!(resolved.value.as_ref(), "test");
    }

    #[test]
    fn for_machine() {
        let resolved = Resolved::for_machine(
            TestingField(String::from("test"))
        );
        pretty_assertions::assert_eq!(resolved.scope, DscSettingsScope::Machine);
        pretty_assertions::assert_eq!(resolved.value.as_ref(), "test");
    }

    #[test]
    fn for_user() {
        let resolved = Resolved::for_user(
            TestingField(String::from("test"))
        );
        pretty_assertions::assert_eq!(resolved.scope, DscSettingsScope::User);
        pretty_assertions::assert_eq!(resolved.value.as_ref(), "test");
    }

    #[test]
    fn for_workspace() {
        let resolved = Resolved::for_workspace(
            TestingField(String::from("test"))
        );
        pretty_assertions::assert_eq!(resolved.scope, DscSettingsScope::Workspace);
        pretty_assertions::assert_eq!(resolved.value.as_ref(), "test");
    }

    #[test]
    fn for_policy() {
        let resolved = Resolved::for_policy(
            TestingField(String::from("test"))
        );
        pretty_assertions::assert_eq!(resolved.scope, DscSettingsScope::Policy);
        pretty_assertions::assert_eq!(resolved.value.as_ref(), "test");
    }

    #[test_case(resolved_field!("test", Default) => false; "default scope")]
    #[test_case(resolved_field!("test", Policy) => true; "policy scope")]
    #[test_case(resolved_field!("test", Machine) => false; "machine scope")]
    #[test_case(resolved_field!("test", User) => false; "user scope")]
    #[test_case(resolved_field!("test", Workspace) => false; "workspace scope")]
    #[test_case(resolved_field!("test", Environment) => false; "environment scope")]
    #[test_case(resolved_field!("test", CommandLine) => false; "command line scope")]
    fn is_policy(setting: Resolved) -> bool {
        setting.is_policy()
    }
}

#[cfg(test)]
mod schema {
    use std::sync::LazyLock;

    use super::fixture::*;

    use dsc_lib::{schemas::schema_utility_extensions::SchemaUtilityExtensions};
    use jsonschema::Validator;
    use regex::Regex;
    use schemars::{schema_for, Schema};
    use serde_json::{json, Value};
    use test_case::test_case;

    static ROOT_SCHEMA: LazyLock<Schema> = LazyLock::new(|| schema_for!(Resolved));
    static VALIDATOR: LazyLock<Validator> =
        LazyLock::new(|| Validator::new((&*ROOT_SCHEMA).as_value()).unwrap());
    static KEYWORD_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\w+(\.\w+)+$").expect("pattern is valid"));

    #[test]fn is_valid_for_meta_schema() {
        let schema = ROOT_SCHEMA.as_value();
        let result = jsonschema::meta::validate(schema);
        assert!(
            result.is_ok(),
            "expected schema to be valid but got error: {:#?}",
            result.unwrap_err()
        );
    }

    #[test_case("title", &*ROOT_SCHEMA; "title")]
    #[test_case("description", &*ROOT_SCHEMA; "description")]
    #[test_case("markdownDescription", &*ROOT_SCHEMA; "markdown description")]
    fn has_documentation_keyword(keyword: &str, schema: &Schema) {
        match keyword {
            "markdownEnumDescriptions" => {
                schema.get_keyword_as_array(keyword)
                    .expect(&format!("expected keyword '{keyword}' to be defined as an array"))
                    .iter()
                    .for_each(|value| {
                        let string_value = value.as_str()
                            .expect("expected each item in array value to be a string");
                        assert!(
                            !(&*KEYWORD_PATTERN).is_match(string_value),
                            "expected each item in `{}` to be defined in translation, but was set to i18n key '{}'",
                            keyword,
                            value,
                        )
                    })
            }
            _ => {
                let value = schema
                    .get_keyword_as_str(keyword)
                    .expect(&format!(
                        "expected keyword '{keyword}' to be defined in schema: {}",
                        serde_json::to_string_pretty(schema).unwrap()
                    ));

                assert!(
                    !(&*KEYWORD_PATTERN).is_match(value),
                    "Expected keyword '{keyword}' to be defined in translation, but was set to i18n key '{value}'",
                );
            }
        }
    }

    #[test]
    fn value_schema_is_for_wrapped_type() {
        let value_ref = (&*ROOT_SCHEMA)
            .get_property_subschema("value")
            .expect("expected 'value' property to have a subschema")
            .get_keyword_as_str("$ref")
            .expect("expected '$ref' keyword to be defined");
        let expected_ref = "#/$defs/TestingField";
        pretty_assertions::assert_eq!(value_ref, expected_ref);
    }

    #[test]
    fn scope_schema_is_for_setings_scope() {
        let scope_ref = (&*ROOT_SCHEMA)
            .get_property_subschema("scope")
            .expect("expected 'scope' property to have a subschema")
            .get_keyword_as_str("$ref")
            .expect("expected '$ref' keyword to be defined");
        let expected_ref = "#/$defs/DscSettingsScope";
        pretty_assertions::assert_eq!(scope_ref, expected_ref);
    }

    #[test_case(&json!({"value": "test", "scope": "default"}), true; "valid value and default scope")]
    #[test_case(&json!({"value": "test", "scope": "machine"}), true; "valid value and machine scope")]
    #[test_case(&json!({"value": "test", "scope": "user"}), true; "valid value and user scope")]
    #[test_case(&json!({"value": "test", "scope": "workspace"}), true; "valid value and workspace scope")]
    #[test_case(&json!({"value": "test", "scope": "environment"}), true; "valid value and environment scope")]
    #[test_case(&json!({"value": "test", "scope": "cli"}), true; "valid value and cli scope")]
    #[test_case(&json!({"value": "test", "scope": "unknown"}), false; "valid value and unknown scope")]
    #[test_case(&json!({"value": ["invalid"], "scope": "default"}), false; "invalid value and valid scope")]
    fn validation(input_json: &Value, expected_valid: bool) {
        let result = (&*VALIDATOR).validate(input_json);
        if expected_valid {
            assert!(result.is_ok(), "expected input to be valid but got error {:#?}", result.unwrap_err());
        } else {
            assert!(result.is_err(), "expected input to be invalid: {:#?}", input_json);
        }
    }
}

#[cfg(test)]
mod serde {
    use serde_json::{json, Value};
    use test_case::test_case;
    use super::fixture::*;

    #[test_case(&resolved_field!("test", Default) => json!({"value": "test", "scope": "default"}); "default scope")]
    #[test_case(&resolved_field!("test", Policy) => json!({"value": "test", "scope": "policy"}); "policy scope")]
    #[test_case(&resolved_field!("test", Machine) => json!({"value": "test", "scope": "machine"}); "machine scope")]
    #[test_case(&resolved_field!("test", User) => json!({"value": "test", "scope": "user"}); "user scope")]
    #[test_case(&resolved_field!("test", Workspace) => json!({"value": "test", "scope": "workspace"}); "workspace scope")]
    #[test_case(&resolved_field!("test", Environment) => json!({"value": "test", "scope": "environment"}); "environment scope")]
    #[test_case(&resolved_field!("test", CommandLine) => json!({"value": "test", "scope": "cli"}); "command line scope")]
    fn serializing(value: &Resolved) -> Value {
        serde_json::to_value(value)
            .expect("serialization should never fail")
    }


    #[test_case(&json!({"value": false, "scope": "default"}); "invalid value")]
    #[test_case(&json!({"value": "test", "scope": "unknown"}); "invalid scope")]
    fn deserialize_invalid(input_json: &Value) {
        let result: Result<Resolved, _> = serde_json::from_value(input_json.clone());
        assert!(result.is_err(), "expected input to be invalid: {:#?}", input_json);
    }

    #[test_case(&json!({"value": "test", "scope": "default"}), &resolved_field!("test", Default); "default scope")]
    #[test_case(&json!({"value": "test", "scope": "policy"}), &resolved_field!("test", Policy); "policy scope")]
    #[test_case(&json!({"value": "test", "scope": "machine"}), &resolved_field!("test", Machine); "machine scope")]
    #[test_case(&json!({"value": "test", "scope": "user"}), &resolved_field!("test", User); "user scope")]
    #[test_case(&json!({"value": "test", "scope": "workspace"}), &resolved_field!("test", Workspace); "workspace scope")]
    #[test_case(&json!({"value": "test", "scope": "environment"}), &resolved_field!("test", Environment); "environment scope")]
    #[test_case(&json!({"value": "test", "scope": "cli"}), &resolved_field!("test", CommandLine); "command line scope")]
    fn deserialize_valid(input_json: &Value, expected_value: &Resolved) {
        let result: Result<Resolved, _> = serde_json::from_value(input_json.clone());
        assert!(result.is_ok(), "expected input to be valid but got error {:#?}", result.unwrap_err());
        pretty_assertions::assert_eq!(&result.unwrap(), expected_value, "deserialized value does not match expected");
    }
}
