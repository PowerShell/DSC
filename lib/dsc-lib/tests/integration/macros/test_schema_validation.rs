// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Generate tests for validating data against the root schema.
///
/// This macro creates a `validation` module that contains tests for each
/// provided case. Every case must specify the input json and whether the data
/// is expected to be valid against the schema.
///
/// <div class="info">
///
/// This macro is best used through the [`test_dsc_repo_schema!`] macro. Only
/// use this macro directly when the higher level macro is unsuitable.
///
/// </div>
///
/// # Syntax
///
/// Every test case must be defined with the following structure:
///
/// ```text
/// <test_name> => {
///     input_json: json!(...),
///     expected_valid: true|false,
/// }
/// ```
///
/// - `<test_name>`: The name of the test case function.
/// - `input_json`: The JSON input to be validated against the schema. The macro
///   expects you to define a JSON value with the `json!` macro, but you don't
///   need to define a using statement, because the macro automatically handles
///   expanding to the `serde_json::json!` macro internally.
/// - `expected_valid`: A boolean indicating whether the input is expected to
///   be valid.
///
/// # Example
///
/// This example shows how to define test cases for valid and invalid data.
///
/// ```rust, ignore
/// test_schema_validation! {
///     valid_case => {
///         input_json: json!({"key": "value"}),
///         expected_valid: true,
///     },
///     invalid_case => {
///         input_json: json!({"key": 123}),
///         expected_valid: false,
///     },
/// }
/// ```
///
/// <details><summary><Expanded macro usage</summary>
///
/// ```rust, ignore
/// #[cfg(test)]
/// mod validation {
///     use super::*;
///     #[test]
///     fn valid_case() {
///         $crate::schemas::assert_schema_validation(
///             &serde_json::json!({"key": "value"}),
///             true,
///             &VALIDATOR.clone(),
///             &ROOT_SCHEMA.clone()
///         );
///     }
///
///     #[test]
///     fn invalid_case() {
///         $crate::schemas::assert_schema_validation(
///             &serde_json::json!({"key": 123})    ,
///             false,
///             &VALIDATOR.clone(),
///             &ROOT_SCHEMA.clone()
///         );
///     }
/// }
/// ```
///
/// </details>
///
/// [`test_dsc_repo_schema!`]: crate::macros::test_dsc_repo_schema
macro_rules! test_schema_validation {
    ($(
        $case_description:ident => {
            input_json: json!($($case_input:tt)*),
            expected_valid: $case_expected_valid:literal,
        }
    ),+ $(,)?) => {
        #[cfg(test)] mod validation {
            use super::*;
            $(
                #[test]
                fn $case_description() {
                    $crate::schemas::assert_schema_validation(
                        &serde_json::json!($($case_input)*),
                        $case_expected_valid,
                        &VALIDATOR.clone(),
                        &ROOT_SCHEMA.clone()
                    );
                }
            )+
        }
    };
}

pub(crate) use test_schema_validation;
