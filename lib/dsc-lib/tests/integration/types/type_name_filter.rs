// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

macro_rules! literal {
    ($text:literal) => {
        dsc_lib::types::TypeNameFilter::Literal(
            $text.parse().unwrap()
        )
    };
}

macro_rules! wildcard {
    ($text:literal) => {
        dsc_lib::types::TypeNameFilter::Wildcard(
            $text.parse().unwrap()
        )
    };
}

macro_rules! fqtn {
    ($text:literal) => {
        dsc_lib::types::FullyQualifiedTypeName::parse($text).unwrap()
    };
}

#[cfg(test)]
mod methods {
    use dsc_lib::types::{FullyQualifiedTypeName, TypeNameFilter};
    use test_case::test_case;

    #[cfg(test)]
    mod parse {
        use dsc_lib::types::{
            TypeNameFilter,
            TypeNameFilter::*,
            TypeNameFilterError,
            TypeNameFilterError::*,
        };
        use test_case::test_case;

        #[test_case("Owner/Name" =>
            matches Literal(_);
            "literal filter without namespace segments"
        )]
        #[test_case("Owner.Namespace/Name" =>
            matches Literal(_);
            "literal filter with one namespace segment"
        )]
        #[test_case("Owner.A.B.C/Name" =>
            matches Literal(_);
            "literal filter with multiple namespace segments"
        )]
        #[test_case("*" =>
            matches Wildcard(_);
            "wildcard filter with single wildcard"
        )]
        #[test_case("**" =>
            matches Wildcard(_);
            "wildcard filter with consecutive wildcards"
        )]
        #[test_case("Contoso.Example/*" =>
            matches Wildcard(_);
            "wildcard filter with wildcard name"
        )]
        #[test_case("Contoso.*/Resource" =>
            matches Wildcard(_);
            "wildcard filter with wildcard namespace"
        )]
        #[test_case("*.Example/Resource" =>
            matches Wildcard(_);
            "wildcard filter with wildcard owner"
        )]
        #[test_case("Contoso.*.Example/*" =>
            matches Wildcard(_);
            "wildcard filter with multiple wildcards"
        )]
        #[test_case("Contoso*Owner" =>
            matches Wildcard(_);
            "wildcard filter with wildcard in owner without other segments"
        )]
        #[test_case("Contoso.Example*Namespace" =>
            matches Wildcard(_);
            "wildcard filter with wildcard in last namespace without name segment"
        )]
        fn valid(text: &str) -> TypeNameFilter {
            TypeNameFilter::parse(text).expect(&format!(
                "Expected '{}' to be a valid type name filter, but parsing failed",
                text
            ))
        }

        #[test_case("" =>
            matches InvalidLiteralTypeNameFilter{..};
            "empty string is not a valid literal filter"
        )]
        #[test_case("Owner.MissingName" =>
            matches InvalidLiteralTypeNameFilter{..};
            "literal filter missing forward slash"
        )]
        #[test_case("Owner.MissingName/" =>
            matches InvalidLiteralTypeNameFilter{..};
            "literal filter missing name after forward slash"
        )]
        #[test_case("Owner/Invalid&Name" =>
            matches InvalidLiteralTypeNameFilter{..};
            "literal filter with invalid character in name segment"
        )]
        #[test_case("Owner.ValidNamespace.Invalid&Namespace.ValidNamespace/Name" =>
            matches InvalidLiteralTypeNameFilter{..};
            "literal filter with invalid character in namespace segment"
        )]
        #[test_case(".Missing.Owner/Name" =>
            matches InvalidLiteralTypeNameFilter{..};
            "literal filter with missing owner segment before first namespace segment"
        )]
        #[test_case("/Name" =>
            matches InvalidLiteralTypeNameFilter{..};
            "literal filter with missing owner segment and leading slash"
        )]
        #[test_case("Owner.Empty.Namespace..Segment/Name" =>
            matches InvalidLiteralTypeNameFilter{..};
            "literal filter with empty namespace segment"
        )]
        #[test_case("Invalid&Owner/Name" => 
            matches InvalidLiteralTypeNameFilter{..};
            "literal filter with invalid character in owner segment"
        )]
        #[test_case("Invalid&Characters/In*Owner" =>    
            matches InvalidWildcardTypeNameFilter{..};
            "wildcard filter with invalid characters in owner segment"
        )]
        #[test_case(".Empty.Owner/*" =>
            matches InvalidWildcardTypeNameFilter{..};
            "wildcard filter with missing owner segment before first namespace segment"
        )]
        #[test_case("Owner.Invalid&Characters.*/InNameSpace" =>
            matches InvalidWildcardTypeNameFilter{..};
            "wildcard filter with invalid characters in namespace segment"
        )]
        #[test_case("Owner.With.Empty..Namespace/*" =>
            matches InvalidWildcardTypeNameFilter{..};
            "wildcard filter with empty namespace segment"
        )]
        #[test_case("Owner.*/Invalid&CharactersInName" =>
            matches InvalidWildcardTypeNameFilter{..};
            "wildcard filter with invalid characters in name segment"
        )]
        #[test_case("Owner.*.NamespaceWithoutNameSegment" =>
            matches InvalidWildcardTypeNameFilter{..};
            "wildcard filter with wildcard in namespace but missing name segment"
        )]
        #[test_case("Owner*.Namespace" =>
            matches InvalidWildcardTypeNameFilter{..};
            "wildcard filter with wildcard in owner but missing name segment"
        )]
        fn invalid(text: &str) -> TypeNameFilterError {
            TypeNameFilter::parse(text).expect_err(&format!(
                "Expected '{}' to be an invalid type name filter, but parsing succeeded",
                text
            ))
        }
    }

    #[test_case(&TypeNameFilter::Literal(FullyQualifiedTypeName::default()) => true; "only the default literal filter is empty")]
    #[test_case(&literal!("Contoso.Example/Resource") => false; "literal filter is never empty")]
    #[test_case(&wildcard!("Contoso.Example/*") => false; "wildcard filter is never empty")]
    fn is_empty(filter: &TypeNameFilter) -> bool {
        filter.is_empty()
    }

    #[test_case(&literal!("Contoso.Example/Resource"), &fqtn!("Contoso.Example/Resource") => true; "candidate matches literal filter exactly")]
    #[test_case(&literal!("Contoso.Example/Resource"), &fqtn!("contoso.example/resource") => true; "candidate matches literal filter with different casing")]
    #[test_case(&literal!("Contoso.Example/Resource"), &fqtn!("Example.Contoso/Resource") => false; "candidate does not match literal filter when text varies beyond casing")]
    #[test_case(&wildcard!("Contoso*"), &fqtn!("Contoso.Example/Resource") => true; "candidate matches wildcard filter when it starts with the wildcard text")]
    #[test_case(&wildcard!("Contoso*"), &fqtn!("Contoso/Resource") => true; "candidate matches wildcard filter when it starts with the wildcard text even without additional segments")]
    #[test_case(&wildcard!("Contoso*"), &fqtn!("Example.Contoso/Resource") => false; "candidate does not match wildcard filter when it does not start with the wildcard text")]
    fn is_match(filter: &TypeNameFilter, candidate: &FullyQualifiedTypeName) -> bool {
        filter.is_match(candidate)
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::types::TypeNameFilter;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case(json!("Contoso.Example/Resource") => matches Ok(_); "literal filter string deserializes")]
    #[test_case(json!("Contoso*") => matches Ok(_); "wildcard filter string deserializes")]
    #[test_case(json!("") => matches Err(_); "empty string fails")]
    #[test_case(json!(true) => matches Err(_); "boolean value fails")]
    #[test_case(json!(1) => matches Err(_); "integer value fails")]
    #[test_case(json!(1.2) => matches Err(_); "float value fails")]
    #[test_case(json!({"filter": "*"}) => matches Err(_); "object value fails")]
    #[test_case(json!(["*"]) => matches Err(_); "array value fails")]
    #[test_case(serde_json::Value::Null => matches Err(_); "null value fails")]
    fn deserialize(value: Value) -> Result<TypeNameFilter, serde_json::Error> {
        serde_json::from_value(value)
    }

    #[test_case(&literal!("Contoso.Example/Resource") =>
        json!("Contoso.Example/Resource");
        "literal filter serializes to string"
    )]
    #[test_case(&wildcard!("Contoso*") =>
        json!("Contoso*");
        "wildcard filter serializes to string"
    )]
    fn serialize(filter: &TypeNameFilter) -> Value {
        serde_json::to_value(filter).expect("serialize should never fail")
    }
}

#[cfg(test)]
mod traits {
    #[cfg(test)]
    mod default {
        use dsc_lib::types::TypeNameFilter;

        #[test]
        fn default() {
            let default_filter = TypeNameFilter::default();
            assert_eq!(
                default_filter,
                TypeNameFilter::Wildcard(dsc_lib::types::WildcardTypeName::default())
            );
        }
    }

    #[cfg(test)]
    mod display {
        use dsc_lib::types::TypeNameFilter;
        use test_case::test_case;

        #[test_case(&literal!("Contoso/Resource") => "Contoso/Resource".to_string(); "literal filter text")]
        #[test_case(&wildcard!("Contoso*") => "Contoso*".to_string(); "wildcard filter text")]
        fn to_string(filter: &TypeNameFilter) -> String {
            filter.to_string()
        }

        #[test_case(&literal!("Contoso/Resource") => "Contoso/Resource".to_string(); "literal filter text")]
        #[test_case(&wildcard!("Contoso*") => "Contoso*".to_string(); "wildcard filter text")]
        fn format(filter: &TypeNameFilter) -> String {
            format!("{}", filter)
        }
    }

    #[cfg(test)]
    mod from_str {
        use dsc_lib::types::{TypeNameFilter, TypeNameFilterError};
        use test_case::test_case;

        #[test_case("Contoso/Resource" => matches Ok(_); "literal filter string parses")]
        #[test_case("Contoso*" => matches Ok(_); "wildcard filter string parses")]
        #[test_case("" => matches Err(_); "empty string fails to parse")]
        #[test_case("Invalid&Filter" => matches Err(_); "string with invalid characters fails to parse")]
        fn from_str(text: &str) -> Result<TypeNameFilter, TypeNameFilterError> {
            text.parse()
        }

        #[test_case("Contoso/Resource" => matches Ok(_); "literal filter string parses")]
        #[test_case("Contoso*" => matches Ok(_); "wildcard filter string parses")]
        #[test_case("" => matches Err(_); "empty string fails to parse")]
        #[test_case("Invalid&Filter" => matches Err(_); "string with invalid characters fails to parse")]
        fn parse(text: &str) -> Result<TypeNameFilter, TypeNameFilterError> {
            TypeNameFilter::parse(text)
        }
    }

    #[cfg(test)]
    mod try_from {
        use dsc_lib::types::{TypeNameFilter, TypeNameFilterError};
        use std::convert::TryFrom;
        use test_case::test_case;

        #[test_case("Contoso/Resource" => matches Ok(_); "literal filter string parses")]
        #[test_case("Contoso*" => matches Ok(_); "wildcard filter string parses")]
        #[test_case("" => matches Err(_); "empty string fails to parse")]
        #[test_case("Invalid&Filter" => matches Err(_); "string with invalid characters fails to parse")]
        fn str(text: &str) -> Result<TypeNameFilter, TypeNameFilterError> {
            TypeNameFilter::try_from(text)
        }

        #[test_case("Contoso/Resource" => matches Ok(_); "literal filter string parses")]
        #[test_case("Contoso*" => matches Ok(_); "wildcard filter string parses")]
        #[test_case("" => matches Err(_); "empty string fails to parse")]
        #[test_case("Invalid&Filter" => matches Err(_); "string with invalid characters fails to parse")]
        fn string(text: &str) -> Result<TypeNameFilter, TypeNameFilterError> {
            TypeNameFilter::try_from(text.to_string())
        }
    }

    #[cfg(test)]
    mod from {
        use dsc_lib::types::{FullyQualifiedTypeName, TypeNameFilter, WildcardTypeName};

        #[test]
        fn fully_qualified_type_name() {
            let filter = TypeNameFilter::from(
                FullyQualifiedTypeName::parse("Contoso/Resource").unwrap()
            );

            assert!(matches!(filter, TypeNameFilter::Literal(_)));
            assert_eq!(filter.to_string(), "Contoso/Resource");
        }

        #[test]
        fn wildcard_type_name() {
            let filter = TypeNameFilter::from(
                WildcardTypeName::parse("Contoso*").unwrap()
            );

            assert!(matches!(filter, TypeNameFilter::Wildcard(_)));
            assert_eq!(filter.to_string(), "Contoso*");
        }
    }

    #[cfg(test)]
    mod into {
        use dsc_lib::types::TypeNameFilter;
        use test_case::test_case;

        #[test_case(literal!("Contoso/Resource") => "Contoso/Resource".to_string(); "literal filter into string")]
        #[test_case(wildcard!("Contoso*") => "Contoso*".to_string(); "wildcard filter into string")]
        fn string(filter: TypeNameFilter) -> String {
            filter.into()
        }
    }

    #[cfg(test)]
    mod try_into {
        use dsc_lib::types::{FullyQualifiedTypeName, TypeNameFilter, TypeNameFilterError, WildcardTypeName};
        use test_case::test_case;

        #[test_case(literal!("Contoso/Resource") => matches Ok(_); "literal filter converts")]
        #[test_case(wildcard!("Contoso*") => matches Err(_); "wildcard filter fails")]
        fn fully_qualified_type_name(filter: TypeNameFilter) -> Result<FullyQualifiedTypeName, TypeNameFilterError> {
            filter.try_into()
        }

        #[test_case(literal!("Contoso/Resource") => matches Err(_); "literal filter fails")]
        #[test_case(wildcard!("Contoso*") => matches Ok(_); "wildcard filter converts")]
        fn wildcard_type_name(filter: TypeNameFilter) -> Result<WildcardTypeName, TypeNameFilterError> {
            filter.try_into()
        }
    }
}
