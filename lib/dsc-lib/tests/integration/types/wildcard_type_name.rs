// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    use dsc_lib::types::WildcardTypeName;
    use test_case::test_case;

    #[cfg(test)]
    mod parse {
        use dsc_lib::types::{WildcardTypeName, WildcardTypeNameError};
        use test_case::test_case;

        #[test_case("*" => matches Ok(_); "single wildcard parses successfully")]
        #[test_case("**" => matches Ok(_); "consecutive wildcards parse successfully")]
        #[test_case("Contoso.Example/*" => matches Ok(_); "wildcard name parses successfully")]
        #[test_case("Contoso.*/Resource" => matches Ok(_); "wildcard namespace parses successfully")]
        #[test_case("*.Example/Resource" => matches Ok(_); "wildcard owner parses successfully")]
        #[test_case("Contoso.*.Example/*" => matches Ok(_); "multiple wildcards parse successfully")]
        #[test_case("Contoso*Owner" => matches Ok(_); "wildcard in owner without other segments parses successfully")]
        #[test_case("Contoso.Example*Namespace" => matches Ok(_); "wildcard in last namespace without name segment parses successfully")]
        fn valid(text: &str) -> Result<WildcardTypeName, WildcardTypeNameError> {
            WildcardTypeName::parse(text)
        }

        #[test_case("" =>
            WildcardTypeNameError::EmptyTypeName;
            "empty string"
        )]
        #[test_case("Type.Name.Without/Wildcards" =>
            WildcardTypeNameError::NoWildcard {
                text: "Type.Name.Without/Wildcards".to_string()
            }; "missing wildcard character"
        )]
        #[test_case("Invalid&Characters/In*Owner" =>
            WildcardTypeNameError::InvalidTypeName {
                text: "Invalid&Characters/In*Owner".to_string(),
                errors: vec![
                    WildcardTypeNameError::InvalidOwnerSegment {
                        segment_text: "Invalid&Characters".to_string(),
                    },
                ]
            }; "invalid characters in owner segment"
        )]
        #[test_case(".Empty.Owner/*" =>
            WildcardTypeNameError::InvalidTypeName {
                text: ".Empty.Owner/*".to_string(),
                errors: vec![
                    WildcardTypeNameError::EmptyOwnerSegment,
                ]
            }; "empty owner segment"
        )]
        #[test_case("Owner.Invalid&Characters.*/InNameSpace" =>
            WildcardTypeNameError::InvalidTypeName {
                text: "Owner.Invalid&Characters.*/InNameSpace".to_string(),
                errors: vec![
                    WildcardTypeNameError::InvalidNamespaceSegment {
                        segment_text: "Invalid&Characters".to_string(),
                    },
                ]
            }; "invalid characters in namespace segment"
        )]
        #[test_case("Owner.With.Empty..Namespace/*" =>
            WildcardTypeNameError::InvalidTypeName {
                text: "Owner.With.Empty..Namespace/*".to_string(),
                errors: vec![
                    WildcardTypeNameError::EmptyNamespaceSegment {
                        index: 3,
                    },
                ]
            }; "empty namespace segment"
        )]
        #[test_case("Owner.Empty.Last.Namespace./*" =>
            WildcardTypeNameError::InvalidTypeName{
                text: "Owner.Empty.Last.Namespace./*".to_string(),
                errors: vec![
                    WildcardTypeNameError::EmptyNamespaceSegment {
                        index: 4
                    }
                ]
            }; "empty namespace segment at end of namespaces"
        )]
        #[test_case("Owner.*/Invalid&CharactersInName" =>
            WildcardTypeNameError::InvalidTypeName {
                text: "Owner.*/Invalid&CharactersInName".to_string(),
                errors: vec![
                    WildcardTypeNameError::InvalidNameSegment {
                        segment_text: "Invalid&CharactersInName".to_string(),
                    },
                ]
            }; "invalid characters in name segment"
        )]
        #[test_case("Owner.*.NamespaceWithoutNameSegment" =>
            WildcardTypeNameError::InvalidTypeName {
                text: "Owner.*.NamespaceWithoutNameSegment".to_string(),
                errors: vec![
                    WildcardTypeNameError::MissingNameSegment,
                ]
            }; "missing name segment when wildcard is in prior namespace segment"
        )]
        #[test_case("Owner*.Namespace" =>
            WildcardTypeNameError::InvalidTypeName {
                text: "Owner*.Namespace".to_string(),
                errors: vec![
                    WildcardTypeNameError::MissingNameSegment,
                ]
            }; "missing name segment when wildcard is in owner segment succeeded by a namespace"
        )]
        #[test_case("Invalid&Owner.With.Empty..Namespace/And&Invalid*Name" =>
            WildcardTypeNameError::InvalidTypeName {
                text: "Invalid&Owner.With.Empty..Namespace/And&Invalid*Name".to_string(),
                errors: vec![
                    WildcardTypeNameError::InvalidOwnerSegment {
                        segment_text: "Invalid&Owner".to_string(),
                    },
                    WildcardTypeNameError::EmptyNamespaceSegment {
                        index: 3,
                    },
                    WildcardTypeNameError::InvalidNameSegment {
                        segment_text: "And&Invalid*Name".to_string(),
                    },
                ]
            }; "reports all errors in the wildcard type name"
        )]
        fn invalid(text: &str) -> WildcardTypeNameError {
            WildcardTypeName::parse(text).unwrap_err()
        }
    }

    #[test_case(&WildcardTypeName::default() => false; "default instance is not empty")]
    #[test_case(&WildcardTypeName::parse("Contoso.Example/*").unwrap() => false; "wildcard name is not empty")]
    fn is_empty(instance: &WildcardTypeName) -> bool {
        instance.is_empty()
    }

    #[test_case(
        &WildcardTypeName::parse("Contoso.Example/*").unwrap(),
        vec!["Contoso.Example/Resource", "Contoso.Example/OtherResource"],
        true;
        "matches candidate with same owner and namespace"
    )]
    #[test_case(
        &WildcardTypeName::parse("Contoso.Example/*").unwrap(),
        vec!["Contoso.OtherExample/Resource", "OtherContoso.Example/Resource"],
        false;
        "not matches candidate with different owner or namespace"
    )]
    #[test_case(
        &WildcardTypeName::parse("Contoso.*").unwrap(),
        vec!["Contoso.Example/Resource", "Contoso.OtherExample/Resource"],
        true;
        "matches candidate with same owner and any namespace and name"
    )]
    #[test_case(
        &WildcardTypeName::parse("Contoso.*").unwrap(),
        vec!["OtherContoso.Example/Resource", "OtherContoso.OtherExample/Resource"],
        false;
        "not matches candidate with different owner"
    )]
    fn is_match(filter: &WildcardTypeName, candidates: Vec<&str>, should_match: bool) {
        for candidate in candidates {
            pretty_assertions::assert_eq!(
                filter.is_match(&candidate.parse().unwrap()),
                should_match,
                "expected filter {filter} to {}match candidate {candidate}",
                if should_match { "" } else { "not " }
            );
        }
    }

    #[test_case("*", r"^.*?$"; "regex for single wildcard")]
    #[test_case("Contoso.Example/*", r"^Contoso\.Example/.*?$"; "regex for wildcard name")]
    #[test_case("Contoso.*/Resource", r"^Contoso\..*?/Resource$"; "regex for wildcard namespace")]
    #[test_case("*.Example/Resource", r"^.*?\.Example/Resource$"; "regex for wildcard owner")]
    #[test_case("Contoso.*.Example/*", r"^Contoso\..*?\.Example/.*?$"; "regex for multiple wildcards")]
    fn as_regex(text: &str, expected: &str) {
        pretty_assertions::assert_eq!(
             WildcardTypeName::parse(text).unwrap().as_regex().as_str(),
             expected,
             "expected wildcard type name '{text}' to convert to regex `{expected}`"
        );
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::types::WildcardTypeName;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!("*") => matches Ok(_); "single wildcard string deserializes")]
    #[test_case(json!("**") => matches Ok(_); "consecutive wildcards string deserializes")]
    #[test_case(json!("Contoso.Example/*") => matches Ok(_); "wildcard name string deserializes")]
    #[test_case(json!("Contoso.*/Resource") => matches Ok(_); "wildcard namespace string deserializes")]
    #[test_case(json!("*.Example/Resource") => matches Ok(_); "wildcard owner string deserializes")]
    #[test_case(json!("Contoso.*.Example/*") => matches Ok(_); "multiple wildcards string deserializes")]
    #[test_case(json!("invalid_name") => matches Err(_); "invalid type name string fails")]
    #[test_case(json!("") => matches Err(_); "empty string fails")]
    #[test_case(json!(true) => matches Err(_); "boolean value fails")]
    #[test_case(json!(1) => matches Err(_); "integer value fails")]
    #[test_case(json!(1.2) => matches Err(_); "float value fails")]
    #[test_case(json!({"filter": "*"}) => matches Err(_); "object value fails")]
    #[test_case(json!(["*"]) => matches Err(_); "array value fails")]
    #[test_case(serde_json::Value::Null => matches Err(_); "null value fails")]
    fn deserialize(text: Value) -> Result<WildcardTypeName, serde_json::Error> {
        serde_json::from_value(json!(text))
    }

    #[test_case("*" => json!("*"); "single wildcard serializes")]
    #[test_case("**" => json!("**"); "consecutive wildcards serializes")]
    #[test_case("Contoso.Example/*" => json!("Contoso.Example/*"); "wildcard name serializes")]
    #[test_case("Contoso.*/Resource" => json!("Contoso.*/Resource"); "wildcard namespace serializes")]
    #[test_case("*.Example/Resource" => json!("*.Example/Resource"); "wildcard owner serializes")]
    #[test_case("Contoso.*.Example/*" => json!("Contoso.*.Example/*"); "multiple wildcards serializes")]
    fn serialize(text: &str) -> Value {
        let instance = WildcardTypeName::parse(text).unwrap();
        serde_json::to_value(instance).unwrap()
    }
}

// #[cfg(test)]
// mod traits {
//     #[cfg(test)]
//     mod default {
//         use dsc_lib::types::WildcardTypeName;

//         #[test]
//         fn default_is_empty() {
//             let instance = WildcardTypeName::default();
//             assert!(instance.is_empty())
//         }
//     }

//     #[cfg(test)]
//     mod display {
//         use dsc_lib::types::WildcardTypeName;
//         use test_case::test_case;

//         #[test_case("Owner/Name", "Owner/Name"; "valid type name without namespace segments")]
//         #[test_case("Owner.Namespace/Name", "Owner.Namespace/Name"; "valid type name with one namespace segment")]
//         #[test_case("Owner.A.B.C/Name", "Owner.A.B.C/Name"; "valid type name with multiple namespace segments")]
//         fn format(type_name: &str, expected: &str) {
//             pretty_assertions::assert_eq!(
//                 format!("type name: '{}'", WildcardTypeName::parse(type_name).unwrap()),
//                 format!("type name: '{}'", expected)
//             )
//         }

//         #[test_case("Owner/Name", "Owner/Name"; "valid type name without namespace segments")]
//         #[test_case("Owner.Namespace/Name", "Owner.Namespace/Name"; "valid type name with one namespace segment")]
//         #[test_case("Owner.A.B.C/Name", "Owner.A.B.C/Name"; "valid type name with multiple namespace segments")]
//         fn to_string(text: &str, expected: &str) {
//             pretty_assertions::assert_eq!(
//                 WildcardTypeName::parse(text).unwrap().to_string(),
//                 expected.to_string()
//             )
//         }
//     }

//     #[cfg(test)]
//     mod from_str {
//         use std::str::FromStr;

//         use dsc_lib::types::{WildcardTypeName, WildcardTypeNameError};
//         use test_case::test_case;

//         #[test_case("Owner/Name" => matches Ok(_); "valid type name without namespace segments parses from string")]
//         #[test_case("Owner.Namespace/Name" => matches Ok(_); "valid type name with one namespace segment parses from string")]
//         #[test_case("Owner.A.B.C/Name" => matches Ok(_); "valid type name with multiple namespace segments parses from string")]
//         #[test_case("invalid_name" => matches Err(_); "invalid type name fails to parse from string")]
//         fn from_str(text: &str) -> Result<WildcardTypeName, WildcardTypeNameError> {
//             WildcardTypeName::from_str(text)
//         }

//         #[test_case("Owner/Name" => matches Ok(_); "valid type name without namespace segments parses from string")]
//         #[test_case("Owner.Namespace/Name" => matches Ok(_); "valid type name with one namespace segment parses from string")]
//         #[test_case("Owner.A.B.C/Name" => matches Ok(_); "valid type name with multiple namespace segments parses from string")]
//         #[test_case("invalid_name" => matches Err(_); "invalid type name fails to parse from string")]
//         fn parse(text: &str) -> Result<WildcardTypeName, WildcardTypeNameError> {
//             text.parse()
//         }
//     }

//     #[cfg(test)]
//     mod try_from {
//         use dsc_lib::types::{WildcardTypeName, WildcardTypeNameError};
//         use test_case::test_case;

//         #[test_case("Owner/Name" => matches Ok(_); "valid type name without namespace segments converts from string")]
//         #[test_case("Owner.Namespace/Name" => matches Ok(_); "valid type name with one namespace segment converts from string")]
//         #[test_case("Owner.A.B.C/Name" => matches Ok(_); "valid type name with multiple namespace segments converts from string")]
//         #[test_case("invalid_name" => matches Err(_); "invalid type name fails to convert from string")]
//         fn string(text: &str) -> Result<WildcardTypeName, WildcardTypeNameError> {
//             WildcardTypeName::try_from(text.to_string())
//         }

//         #[test_case("Owner/Name" => matches Ok(_); "valid type name without namespace segments converts from string")]
//         #[test_case("Owner.Namespace/Name" => matches Ok(_); "valid type name with one namespace segment converts from string")]
//         #[test_case("Owner.A.B.C/Name" => matches Ok(_); "valid type name with multiple namespace segments converts from string")]
//         #[test_case("invalid_name" => matches Err(_); "invalid type name fails to convert from string")]
//         fn str(text: &str) -> Result<WildcardTypeName, WildcardTypeNameError> {
//             WildcardTypeName::try_from(text)
//         }
//     }

//     #[cfg(test)]
//     mod into {
//         use dsc_lib::types::WildcardTypeName;
        
//         #[test]
//         fn string() {
//             let _: String = WildcardTypeName::parse("Owner/Name").unwrap().into();
//         }
//     }

//     #[cfg(test)]
//     mod as_ref {
//         use dsc_lib::types::WildcardTypeName;
        
//         #[test]
//         fn as_ref() {
//             let _: &str = WildcardTypeName::parse("Owner/Name").unwrap().as_ref();
//         }
//     }

//     #[cfg(test)]
//     mod deref {
//         use dsc_lib::types::WildcardTypeName;

//         #[test]
//         fn to_lowercase() {
//             let n = WildcardTypeName::parse("Owner.Namespace/Name").unwrap();
//             assert_eq!(n.to_lowercase(), "owner.namespace/name".to_string());
//         }
//     }

//     #[cfg(test)]
//     mod partial_eq {
//         use dsc_lib::types::WildcardTypeName;
//         use test_case::test_case;

//         #[test_case("Owner/Name", "Owner/Name", true; "identical type names are equal")]
//         #[test_case("Owner.Namespace/Name", "owner.namespace/name", true; "type names with different casing are equal")]
//         #[test_case("Owner/Name", "Owner.Namespace/Name", false; "type names with different namespaces are not equal")]
//         #[test_case("Owner/Name", "Owner/OtherName", false; "type names with different name segments are not equal")]
//         #[test_case("Owner.Namespace/Name", "OtherOwner.Namespace/Name", false; "type names with different owner segments are not equal")]
//         fn fully_qualified_type_name(lhs: &str, rhs: &str, should_be_equal: bool) {
//             if should_be_equal {
//                 assert_eq!(
//                     WildcardTypeName::parse(lhs).unwrap(),
//                     WildcardTypeName::parse(rhs).unwrap()
//                 );
//             } else {
//                 assert_ne!(
//                     WildcardTypeName::parse(lhs).unwrap(),
//                     WildcardTypeName::parse(rhs).unwrap()
//                 );
//             }
//         }

//         #[test_case("Owner/Name", "Owner/Name", true; "identical type names are equal")]
//         #[test_case("Owner.Namespace/Name", "owner.namespace/name", true; "type names with different casing are equal")]
//         #[test_case("Owner/Name", "Owner.Namespace/Name", false; "type names with different namespaces are not equal")]
//         #[test_case("Owner/Name", "Owner/OtherName", false; "type names with different name segments are not equal")]
//         #[test_case("Owner.Namespace/Name", "OtherOwner.Namespace/Name", false; "type names with different owner segments are not equal")]
//         #[test_case("Owner.Namespace/Name", "Not a FQTN", false; "type names are never equal to strings that can't parse as FQTNs")]
//         fn string(type_name_string: &str, string_slice: &str, should_be_equal: bool) {
//             let name = WildcardTypeName::parse(type_name_string).unwrap();
//             let string = string_slice.to_string();

//             // Test equivalency bidirectionally
//             pretty_assertions::assert_eq!(
//                 name == string,
//                 should_be_equal,
//                 "expected comparison of {name} and {string} to be {should_be_equal}"
//             );

//             pretty_assertions::assert_eq!(
//                 string == name,
//                 should_be_equal,
//                 "expected comparison of {string} and {name} to be {should_be_equal}"
//             );
//         }

//         #[test_case("Owner/Name", "Owner/Name", true; "identical type names are equal")]
//         #[test_case("Owner.Namespace/Name", "owner.namespace/name", true; "type names with different casing are equal")]
//         #[test_case("Owner/Name", "Owner.Namespace/Name", false; "type names with different namespaces are not equal")]
//         #[test_case("Owner/Name", "Owner/OtherName", false; "type names with different name segments are not equal")]
//         #[test_case("Owner.Namespace/Name", "OtherOwner.Namespace/Name", false; "type names with different owner segments are not equal")]
//         #[test_case("Owner.Namespace/Name", "Not a FQTN", false; "type names are never equal to strings that can't parse as FQTNs")]
//         fn str(type_name_string: &str, string_slice: &str, should_be_equal: bool) {
//             let name = WildcardTypeName::parse(type_name_string).unwrap();

//             // Test equivalency bidirectionally
//             pretty_assertions::assert_eq!(
//                 name == string_slice,
//                 should_be_equal,
//                 "expected comparison of {name} and {string_slice} to be {should_be_equal}"
//             );

//             pretty_assertions::assert_eq!(
//                 string_slice == name,
//                 should_be_equal,
//                 "expected comparison of {string_slice} and {name} to be {should_be_equal}"
//             );
//         }
//     }
// }
