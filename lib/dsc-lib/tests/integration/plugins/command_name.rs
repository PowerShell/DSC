// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    #[cfg(test)]
    mod parse {
        use dsc_lib::plugins::command_name::{PluginCommandName, PluginCommandNameError};
        use test_case::test_case;

        #[test_case("lint"; "lowercase letters")]
        #[test_case("a"; "single letter")]
        #[test_case("lint2"; "letters followed by a digit")]
        #[test_case("resource-kit"; "hyphen between two groups")]
        #[test_case("a-1-b2"; "several hyphenated groups")]
        fn valid(text: &str) {
            let name = PluginCommandName::parse(text).expect("name should be valid");

            pretty_assertions::assert_eq!(name.to_string(), text);
        }

        #[test_case("" =>
            vec![PluginCommandNameError::EmptyName];
            "empty string"
        )]
        #[test_case("Lint" =>
            vec![
                PluginCommandNameError::InvalidFirstCharacter { character: 'L' },
                PluginCommandNameError::InvalidCharacters { characters: "L".to_string() },
            ];
            "uppercase first letter"
        )]
        #[test_case("2lint" =>
            vec![PluginCommandNameError::InvalidFirstCharacter { character: '2' }];
            "digit as first character"
        )]
        #[test_case("-lint" =>
            vec![PluginCommandNameError::InvalidFirstCharacter { character: '-' }];
            "hyphen as first character"
        )]
        #[test_case("lint_docs" =>
            vec![PluginCommandNameError::InvalidCharacters { characters: "_".to_string() }];
            "underscore"
        )]
        #[test_case("lint docs" =>
            vec![PluginCommandNameError::InvalidCharacters { characters: " ".to_string() }];
            "space"
        )]
        #[test_case("lintDOCS.DOC" =>
            vec![PluginCommandNameError::InvalidCharacters { characters: "DOCS.".to_string() }];
            "lists each invalid character once"
        )]
        #[test_case("lint-" =>
            vec![PluginCommandNameError::MisplacedHyphen];
            "trailing hyphen"
        )]
        #[test_case("lint--docs" =>
            vec![PluginCommandNameError::MisplacedHyphen];
            "consecutive hyphens"
        )]
        #[test_case("L_i--" =>
            vec![
                PluginCommandNameError::InvalidFirstCharacter { character: 'L' },
                PluginCommandNameError::InvalidCharacters { characters: "L_".to_string() },
                PluginCommandNameError::MisplacedHyphen,
            ];
            "reports every rule the name breaks"
        )]
        fn invalid(text: &str) -> Vec<PluginCommandNameError> {
            match PluginCommandName::parse(text).unwrap_err() {
                PluginCommandNameError::InvalidName { text: actual_text, errors } => {
                    pretty_assertions::assert_eq!(actual_text, text);
                    errors
                },
                other => panic!("expected the collecting error, got: {other:?}"),
            }
        }
    }

    #[cfg(test)]
    mod validate {
        use dsc_lib::plugins::command_name::PluginCommandName;
        use regex::Regex;
        use test_case::test_case;

        #[test_case("lint")]
        #[test_case("a")]
        #[test_case("lint2")]
        #[test_case("resource-kit")]
        #[test_case("a-1-b2")]
        #[test_case("")]
        #[test_case("Lint")]
        #[test_case("2lint")]
        #[test_case("-lint")]
        #[test_case("lint_docs")]
        #[test_case("lint docs")]
        #[test_case("lint-")]
        #[test_case("lint--docs")]
        #[test_case("-")]
        #[test_case("lint\n")]
        fn agrees_with_validating_pattern(text: &str) {
            let pattern = Regex::new(PluginCommandName::VALIDATING_PATTERN).expect("pattern is valid");

            pretty_assertions::assert_eq!(
                PluginCommandName::validate(text).is_ok(),
                pattern.is_match(text),
                "validation and the published pattern disagree for '{}'",
                text
            );
        }
    }
}

crate::macros::test_dsc_repo_schema! {
    define_statics: {
        dsc_lib::plugins::command_name::PluginCommandName
    },
    test_meta_schema: {},
    test_docs: {
        title => "title",
        description => "description",
        markdown_description => "markdownDescription",
        pattern_error_message => "patternErrorMessage"
    },
    test_validation: {
        lowercase_name_is_valid => {
            input_json: json!("lint"),
            expected_valid: true,
        },
        hyphenated_name_is_valid => {
            input_json: json!("resource-kit"),
            expected_valid: true,
        },
        name_with_digits_is_valid => {
            input_json: json!("lint2"),
            expected_valid: true,
        },
        empty_string_is_invalid => {
            input_json: json!(""),
            expected_valid: false,
        },
        uppercase_name_is_invalid => {
            input_json: json!("Lint"),
            expected_valid: false,
        },
        name_with_leading_digit_is_invalid => {
            input_json: json!("2lint"),
            expected_valid: false,
        },
        name_with_trailing_hyphen_is_invalid => {
            input_json: json!("lint-"),
            expected_valid: false,
        },
        name_with_consecutive_hyphens_is_invalid => {
            input_json: json!("lint--docs"),
            expected_valid: false,
        },
        name_with_underscore_is_invalid => {
            input_json: json!("lint_docs"),
            expected_valid: false,
        },
        boolean_is_invalid => {
            input_json: json!(true),
            expected_valid: false,
        },
        number_is_invalid => {
            input_json: json!(1),
            expected_valid: false,
        },
        object_is_invalid => {
            input_json: json!({"name": "lint"}),
            expected_valid: false,
        },
        array_is_invalid => {
            input_json: json!(["lint"]),
            expected_valid: false,
        },
        null_is_invalid => {
            input_json: json!(null),
            expected_valid: false,
        }
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::plugins::command_name::PluginCommandName;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case("lint"; "lowercase name")]
    #[test_case("resource-kit"; "hyphenated name")]
    fn serializing(name: &str) {
        let actual = serde_json::to_string(&PluginCommandName::parse(name).expect("name should be valid"))
            .expect("serialization should never fail");

        pretty_assertions::assert_eq!(actual, format!(r#""{name}""#));
    }

    #[test_case(json!("lint") => matches Ok(_); "lowercase string value is valid")]
    #[test_case(json!("resource-kit") => matches Ok(_); "hyphenated string value is valid")]
    #[test_case(json!("Lint") => matches Err(_); "uppercase string value raises error")]
    #[test_case(json!("") => matches Err(_); "empty string value raises error")]
    #[test_case(json!(true) => matches Err(_); "boolean value raises error")]
    #[test_case(json!(1) => matches Err(_); "integer value raises error")]
    #[test_case(json!({"name": "lint"}) => matches Err(_); "object value raises error")]
    #[test_case(json!(["lint"]) => matches Err(_); "array value raises error")]
    #[test_case(Value::Null => matches Err(_); "null value raises error")]
    fn deserializing(value: Value) -> Result<PluginCommandName, serde_json::Error> {
        serde_json::from_value::<PluginCommandName>(value)
    }

    #[test]
    fn deserializing_error_names_the_broken_rule() {
        let err = serde_json::from_value::<PluginCommandName>(json!("lint-")).unwrap_err();

        assert!(
            err.to_string().contains("lint-"),
            "expected the error to name the invalid text, got: {err}"
        );
    }
}

#[cfg(test)]
mod traits {
    use std::str::FromStr;

    use dsc_lib::plugins::command_name::{PluginCommandName, PluginCommandNameError};
    use test_case::test_case;

    fn name() -> PluginCommandName {
        PluginCommandName::parse("resource-kit").expect("name should be valid")
    }

    #[test]
    fn display() {
        pretty_assertions::assert_eq!(format!("dsc {}", name()), "dsc resource-kit");
    }

    #[test]
    fn as_ref() {
        let name = name();
        let text: &str = name.as_ref();

        pretty_assertions::assert_eq!(text, "resource-kit");
    }

    #[test]
    fn deref() {
        let name = name();

        assert!(name.starts_with("resource"));
        pretty_assertions::assert_eq!(name.len(), 12);
    }

    #[test_case("lint" => matches Ok(_); "valid name")]
    #[test_case("Lint" => matches Err(PluginCommandNameError::InvalidName { .. }); "invalid name")]
    fn from_str(text: &str) -> Result<PluginCommandName, PluginCommandNameError> {
        PluginCommandName::from_str(text)
    }

    #[test_case("lint" => matches Ok(_); "valid name")]
    #[test_case("Lint" => matches Err(PluginCommandNameError::InvalidName { .. }); "invalid name")]
    fn try_from_string(text: &str) -> Result<PluginCommandName, PluginCommandNameError> {
        PluginCommandName::try_from(text.to_string())
    }

    #[test_case("lint" => matches Ok(_); "valid name")]
    #[test_case("Lint" => matches Err(PluginCommandNameError::InvalidName { .. }); "invalid name")]
    fn try_from_str(text: &str) -> Result<PluginCommandName, PluginCommandNameError> {
        PluginCommandName::try_from(text)
    }

    #[test]
    fn into_string() {
        let text: String = name().into();

        pretty_assertions::assert_eq!(text, "resource-kit".to_string());
    }

    #[test]
    fn partial_eq() {
        let name = name();

        assert!(name == "resource-kit");
        assert!("resource-kit" == name);
        assert!(name == *"resource-kit");
        assert!(*"resource-kit" == name);
        assert!(name == "resource-kit".to_string());
        assert!("resource-kit".to_string() == name);
        assert!(name != "Resource-Kit");
        assert!(name != "lint");
    }

    #[test]
    fn ord() {
        let mut names = vec![
            PluginCommandName::parse("lint").unwrap(),
            PluginCommandName::parse("graph").unwrap(),
            PluginCommandName::parse("lint2").unwrap(),
        ];
        names.sort();

        pretty_assertions::assert_eq!(
            names.iter().map(ToString::to_string).collect::<Vec<String>>(),
            vec!["graph".to_string(), "lint".to_string(), "lint2".to_string()]
        );
    }
}
