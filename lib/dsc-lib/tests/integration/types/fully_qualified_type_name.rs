// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    #[cfg(test)]
    mod parse {
        use dsc_lib::types::{FullyQualifiedTypeName, FullyQualifiedTypeNameError};
        use test_case::test_case;
    
        #[test_case("Owner/Name" => matches Ok(_); "valid type name without namespace segments")]
        #[test_case("Owner.Namespace/Name" => matches Ok(_); "valid type name with one namespace segment")]
        #[test_case("Owner.A.B.C/Name" => matches Ok(_); "valid type name with multiple namespace segments")]
        fn valid(text: &str) -> Result<FullyQualifiedTypeName, FullyQualifiedTypeNameError> {
            FullyQualifiedTypeName::parse(text)
        }

        #[test_case("" =>
            FullyQualifiedTypeNameError::EmptyTypeName;
            "empty string"
        )]
        #[test_case("Owner.MissingName" =>
            FullyQualifiedTypeNameError::InvalidTypeName {
                text: "Owner.MissingName".to_string(),
                errors: vec![
                    FullyQualifiedTypeNameError::MissingNameSegment
                ]
            }; "missing forward slash"
        )]
        #[test_case("Owner/" =>
            FullyQualifiedTypeNameError::InvalidTypeName{
                text: "Owner/".to_string(),
                errors: vec![
                    FullyQualifiedTypeNameError::MissingNameSegment
                ]
            }; "missing name segment after forward slash"
        )]
        #[test_case("Owner/Invalid&Name" =>
            FullyQualifiedTypeNameError::InvalidTypeName{
                text: "Owner/Invalid&Name".to_string(),
                errors: vec![
                    FullyQualifiedTypeNameError::InvalidNameSegment {
                        segment_text: "Invalid&Name".to_string()
                    }
                ]
            }; "invalid characters in name segment"
        )]
        #[test_case("Owner.ValidNamespace.Invalid&Namespace.ValidNamespace/Name" =>
            FullyQualifiedTypeNameError::InvalidTypeName{
                text: "Owner.ValidNamespace.Invalid&Namespace.ValidNamespace/Name".to_string(),
                errors: vec![
                    FullyQualifiedTypeNameError::InvalidNamespaceSegment {
                        segment_text: "Invalid&Namespace".to_string()
                    }
                ]
            }; "invalid characters in namespace segment"
        )]
        #[test_case(".Missing.Owner/Name" =>
            FullyQualifiedTypeNameError::InvalidTypeName{
                text: ".Missing.Owner/Name".to_string(),
                errors: vec![
                    FullyQualifiedTypeNameError::EmptyOwnerSegment,
                ]
            }; "empty owner segment before first namespace"
        )]
        #[test_case("/Name" =>
            FullyQualifiedTypeNameError::InvalidTypeName{
                text: "/Name".to_string(),
                errors: vec![
                    FullyQualifiedTypeNameError::EmptyOwnerSegment,
                ]
            }; "empty owner segment before slash"
        )]
        #[test_case("Owner.Empty.Namespace..Segment/Name" =>
            FullyQualifiedTypeNameError::InvalidTypeName{
                text: "Owner.Empty.Namespace..Segment/Name".to_string(),
                errors: vec![
                    FullyQualifiedTypeNameError::EmptyNamespaceSegment {
                        index: 3
                    }
                ]
            }; "empty namespace segment"
        )]
        #[test_case("Owner.Empty.Last.Namespace./Name" =>
            FullyQualifiedTypeNameError::InvalidTypeName{
                text: "Owner.Empty.Last.Namespace./Name".to_string(),
                errors: vec![
                    FullyQualifiedTypeNameError::EmptyNamespaceSegment {
                        index: 4
                    }
                ]
            }; "empty namespace segment at end of namespaces"
        )]
        #[test_case("Invalid&Owner/Name" =>
            FullyQualifiedTypeNameError::InvalidTypeName{
                text: "Invalid&Owner/Name".to_string(),
                errors: vec![
                    FullyQualifiedTypeNameError::InvalidOwnerSegment {
                        segment_text: "Invalid&Owner".to_string()
                    }
                ]
            }; "invalid characters in owner segment"
        )]
        #[test_case("Invalid&Owner.Empty..Namespace.Invalid&Namespace.MissingName" =>
            FullyQualifiedTypeNameError::InvalidTypeName{
                text: "Invalid&Owner.Empty..Namespace.Invalid&Namespace.MissingName".to_string(),
                errors: vec![
                    FullyQualifiedTypeNameError::InvalidOwnerSegment {
                        segment_text: "Invalid&Owner".to_string()
                    },
                    FullyQualifiedTypeNameError::EmptyNamespaceSegment {
                        index: 2
                    },
                    FullyQualifiedTypeNameError::InvalidNamespaceSegment {
                        segment_text: "Invalid&Namespace".to_string()
                    },
                    FullyQualifiedTypeNameError::MissingNameSegment
                ]
            }; "validation reports all errors for an invalid type name"
        )]
        fn invalid(text: &str) -> FullyQualifiedTypeNameError {
            FullyQualifiedTypeName::parse(text).unwrap_err()
        }
    }
}

#[cfg(test)]
mod schema {
    use std::sync::LazyLock;

    use dsc_lib::types::FullyQualifiedTypeName;
    use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    use jsonschema::Validator;
    use regex::Regex;
    use schemars::{schema_for, Schema};
    use serde_json::{json, Value};
    use test_case::test_case;

    static SCHEMA: LazyLock<Schema> = LazyLock::new(|| schema_for!(FullyQualifiedTypeName));
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

    #[test_case(&json!("Owner/Name") => true; "valid type name without namespaces is valid")]
    #[test_case(&json!("Owner.Namespace/Name") => true; "valid type name with one namespace is valid")]
    #[test_case(&json!("Owner.A.B.C/Name") => true; "valid type name with multiple namespaces is valid")]
    #[test_case(&json!("") => false; "empty string is invalid")]
    #[test_case(&json!("Owner.MissingName") => false; "missing forward slash is invalid")]
    #[test_case(&json!("Owner/") => false; "missing name segment after forward slash is invalid")]
    #[test_case(&json!("Owner/Invalid&Name") => false; "invalid characters in name segment are invalid")]
    #[test_case(&json!("Owner.ValidNamespace.Invalid&Namespace.ValidNamespace/Name") => false; "invalid characters in namespace segment are invalid")]
    #[test_case(&json!("Owner.Empty.Namespace..Segment/Name") => false; "empty namespace segment is invalid")]
    #[test_case(&json!("Invalid&Owner/Name") => false; "invalid characters in owner segment are invalid")]
    #[test_case(&json!("Invalid&Owner.Empty..Namespace.Invalid&Namespace.MissingName") => false; "multiple errors are all invalid")]
    #[test_case(&json!(true) => false; "boolean value is invalid")]
    #[test_case(&json!(1) => false; "integer value is invalid")]
    #[test_case(&json!(1.2) => false; "float value is invalid")]
    #[test_case(&json!({"type": "Owner/Name"}) => false; "object value is invalid")]
    #[test_case(&json!(["Owner/Name"]) => false; "array value is invalid")]
    #[test_case(&serde_json::Value::Null => false; "null value is invalid")]
    fn validation(input_json: &Value) -> bool {
        (&*VALIDATOR).validate(input_json).is_ok()
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::types::FullyQualifiedTypeName;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!("Owner/Name") => matches Ok(_); "valid type name without namespace segments deserializes")]
    #[test_case(json!("Owner.Namespace/Name") => matches Ok(_); "valid type name with one namespace segment deserializes")]
    #[test_case(json!("Owner.A.B.C/Name") => matches Ok(_); "valid type name with multiple namespace segments deserializes")]
    #[test_case(json!("invalid_name") => matches Err(_); "invalid type name fails")]
    #[test_case(json!("") => matches Err(_); "empty string fails")]
    #[test_case(json!(true) => matches Err(_); "boolean value fails")]
    #[test_case(json!(1) => matches Err(_); "integer value fails")]
    #[test_case(json!(1.2) => matches Err(_); "float value fails")]
    #[test_case(json!({"type": "Contoso.Example/Resource"}) => matches Err(_); "object value fails")]
    #[test_case(json!(["Contoso.Example/Resource"]) => matches Err(_); "array value fails")]
    #[test_case(serde_json::Value::Null => matches Err(_); "null value fails")]
    fn deserialize(value: Value) -> Result<FullyQualifiedTypeName, serde_json::Error> {
        serde_json::from_value(json!(value))
    }

    #[test_case("Owner/Name" => json!("Owner/Name"); "valid type name without namespace segments serializes")]
    #[test_case("Owner.Namespace/Name" => json!("Owner.Namespace/Name"); "valid type name with one namespace segment serializes")]
    #[test_case("Owner.A.B.C/Name" => json!("Owner.A.B.C/Name"); "valid type name with multiple namespace segments serializes")]
    fn serialize(text: &str) -> Value {
        let instance = FullyQualifiedTypeName::parse(text).unwrap();
        serde_json::to_value(instance).unwrap()
    }
}

#[cfg(test)]
mod traits {
    #[cfg(test)]
    mod default {
        use dsc_lib::types::FullyQualifiedTypeName;

        #[test]
        fn default_is_empty() {
            let instance = FullyQualifiedTypeName::default();
            assert!(instance.is_empty())
        }
    }

    #[cfg(test)]
    mod display {
        use dsc_lib::types::FullyQualifiedTypeName;
        use test_case::test_case;

        #[test_case("Owner/Name", "Owner/Name"; "valid type name without namespace segments")]
        #[test_case("Owner.Namespace/Name", "Owner.Namespace/Name"; "valid type name with one namespace segment")]
        #[test_case("Owner.A.B.C/Name", "Owner.A.B.C/Name"; "valid type name with multiple namespace segments")]
        fn format(type_name: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                format!("type name: '{}'", FullyQualifiedTypeName::parse(type_name).unwrap()),
                format!("type name: '{}'", expected)
            )
        }

        #[test_case("Owner/Name", "Owner/Name"; "valid type name without namespace segments")]
        #[test_case("Owner.Namespace/Name", "Owner.Namespace/Name"; "valid type name with one namespace segment")]
        #[test_case("Owner.A.B.C/Name", "Owner.A.B.C/Name"; "valid type name with multiple namespace segments")]
        fn to_string(text: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                FullyQualifiedTypeName::parse(text).unwrap().to_string(),
                expected.to_string()
            )
        }
    }

    #[cfg(test)]
    mod from_str {
        use std::str::FromStr;

        use dsc_lib::types::{FullyQualifiedTypeName, FullyQualifiedTypeNameError};
        use test_case::test_case;

        #[test_case("Owner/Name" => matches Ok(_); "valid type name without namespace segments parses from string")]
        #[test_case("Owner.Namespace/Name" => matches Ok(_); "valid type name with one namespace segment parses from string")]
        #[test_case("Owner.A.B.C/Name" => matches Ok(_); "valid type name with multiple namespace segments parses from string")]
        #[test_case("invalid_name" => matches Err(_); "invalid type name fails to parse from string")]
        fn from_str(text: &str) -> Result<FullyQualifiedTypeName, FullyQualifiedTypeNameError> {
            FullyQualifiedTypeName::from_str(text)
        }

        #[test_case("Owner/Name" => matches Ok(_); "valid type name without namespace segments parses from string")]
        #[test_case("Owner.Namespace/Name" => matches Ok(_); "valid type name with one namespace segment parses from string")]
        #[test_case("Owner.A.B.C/Name" => matches Ok(_); "valid type name with multiple namespace segments parses from string")]
        #[test_case("invalid_name" => matches Err(_); "invalid type name fails to parse from string")]
        fn parse(text: &str) -> Result<FullyQualifiedTypeName, FullyQualifiedTypeNameError> {
            text.parse()
        }
    }

    #[cfg(test)]
    mod try_from {
        use dsc_lib::types::{FullyQualifiedTypeName, FullyQualifiedTypeNameError};
        use test_case::test_case;

        #[test_case("Owner/Name" => matches Ok(_); "valid type name without namespace segments converts from string")]
        #[test_case("Owner.Namespace/Name" => matches Ok(_); "valid type name with one namespace segment converts from string")]
        #[test_case("Owner.A.B.C/Name" => matches Ok(_); "valid type name with multiple namespace segments converts from string")]
        #[test_case("invalid_name" => matches Err(_); "invalid type name fails to convert from string")]
        fn string(text: &str) -> Result<FullyQualifiedTypeName, FullyQualifiedTypeNameError> {
            FullyQualifiedTypeName::try_from(text.to_string())
        }

        #[test_case("Owner/Name" => matches Ok(_); "valid type name without namespace segments converts from string")]
        #[test_case("Owner.Namespace/Name" => matches Ok(_); "valid type name with one namespace segment converts from string")]
        #[test_case("Owner.A.B.C/Name" => matches Ok(_); "valid type name with multiple namespace segments converts from string")]
        #[test_case("invalid_name" => matches Err(_); "invalid type name fails to convert from string")]
        fn str(text: &str) -> Result<FullyQualifiedTypeName, FullyQualifiedTypeNameError> {
            FullyQualifiedTypeName::try_from(text)
        }
    }

    #[cfg(test)]
    mod into {
        use dsc_lib::types::FullyQualifiedTypeName;
        
        #[test]
        fn string() {
            let _: String = FullyQualifiedTypeName::parse("Owner/Name").unwrap().into();
        }
    }

    #[cfg(test)]
    mod as_ref {
        use dsc_lib::types::FullyQualifiedTypeName;
        
        #[test]
        fn as_ref() {
            let _: &str = FullyQualifiedTypeName::parse("Owner/Name").unwrap().as_ref();
        }
    }

    #[cfg(test)]
    mod deref {
        use dsc_lib::types::FullyQualifiedTypeName;

        #[test]
        fn to_lowercase() {
            let n = FullyQualifiedTypeName::parse("Owner.Namespace/Name").unwrap();
            assert_eq!(n.to_lowercase(), "owner.namespace/name".to_string());
        }
    }

    #[cfg(test)]
    mod partial_eq {
        use dsc_lib::types::FullyQualifiedTypeName;
        use test_case::test_case;

        #[test_case("Owner/Name", "Owner/Name", true; "identical type names are equal")]
        #[test_case("Owner.Namespace/Name", "owner.namespace/name", true; "type names with different casing are equal")]
        #[test_case("Owner/Name", "Owner.Namespace/Name", false; "type names with different namespaces are not equal")]
        #[test_case("Owner/Name", "Owner/OtherName", false; "type names with different name segments are not equal")]
        #[test_case("Owner.Namespace/Name", "OtherOwner.Namespace/Name", false; "type names with different owner segments are not equal")]
        fn fully_qualified_type_name(lhs: &str, rhs: &str, should_be_equal: bool) {
            if should_be_equal {
                assert_eq!(
                    FullyQualifiedTypeName::parse(lhs).unwrap(),
                    FullyQualifiedTypeName::parse(rhs).unwrap()
                );
            } else {
                assert_ne!(
                    FullyQualifiedTypeName::parse(lhs).unwrap(),
                    FullyQualifiedTypeName::parse(rhs).unwrap()
                );
            }
        }

        #[test_case("Owner/Name", "Owner/Name", true; "identical type names are equal")]
        #[test_case("Owner.Namespace/Name", "owner.namespace/name", true; "type names with different casing are equal")]
        #[test_case("Owner/Name", "Owner.Namespace/Name", false; "type names with different namespaces are not equal")]
        #[test_case("Owner/Name", "Owner/OtherName", false; "type names with different name segments are not equal")]
        #[test_case("Owner.Namespace/Name", "OtherOwner.Namespace/Name", false; "type names with different owner segments are not equal")]
        #[test_case("Owner.Namespace/Name", "Not a FQTN", false; "type names are never equal to strings that can't parse as FQTNs")]
        fn string(type_name_string: &str, string_slice: &str, should_be_equal: bool) {
            let name = FullyQualifiedTypeName::parse(type_name_string).unwrap();
            let string = string_slice.to_string();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                name == string,
                should_be_equal,
                "expected comparison of {name} and {string} to be {should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string == name,
                should_be_equal,
                "expected comparison of {string} and {name} to be {should_be_equal}"
            );
        }

        #[test_case("Owner/Name", "Owner/Name", true; "identical type names are equal")]
        #[test_case("Owner.Namespace/Name", "owner.namespace/name", true; "type names with different casing are equal")]
        #[test_case("Owner/Name", "Owner.Namespace/Name", false; "type names with different namespaces are not equal")]
        #[test_case("Owner/Name", "Owner/OtherName", false; "type names with different name segments are not equal")]
        #[test_case("Owner.Namespace/Name", "OtherOwner.Namespace/Name", false; "type names with different owner segments are not equal")]
        #[test_case("Owner.Namespace/Name", "Not a FQTN", false; "type names are never equal to strings that can't parse as FQTNs")]
        fn str(type_name_string: &str, string_slice: &str, should_be_equal: bool) {
            let name = FullyQualifiedTypeName::parse(type_name_string).unwrap();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                name == string_slice,
                should_be_equal,
                "expected comparison of {name} and {string_slice} to be {should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string_slice == name,
                should_be_equal,
                "expected comparison of {string_slice} and {name} to be {should_be_equal}"
            );
        }
    }

    #[cfg(test)]
    mod ord {
        use dsc_lib::types::FullyQualifiedTypeName;
        use test_case::test_case;

        #[test_case("Owner/Name", "Owner/Name" => std::cmp::Ordering::Equal; "identical type names are equal")]
        #[test_case("owner/name", "Owner/Name" => std::cmp::Ordering::Equal; "differently cased type names are equal")]
        #[test_case("Owner.A/Name", "Owner.B/Name" => std::cmp::Ordering::Less; "type name with lexicographically smaller namespace is less")]
        #[test_case("owner.a/name", "Owner.B/Name" => std::cmp::Ordering::Less; "downcased type name with lexicographically smaller namespace is less")]
        #[test_case("Owner.B/Name", "Owner.A/Name" => std::cmp::Ordering::Greater; "type name with lexicographically larger namespace is greater")]
        #[test_case("owner.b/name", "Owner.A/Name" => std::cmp::Ordering::Greater; "downcased type name with lexicographically larger namespace is greater")]
        fn fully_qualified_type_name(lhs: &str, rhs: &str) -> std::cmp::Ordering {
            FullyQualifiedTypeName::parse(lhs).unwrap().cmp(&FullyQualifiedTypeName::parse(rhs).unwrap())
        }
    }
}
