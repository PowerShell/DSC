// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Defines tests for DSC repository schemas.
///
/// This macro generates a test module that includes the schema statics,
/// meta schema tests, documentation tests, and validation tests for a DSC
/// repository schema. It is intended to be used in integration tests for
/// DSC repository schemas.
///
/// # Syntax
///
/// The macro takes four arguments, all of which are _required_:
///
/// - `define_statics`: Defines the schema statics using the
///   [`define_schema_statics!`] macro.
///
///   These statics are leveraged by the rest of the macro to test the schema.
///
/// - `test_meta_schema`: Defines the meta schema tests using the
///   [`test_meta_schema!`] macro.
///
///   These tests ensure that the generated JSON Schema is valid against its own
///   meta schema using the [`jsonschema::meta::validate`] function.
///
/// - `test_docs`: Defines the documentation tests using the
///   [`test_schema_docs!`] macro.
///
///   These tests ensure that the documentation keywords are defined both in the
///   schema and in translation files.
/// - `test_validation`: Defines the validation tests using the
///   [`test_schema_validation!`] macro.
///
///   These tests provide a set of test cases to validate valid and invalid
///   data against the schema.
///
/// ## `define_statics`
///
/// ```rust, ignore
/// test_dsc_repo_schema! {
///     define_statics: {
///         <type_path>,
///         [properties: {
///             <STATIC_NAME> => "<propertyName>",
///             ...
///         },]
///     },
///     ...
/// }
/// ```
///
/// - `<type_path>` - the path to the type that should be tested, like
///   `my_crate::MyType`.
/// - `properties: {...}` defines a set of additional statics for the property
///   subschemas, which are required for validating documentation keywords in
///   those properties. Every property must be defined with the name of the
///   static variable to create and the property name to retrieve as a subschema.
///
/// ## `test_meta_schema`
///
/// ```rust, ignore
/// test_dsc_repo_schema! {
///     // Elided for brevity
///     test_meta_schema: {},
///     ...
/// }
/// ```
///
/// When using this macro, always define `test_meta_schema` with an empty block.
/// The [`test_meta_schema!`] macro supports specifying an alternate static
/// for validation but this macro should always validate the root schema.
///
/// ## `test_docs`
///
/// ```rust, ignore
/// test_dsc_repo_schema! {
///     // Elided for brevity
///     test_docs: {
///         <test_name> => "<keyword>",
///         <test_name> => (<STATIC_NAME>, "<keyword>"),
///     },
///     ...
/// }
/// ```
///
/// If you define `test_docs` with an empty block, the macro generates tests for
/// the `title`, `description`, and `markdownDescription` keywords on the root
/// schema.
///
/// To test additional keywords on the root schema or on a subschema, you must
/// specify a case for every keyword you want to test:
///
/// - When you define a case as `<test_name> => "<keyword>"`, it tests for the
///   keyword on the root schema.
/// - When you define a case as `<test_name> => (<STATIC_NAME>, "<keyword>")`,
///   it tests for the keyword on the subschema identified by `<STATIC_NAME>`.
///
/// ## `test_validation`
///
/// ```rust, ignore
/// test_dsc_repo_schema! {
///     // Elided for brevity
///     test_validation: {
///         <test_name> => {
///             input_json: json!(<value>),
///             expected_valid: <true|false>,
///         },
///         ...
///     },
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
/// # Examples
///
/// ## Minimal
///
/// The following snippet shows a minimal definition without any additional
/// property subschemas or documentation keywords.
///
/// ```rust, ignore
/// test_dsc_repo_schema! {
///     define_statics: {
///         my_crate::MyType
///     },
///     test_meta_schema: {},
///     test_docs: {},
///     test_validation: {
///         string_value_is_valid => {
///             input_json: json!("string value"),
///             expected_valid: true,
///         },
///         boolean_value_is_invalid => {
///             input_json: json!(true),
///             expected_valid: false,
///         },
///     },
/// }
/// ```
///
/// <details><summary>Expanded macro usage</summary>
///
/// ```rust, ignore
/// #[cfg(test)]
/// mod schemas {
///     static ROOT_SCHEMA: std::sync::LazyLock<schemars::Schema> = std::sync::LazyLock::new(|| {
///         use dsc_lib::schemas::dsc_repo::DscRepoSchema;
///         <my_crate::MyType>::generate_exportable_schema(
///             dsc_lib::schemas::dsc_repo::RecognizedSchemaVersion::latest(),
///             dsc_lib::schemas::dsc_repo::SchemaForm::VSCode,
///         )
///     });
///     static VALIDATOR: std::sync::LazyLock<jsonschema::Validator> = std::sync::LazyLock::new(|| {
///         use dsc_lib::schemas::dsc_repo::DscRepoSchema;
///         let ref schema = <my_crate::MyType>::generate_exportable_schema(
///             dsc_lib::schemas::dsc_repo::RecognizedSchemaVersion::latest(),
///             dsc_lib::schemas::dsc_repo::SchemaForm::Bundled,
///         );
///         match jsonschema::Validator::new(schema.as_value()) {
///             Ok(validator) => validator,
///             Err(err) => {
///                 core::panicking::panic_fmt(core::const_format_args!(
///                     "Failed to create JSON schema validator:\n---\nERROR: {:#?}\n---\nSCHEMA: {}\n---",
///                     err,
///                     serde_json::to_string_pretty(schema).unwrap(),
///                 ));
///             }
///         }
///     });
///     #[cfg(test)]
///     mod meta_schema {
///         use super::*;
///         #[test]
///         fn is_valid() {
///             let schema = ROOT_SCHEMA.as_value();
///             let result = jsonschema::meta::validate(schema);
///             {
///                 if !(result.is_ok()) {
///                     {
///                         core::panicking::panic_fmt(core::const_format_args!(
///                             "expected schema to be valid but got error: {:#?}",
///                             result.unwrap_err()
///                         ));
///                     };
///                 }
///             };
///         }
///     }
///     #[cfg(test)]
///     mod docs_keywords {
///         use super::*;
///         #[test]
///         fn title() {
///             crate::schemas::assert_schema_has_docs_keyword("title", &ROOT_SCHEMA.clone())
///         }
///         #[test]
///         fn description() {
///             crate::schemas::assert_schema_has_docs_keyword("description", &ROOT_SCHEMA.clone())
///         }
///         #[test]
///         fn markdown_description() {
///             crate::schemas::assert_schema_has_docs_keyword(
///                 "markdownDescription",
///                 &ROOT_SCHEMA.clone(),
///             )
///         }
///     }
///     #[cfg(test)]
///     mod validation {
///         use super::*;
///         #[test]
///         fn string_value_is_valid() {
///             crate::schemas::assert_schema_validation(
///                 &serde_json::to_value(&"string value").unwrap(),
///                 true,
///                 &VALIDATOR.clone(),
///                 &ROOT_SCHEMA.clone(),
///             );
///         }
///         #[test]
///         fn boolean_value_is_invalid() {
///             crate::schemas::assert_schema_validation(
///                 &serde_json::Value::Bool(true),
///                 false,
///                 &VALIDATOR.clone(),
///                 &ROOT_SCHEMA.clone(),
///             );
///         }
///     }
/// }
/// ```
///
/// </details>
///
/// ## With additional properties and keywords
///
/// The following snippet shows a definition that includes additional property
/// subschemas and documentation keywords.
///
/// ```rust, ignore
/// test_dsc_repo_schema! {
///     define_statics: {
///         my_crate::MyType,
///         properties: {
///             FOO_PROPERTY_SCHEMA => "foo",
///             BAR_PROPERTY_SCHEMA => "bar",
///         }
///     },
///     test_meta_schema: {},
///     test_docs: {
///         title => "title",
///         description => "description",
///         markdown_description => "markdownDescription",
///         deprecation_message => "deprecationMessage",
///         foo_title => (FOO_PROPERTY_SCHEMA, "title"),
///         foo_description => (FOO_PROPERTY_SCHEMA, "description"),
///         foo_markdown_description => (FOO_PROPERTY_SCHEMA, "markdownDescription"),
///         foo_error_message => (FOO_PROPERTY_SCHEMA, "errorMessage"),
///         bar_title => (BAR_PROPERTY_SCHEMA, "title"),
///         bar_description => (BAR_PROPERTY_SCHEMA, "description"),
///         bar_markdown_description => (BAR_PROPERTY_SCHEMA, "markdownDescription"),
///         bar_markdown_enum_descriptions => (BAR_PROPERTY_SCHEMA, "markdownEnumDescriptions"),
///     },
///     test_validation: {
///         empty_object_is_invalid => {
///             input_json: json!({}),
///             expected_valid: false,
///         },
///         foo_as_true_and_bar_as_baz_is_valid => {
///             input_json: json!({
///                 "foo": true,
///                 "bar": "baz"
///             }),
///             expected_valid: true,
///         },
///     },
/// }
/// ```
///
/// <details><summary>Expanded macro usage</summary>
///
/// ```rust, ignore
/// #[cfg(test)]
/// mod schemas {
///     static ROOT_SCHEMA: std::sync::LazyLock<schemars::Schema> = std::sync::LazyLock::new(|| {
///         use dsc_lib::schemas::dsc_repo::DscRepoSchema;
///         <my_crate::MyType>::generate_exportable_schema(
///             dsc_lib::schemas::dsc_repo::RecognizedSchemaVersion::latest(),
///             dsc_lib::schemas::dsc_repo::SchemaForm::VSCode,
///         )
///     });
///     static VALIDATOR: std::sync::LazyLock<jsonschema::Validator> = std::sync::LazyLock::new(|| {
///         use dsc_lib::schemas::dsc_repo::DscRepoSchema;
///         let ref schema = <my_crate::MyType>::generate_exportable_schema(
///             dsc_lib::schemas::dsc_repo::RecognizedSchemaVersion::latest(),
///             dsc_lib::schemas::dsc_repo::SchemaForm::Bundled,
///         );
///         match jsonschema::Validator::new(schema.as_value()) {
///             Ok(validator) => validator,
///             Err(err) => {
///                 core::panicking::panic_fmt(core::const_format_args!(
///                     "Failed to create JSON schema validator:\n---\nERROR: {:#?}\n---\nSCHEMA: {}\n---",
///                     err,
///                     serde_json::to_string_pretty(schema).unwrap(),
///                 ));
///             }
///         }
///     });
///     static FOO_PROPERTY_SCHEMA: std::sync::LazyLock<schemars::Schema> =
///         std::sync::LazyLock::new(|| {
///             use dsc_lib::schemas::schema_utility_extensions::SchemaUtilityExtensions;
///             ROOT_SCHEMA
///                 .clone()
///                 .get_property_subschema("foo")
///                 .expect(&alloc::__export::must_use({
///                     alloc::fmt::format(alloc::__export::format_args!(
///                         "properties.{} should be defined",
///                         "foo"
///                     ))
///                 }))
///                 .clone()
///         });
///     static BAR_PROPERTY_SCHEMA: std::sync::LazyLock<schemars::Schema> =
///         std::sync::LazyLock::new(|| {
///             use dsc_lib::schemas::schema_utility_extensions::SchemaUtilityExtensions;
///             ROOT_SCHEMA
///                 .clone()
///                 .get_property_subschema("bar")
///                 .expect(&alloc::__export::must_use({
///                     alloc::fmt::format(alloc::__export::format_args!(
///                         "properties.{} should be defined",
///                         "bar"
///                     ))
///                 }))
///                 .clone()
///         });
///     #[cfg(test)]
///     mod meta_schema {
///         use super::*;
///         #[test]
///         fn is_valid() {
///             let schema = ROOT_SCHEMA.as_value();
///             let result = jsonschema::meta::validate(schema);
///             {
///                 if !(result.is_ok()) {
///                     {
///                         core::panicking::panic_fmt(core::const_format_args!(
///                             "expected schema to be valid but got error: {:#?}",
///                             result.unwrap_err()
///                         ));
///                     };
///                 }
///             };
///         }
///     }
///     #[cfg(test)]
///     mod docs_keywords {
///         use super::*;
///         #[test]
///         fn title() {
///             crate::schemas::assert_schema_has_docs_keyword("title", &ROOT_SCHEMA.clone())
///         }
///         #[test]
///         fn description() {
///             crate::schemas::assert_schema_has_docs_keyword("description", &ROOT_SCHEMA.clone())
///         }
///         #[test]
///         fn markdown_description() {
///             crate::schemas::assert_schema_has_docs_keyword(
///                 "markdownDescription",
///                 &ROOT_SCHEMA.clone(),
///             )
///         }
///         #[test]
///         fn deprecation_message() {
///             crate::schemas::assert_schema_has_docs_keyword(
///                 "deprecationMessage",
///                 &ROOT_SCHEMA.clone(),
///             )
///         }
///         #[test]
///         fn foo_title() {
///             crate::schemas::assert_schema_has_docs_keyword("title", &FOO_PROPERTY_SCHEMA.clone())
///         }
///         #[test]
///         fn foo_description() {
///             crate::schemas::assert_schema_has_docs_keyword(
///                 "description",
///                 &FOO_PROPERTY_SCHEMA.clone(),
///             )
///         }
///         #[test]
///         fn foo_markdown_description() {
///             crate::schemas::assert_schema_has_docs_keyword(
///                 "markdownDescription",
///                 &FOO_PROPERTY_SCHEMA.clone(),
///             )
///         }
///         #[test]
///         fn foo_error_message() {
///             crate::schemas::assert_schema_has_docs_keyword(
///                 "errorMessage",
///                 &FOO_PROPERTY_SCHEMA.clone(),
///             )
///         }
///         #[test]
///         fn bar_title() {
///             crate::schemas::assert_schema_has_docs_keyword("title", &BAR_PROPERTY_SCHEMA.clone())
///         }
///         #[test]
///         fn bar_description() {
///             crate::schemas::assert_schema_has_docs_keyword(
///                 "description",
///                 &BAR_PROPERTY_SCHEMA.clone(),
///             )
///         }
///         #[test]
///         fn bar_markdown_description() {
///             crate::schemas::assert_schema_has_docs_keyword(
///                 "markdownDescription",
///                 &BAR_PROPERTY_SCHEMA.clone(),
///             )
///         }
///         #[test]
///         fn bar_markdown_enum_descriptions() {
///             crate::schemas::assert_schema_has_docs_keyword(
///                 "markdownEnumDescriptions",
///                 &BAR_PROPERTY_SCHEMA.clone(),
///             )
///         }
///     }
///     #[cfg(test)]
///     mod validation {
///         use super::*;
///         #[test]
///         fn empty_object_is_invalid() {
///             crate::schemas::assert_schema_validation(
///                 &serde_json::Value::Object(serde_json::Map::new()),
///                 false,
///                 &VALIDATOR.clone(),
///                 &ROOT_SCHEMA.clone(),
///             );
///         }
///         #[test]
///         fn foo_as_true_and_bar_as_baz_is_valid() {
///             crate::schemas::assert_schema_validation(
///                 &serde_json::Value::Object({
///                     let mut object = serde_json::Map::new();
///                     let _ = object.insert(("foo").into(), (serde_json::Value::Bool(true)));
///                     let _ = object.insert(("bar").into(), (serde_json::to_value(&"baz").unwrap()));
///                     object
///                 }),
///                 true,
///                 &VALIDATOR.clone(),
///                 &ROOT_SCHEMA.clone(),
///             );
///         }
///     }
/// }
/// ```
///
/// </details>
///
/// [`define_schema_statics!`]: crate::macros::define_schema_statics!
/// [`test_meta_schema!`]: crate::macros::test_meta_schema!
/// [`test_schema_docs!`]: crate::macros::test_schema_docs!
/// [`test_schema_validation!`]: crate::macros::test_schema_validation!
macro_rules! test_dsc_repo_schema {
    (
        define_statics: {$($definition:tt)*},
        test_meta_schema: {$($test_meta_schema:tt)*},
        test_docs: {$($test_docs:tt)*},
        test_validation: {$($test_validation:tt)*}
        $(,)?
    ) => {
        #[cfg(test)]
        mod schemas {
            $crate::macros::define_schema_statics! {
                $($definition)*
            }
            $crate::macros::test_meta_schema! {
                $($test_meta_schema)*
            }
            $crate::macros::test_schema_docs! {
                $($test_docs)*
            }
            $crate::macros::test_schema_validation! {
                $($test_validation)*
            }
        }
    };
}

pub(crate) use test_dsc_repo_schema;
