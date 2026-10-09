// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Generates tests for schema documentation keywords.
///
/// This macro generates tests for schema documentation keywords, ensuring that
/// they're defined and have valid translations.
///
/// <div class="info">
///
/// This macro is best used through the [`test_dsc_repo_schema!`] macro. Only
/// use this macro directly when the higher level macro is unsuitable.
///
/// </div>
///
/// The available keywords are:
///
/// - `title`
/// - `description`
/// - `markdownDescription`
/// - `enumDescriptions`
/// - `markdownEnumDescriptions`
/// - `errorMessage`
/// - `patternErrorMessage`
/// - `deprecationMessage`
///
/// The default set is `[title, description, markdownDescription]`.
///
/// # Examples
///
/// ## Without arguments
///
/// ```rust, ignore
/// test_schema_docs!();
/// ```
///
/// <details><summary>Expanded macro</summary>
///
/// ```rust, ignore
/// #[cfg(test)]
/// mod docs_keywords {
///     use super::*;
///     #[test]
///     fn title() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "title",
///             &ROOT_SCHEMA.clone()
///         )
///     }
///     #[test]
///     fn description() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "description",
///             &ROOT_SCHEMA.clone()
///         )
///     }
///     #[test]
///     fn markdown_description() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "markdownDescription",
///             &ROOT_SCHEMA.clone(),
///         )
///     }
/// }
/// ```
///
/// </details>
///
/// ## With specific keywords
///
/// ```rust, ignore
/// test_schema_docs!{
///     title => "title",
///     description => "description",
///     markdown_description => "markdownDescription",
///     deprecation_message => "deprecationMessage",
/// }
/// ```
///
/// <details><summary>Expanded macro</summary>
///
/// ```rust, ignore
/// #[cfg(test)]
/// mod docs_keywords {
///     use super::*;
///     #[test]
///     fn title() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "title",
///             &ROOT_SCHEMA.clone()
///         )
///     }
///     #[test]
///     fn description() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "description",
///             &ROOT_SCHEMA.clone()
///         )
///     }
///     #[test]
///     fn markdown_description() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "markdownDescription",
///             &ROOT_SCHEMA.clone(),
///         )
///     }
///     #[test]
///     fn deprecation_message() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "deprecationMessage",
///             &ROOT_SCHEMA.clone(),
///         )
///     }
/// }
/// ```
///
/// ## With specific keywords and schemas
///
/// ```rust, ignore
/// test_schema_docs!{
///    title => "title",
///    description => "description",
///    markdown_description => "markdownDescription",
///    tracing_title => (TRACING_PROPERTY_SCHEMA, "title"),
///    tracing_description => (TRACING_PROPERTY_SCHEMA, "description"),
///    tracing_markdown_description => (TRACING_PROPERTY_SCHEMA, "markdownDescription"),
///    resource_path_title => (RESOURCE_PATH_PROPERTY_SCHEMA, "title"),
///    resource_path_description => (RESOURCE_PATH_PROPERTY_SCHEMA, "description"),
///    resource_path_markdown_description => (RESOURCE_PATH_PROPERTY_SCHEMA, "markdownDescription"),
/// }
/// ```
///
/// <details><summary>Expanded macro</summary>
///
/// ```rust, ignore
/// #[cfg(test)]
/// mod docs_keywords {
///     use super::*;
///     #[test]
///     fn title() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "title",
///             &ROOT_SCHEMA.clone()
///         )
///     }
///     #[test]
///     fn description() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "description",
///             &ROOT_SCHEMA.clone()
///         )
///     }
///     #[test]
///     fn markdown_description() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "markdownDescription",
///             &ROOT_SCHEMA.clone(),
///         )
///     }
///     #[test]
///     fn tracing_title() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "title",
///             &TRACING_PROPERTY_SCHEMA.clone(),
///         )
///     }
///     #[test]
///     fn tracing_description() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "description",
///             &TRACING_PROPERTY_SCHEMA.clone(),
///         )
///     }
///     #[test]
///     fn tracing_markdown_description() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "markdownDescription",
///             &TRACING_PROPERTY_SCHEMA.clone(),
///         )
///     }
///     #[test]
///     fn resource_path_title() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "title",
///             &RESOURCE_PATH_PROPERTY_SCHEMA.clone(),
///         )
///     }
///     #[test]
///     fn resource_path_description() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "description",
///             &RESOURCE_PATH_PROPERTY_SCHEMA.clone(),
///         )
///     }
///     #[test]
///     fn resource_path_markdown_description() {
///         crate::schemas::assert_schema_has_docs_keyword(
///             "markdownDescription",
///             &RESOURCE_PATH_PROPERTY_SCHEMA.clone(),
///         )
///     }
/// }
/// ```
///
/// [`test_dsc_repo_schema!`]: crate::macros::test_dsc_repo_schema
macro_rules! test_schema_docs {
    // `test_schema_docs!()` uses the default set of keywords for the root schema.
    () => {
        $crate::macros::test_schema_docs!{
            title => "title",
            description => "description",
            markdown_description => "markdownDescription"
        }
    };
    // `test_schema_docs!(title => "title", property_title => (PROPERTY_SCHEMA, "title"))`
    // Generates a test for each case provided.
    ($($test_name:ident => $test_info:tt),+ $(,)?) => {
        #[cfg(test)] mod docs_keywords {
            use super::*;
            $crate::macros::test_schema_docs!{
                $($test_name: $test_info),+
            }
        }
    };
    /**************** Incremental munchers for each test case *****************/
    ( // Muncher for a keyword in the root schema
        $test_name:ident: $keyword:literal
        $(, $($rest:tt)*)?
    ) => {
        $crate::macros::test_schema_docs!{
            $test_name: (ROOT_SCHEMA, $keyword)
        }

        $($crate::macros::test_schema_docs!($($rest)*);)?
    };
    ( // Muncher for a keyword in a specific static schema
        $test_name:ident: ($static_schema:ident, $keyword:literal)
        $(, $($rest:tt)*)?
    ) => {
        #[test] fn $test_name() {
            $crate::schemas::assert_schema_has_docs_keyword(
                $keyword,
                &$static_schema.clone()
            )
        }

        $($crate::macros::test_schema_docs!($($rest)*);)?
    };
}

pub(crate) use test_schema_docs;
