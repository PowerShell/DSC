// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Generates tests to ensure that a given JSON schema is valid according to
/// the meta-schema.
///
/// This macro generates the `meta_schema::is_valid` test function. It uses the
/// [`jsonschema::meta::validate`] function. Without any arguments, the macro
/// tests the `ROOT_SCHEMA`. To test a specific schema, pass the static variable
/// representing that schema as an argument to the macro.
///
/// <div class="info">
///
/// This macro is best used through the [`test_dsc_repo_schema!`] macro. Only
/// use this macro directly when the higher level macro is unsuitable.
///
/// </div>
///
/// # Examples
///
/// When you use the macro without any arguments, it defaults to testing the
/// `ROOT_SCHEMA`.
///
/// ```rust, ignore
/// crate::macros::test_meta_schema!();
/// ```
///
/// <details><summary>Expanded macro</summary>
///
/// ```rust, ignore
/// #[cfg(test)]
/// mod meta_schema {
///     use super::*;
///     #[test]
///     fn is_valid() {
///         let schema = ROOT_SCHEMA.as_value();
///         let result = jsonschema::meta::validate(schema);
///         {
///             if !(result.is_ok()) {
///                 {
///                     core::panicking::panic_fmt(core::const_format_args!(
///                         "expected schema to be valid but got error: {:#?}",
///                         result.unwrap_err()
///                     ));
///                 };
///             }
///         };
///     }
/// }
/// ```
///
/// </details>
///
/// When you specify a specific static schema, the macro will generate a test
/// for that schema instead of the `ROOT_SCHEMA`.
///
/// ```rust, ignore
/// crate::macros::test_meta_schema!(FOO_PROPERTY_SCHEMA);
/// ```
///
/// <details><summary>Expanded macro</summary>
///
/// ```rust, ignore
/// #[cfg(test)]
/// mod meta_schema {
///     use super::*;
///     #[test]
///     fn is_valid() {
///         let schema = FOO_PROPERTY_SCHEMA.as_value();
///         let result = jsonschema::meta::validate(schema);
///         {
///             if !(result.is_ok()) {
///                 {
///                     core::panicking::panic_fmt(core::const_format_args!(
///                         "expected schema to be valid but got error: {:#?}",
///                         result.unwrap_err()
///                     ));
///                 };
///             }
///         };
///     }
/// }
/// ```
///
/// </details>
///
/// [`test_dsc_repo_schema!`]: crate::macros::test_dsc_repo_schema
macro_rules! test_meta_schema {
    () => {
        $crate::macros::test_meta_schema!(@fn ROOT_SCHEMA);
    };
    ($schema_static:ident) => {
        $crate::macros::test_meta_schema!(@fn $schema_static);
    };
    (@fn $schema_static:ident) => {
        #[cfg(test)]
        mod meta_schema {
            use super::*;

            #[test]
            fn is_valid() {
                let schema = $schema_static.as_value();
                let result = jsonschema::meta::validate(schema);
                assert!(
                    result.is_ok(),
                    "expected schema to be valid but got error: {:#?}",
                    result.unwrap_err()
                );
            }
        }
    };
}

pub(crate) use test_meta_schema;
