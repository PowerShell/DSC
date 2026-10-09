// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines integration tests for generating JSON Schemas for the public types in [`dsc-lib`].

#[cfg(test)] mod schema_for;

/// Defines helper macros for testing JSON Schemas.
#[allow(unused_imports)]
#[macro_use]
pub(crate) mod macros {
    pub(crate) use crate::macros::define_schema_statics;
    pub(crate) use crate::macros::test_dsc_repo_schema;
    pub(crate) use crate::macros::test_meta_schema;
    pub(crate) use crate::macros::test_schema_docs;
    pub(crate) use crate::macros::test_schema_validation;
}

/// Defines a pattern that matches an unresolved i18n key.
///
/// Used in the [`assert_schema_has_docs_keyword`] function.
static KEYWORD_PATTERN: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"^\w+(\.\w+)+$").expect("pattern is valid")
});

/// Asserts that a given schema contains the specified documentation keyword
/// and that its value is properly defined in the translation files.
///
/// This helper function is only intended to be used within the integration
/// tests for schemas. Generally, it is invoked through the tests automatically
/// created by the [`test_schema_docs!`] macro.
///
/// # Arguments
///
/// - `keyword`: The documentation keyword to check in the schema.
/// - `schema`: The schema to be validated.
///
/// # Panics
///
/// Panics if the specified keyword isn't found in the schema or if its value
/// appears to be a translation key, like `some.schema.title`.
///
/// [`test_schema_docs!`]: crate::macros::test_schema_docs
pub(crate) fn assert_schema_has_docs_keyword(keyword: &str, schema: &schemars::Schema) {
    use dsc_lib::schemas::schema_utility_extensions::SchemaUtilityExtensions;
    match keyword {
        "enumDescriptions" | "markdownEnumDescriptions" => {
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
                .expect(&format!("expected keyword '{keyword}' to be defined"));

            assert!(
                !(&*KEYWORD_PATTERN).is_match(value),
                "Expected keyword '{keyword}' to be defined in translation, but was set to i18n key '{value}'",
            );
        }
    }
}

/// Asserts that a given JSON input is valid or invalid against the provided
/// schema.
///
/// This helper function is only intended to be used within the integration
/// tests for schemas. Generally, it is invoked through the tests automatically
/// created by the [`test_schema_validation!`] macro.
///
/// # Arguments
///
/// - `input_json`: The JSON input to validate against the schema.
/// - `expected_valid`: A boolean indicating whether the input is expected to be valid.
/// - `validator`: The JSON schema validator to use.
/// - `root_schema`: The root schema against which the input is validated.
///
/// # Panics
///
/// Panics if the validation result does not match the expected validity.
///
/// [`test_schema_validation!`]: crate::macros::test_schema_validation
pub(crate)fn assert_schema_validation(
    input_json: &serde_json::Value,
    expected_valid: bool,
    validator: &jsonschema::Validator,
    root_schema: &schemars::Schema,
) {
    let validator = validator.clone();
    let result = validator.validate(input_json);
    if expected_valid {
        assert!(
            result.is_ok(),
            "expected input to be valid for schema:\n---\ninput: {}\n---\nerror: {:#?}\n---",
            serde_json::to_string_pretty(input_json).unwrap(),
            result.unwrap_err());
    } else {
        assert!(
            result.is_err(),
            "expected input to be invalid for schema:\n---\ninput: {}\n---\nschema: {}\n---",
            serde_json::to_string_pretty(input_json).unwrap(),
            serde_json::to_string_pretty(&*root_schema).unwrap()
        );
    }
}
