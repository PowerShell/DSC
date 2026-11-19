// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    use dsc_lib::settings::fields::IgnoreSettingsFileField;
    use test_case::test_case;

    #[test_case(true; "when true")]
    #[test_case(false; "when false")]
    fn new(value: bool) {
        let actual = IgnoreSettingsFileField::new(value);
        pretty_assertions::assert_eq!(actual.as_ref(), &value);
    }
}

crate::macros::test_dsc_repo_schema! {
    define_statics: {
        dsc_lib::settings::fields::IgnoreSettingsFileField
    },
    test_meta_schema: {},
    test_docs: {
        title => "title",
        description => "description",
        markdown_description => "markdownDescription",
        markdown_enum_descriptions => "markdownEnumDescriptions"
    },
    test_validation: {
        boolean_true_is_valid => {
            input_json: json!(true),
            expected_valid: true,
        },
        boolean_false_is_valid => {
            input_json: json!(false),
            expected_valid: true,
        },
        null_is_invalid => {
            input_json: json!(null),
            expected_valid: false,
        },
        empty_object_is_invalid => {
            input_json: json!({}),
            expected_valid: false,
        },
        empty_array_is_invalid => {
            input_json: json!([]),
            expected_valid: false,
        },
        number_is_invalid => {
            input_json: json!(42),
            expected_valid: false,
        },
        string_is_invalid => {
            input_json: json!("string"),
            expected_valid: false,
        },
        non_empty_object_is_invalid => {
            input_json: json!({"key": true}),
            expected_valid: false,
        },
        non_empty_array_is_invalid => {
            input_json: json!([true, false, true]),
            expected_valid: false,
        }
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::settings::fields::IgnoreSettingsFileField;
    use test_case::test_case;
    use serde_json::{json, Value};

    #[test_case(true; "when true")]
    #[test_case(false; "when false")]
    fn serializing_to_string(value: bool) {
        let actual = serde_json::to_string(&IgnoreSettingsFileField::new(value))
            .expect("serialization should never fail");
        let expected = format!("{value}");
        assert_eq!(actual, expected);
    }

    #[test_case(true; "when true")]
    #[test_case(false; "when false")]
    fn serializing_to_value(value: bool) {
        let actual = serde_json::to_value(&IgnoreSettingsFileField::new(value))
            .expect("serialization should never fail");
        let expected = Value::Bool(value);
        assert_eq!(actual, expected);
    }

    #[test_case(json!("true"); "string value")]
    #[test_case(json!(1); "integer value")]
    #[test_case(json!(1.5); "number value")]
    #[test_case(json!([true]); "array value")]
    #[test_case(json!({"key": true}); "object value")]
    #[test_case(json!(null); "null value")]
    fn deserializing_invalid(input_value: Value) {
        let result = serde_json::from_value::<IgnoreSettingsFileField>(input_value.clone());
        assert!(result.is_err(), "expected deserialization to fail for input: {:?}", input_value);
    }

    #[test_case(json!(true); "when true")]
    #[test_case(json!(false); "when false")]
    fn deserializing_valid(input_value: Value) {
        let result = serde_json::from_value::<IgnoreSettingsFileField>(input_value.clone());
        assert!(result.is_ok(), "expected deserialization to succeed for input: {:?}", input_value);
    }
}

#[cfg(test)]
mod traits {
    mod as_ref {
        use dsc_lib::settings::fields::IgnoreSettingsFileField;

        #[test]
        fn bool() {
            let _: &bool = IgnoreSettingsFileField::new(true).as_ref();
        }
    }
    mod deref {
        use dsc_lib::settings::fields::IgnoreSettingsFileField;
        use std::ops::Deref;

        #[test]
        fn bool() {
            let f = IgnoreSettingsFileField::new(true);
            let b = f.deref();
            pretty_assertions::assert_eq!(b, &true)
        }
    }
    mod display {
        use dsc_lib::settings::fields::IgnoreSettingsFileField;
        use test_case::test_case;

        #[test_case(true, "true"; "when true")]
        #[test_case(false, "false"; "when false")]
        fn format(value: bool, expected: &str) {
            pretty_assertions::assert_eq!(
                format!("field: '{}'", IgnoreSettingsFileField::new(value)),
                format!("field: '{}'", expected)
            )
        }
    }
    mod from {
        use dsc_lib::settings::fields::IgnoreSettingsFileField;
        use test_case::test_case;

        #[test_case(true; "when true")]
        #[test_case(false; "when false")]
        fn bool(value: bool) {
            let actual = IgnoreSettingsFileField::from(value);
            let expected = IgnoreSettingsFileField::new(value);
            pretty_assertions::assert_eq!(actual, expected);
        }
    }
    mod into {
        use dsc_lib::settings::fields::IgnoreSettingsFileField;
        use test_case::test_case;

        #[test_case(true; "when true")]
        #[test_case(false; "when false")]
        fn bool(value: bool) {
            let actual: bool = IgnoreSettingsFileField::new(value).into();
            let expected = value;
            pretty_assertions::assert_eq!(actual, expected);
        }
    }
    mod partial_eq {
        use dsc_lib::settings::fields::IgnoreSettingsFileField;
        use test_case::test_case;

        #[test_case(true; "when true")]
        #[test_case(false; "when false")]
        fn eq_bool(value: bool) {
            let field = IgnoreSettingsFileField::new(value);
            pretty_assertions::assert_eq!(field == value, true);
            pretty_assertions::assert_eq!(value == field, true);
        }
    }
}
