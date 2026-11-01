// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// A macro for retrieving internationalized schema strings from a dotpath.
///
/// This macro is a helper for calling [`DscRepoSchema::schema_i18n`] to
/// retrieve the translated string for schema documentation keywords. It's
/// intended to be used either in the `schemars` attribute or directly in the
/// manual implementation for [`schemars::JsonSchema`].
///
/// # Syntax
///
/// This macro accepts three forms of input:
///
/// - `schema_i18n!("dot.path.to.translation_key")` - Uses the provided string
///   as the suffix to [`DscRepoSchema::SCHEMA_I18N_ROOT_KEY`].
/// - `schema_i18n!("root", "suffix", "another_suffix")` - Concatenates the
///   root and suffixes with dots and uses the result as the suffix to
///   [`DscRepoSchema::SCHEMA_I18N_ROOT_KEY`].
/// - `schema_i18n!(expression)` - Passes the expression directly as the suffix
///   to [`DscRepoSchema::SCHEMA_I18N_ROOT_KEY`].
///
/// # Example
///
/// This example demonstrates the alternate forms for using the macro when
/// defining a type that derives `JsonSchema` and `DscRepoSchema`, which
/// is the most common case for types in the DSC repository.
///
/// ```rust, ignore
/// #[derive(Debug, Clone, JsonSchema, DscRepoSchema)]
/// #[dsc_repo_schema(base_name = "document", folder_path = "examples/macros")]
/// #[serde(rename_all = "camelCase")]
/// #[schemars(
///     title = schema_i18n!("title"),
///     description = schema_i18n("description"),
///     extend("markdownDescription" = schema_i18n("markdownDescription"))
/// )]
/// pub struct Example {
///     /// The `foo` field passes the dot-path literal for the macro
///     #[schemars(
///         title = schema_i18n!("fields.foo.title"),
///         description = schema_i18n!("foo.description"),
///         extend("markdownDescription" = schema_i18n!("foo.markdownDescription"))
///     )]
///     pub foo: String
///     /// The `bar` field passes each segment as a separate literal to the
///     /// macro.
///     #[schemars(
///         title = schema_i18n!("fields", "bar", "title"),
///         description = schema_i18n!("fields", "bar", "description"),
///         extend("markdownDescription" = schema_i18n!("fields", "bar", "markdownDescription"))
///     )]
///     pub bar: bool,
///     /// The `baz` field demonstrates uses the [`format!`] macro to construct
///     /// the dot-path for the macro dynamically.
///     #[schemars(
///         title = schema_i18n!(format!("fields.{}.title", "baz")),
///         description = schema_i18n!(format!("fields.{}.description", "baz")),
///         extend(
///             "markdownDescription" = schema_i18n!(
///                 format!("fields.{}.markdownDescription", "baz")
///             )
///         )
///     )]
///     pub baz: usize,
/// }
/// ```
///
/// The type is defined with the following `dsc_repo_schema` attribute:
///
/// ```rust, ignore
/// #[dsc_repo_schema(base_name = "document", folder_path = "examples/macros")]
/// ```
///
/// The derive macro therefore defines [`DscRepoSchema::SCHEMA_I18N_ROOT_KEY`]
/// as `"schemas.examples.macros.document"`.
///
/// With that, we can see the full dot paths for the translation entries that
/// need to be defined for this schema:
///
/// | Macro call                                                      | Full dot path                                                     |
/// |:----------------------------------------------------------------|:------------------------------------------------------------------|
/// | `schema_i18n!("title")`                                         | `schemas.examples.macros.document.title`                          |
/// | `schema_i18n!("description")`                                   | `schemas.examples.macros.document.description`                    |
/// | `schema_i18n!("markdownDescription")`                           | `schemas.examples.macros.document.markdownDescription`            |
/// | `schema_i18n!("fields.foo.title")`                              | `schemas.examples.macros.document.fields.foo.title`               |
/// | `schema_i18n!("fields.foo.description")`                        | `schemas.examples.macros.document.fields.foo.description`         |
/// | `schema_i18n!("fields.foo.markdownDescription")`                | `schemas.examples.macros.document.fields.foo.markdownDescription` |
/// | `schema_i18n!("fields", "bar", "title")`                        | `schemas.examples.macros.document.fields.bar.title`               |
/// | `schema_i18n!("fields", "bar", "description")`                  | `schemas.examples.macros.document.fields.bar.description`         |
/// | `schema_i18n!(format!("fields.{}.title", "baz"))`               | `schemas.examples.macros.document.fields.baz.title`               |
/// | `schema_i18n!(format!("fields.{}.description", "baz"))`         | `schemas.examples.macros.document.fields.baz.description`         |
/// | `schema_i18n!(format!("fields.{}.markdownDescription", "baz"))` | `schemas.examples.macros.document.fields.baz.markdownDescription` |
///
/// Which should be structured in YAML using the `_version: 2` option for
/// [`rust_i18n`] like so:
///
/// ```yaml
/// _version: 2
/// schemas:
///   examples:
///     macros:
///       document:
///         title:
///           en-us: <title>
///         description:
///           en-us: <description>
///         markdownDescription:
///           en-us: <markdownDescription>
///         fields:
///           foo:
///             title:
///               en-us: <fields.foo.title>
///             description:
///               en-us: <fields.foo.description>
///             markdownDescription:
///               en-us: <fields.foo.markdownDescription>
///           bar:
///             title:
///               en-us: <fields.bar.title>
///             description:
///               en-us: <fields.bar.description>
///             markdownDescription:
///               en-us: <fields.bar.markdownDescription>
///           baz:
///             title:
///               en-us: <fields.baz.title>
///             description:
///               en-us: <fields.baz.description>
///             markdownDescription:
///               en-us: <fields.baz.markdownDescription>
/// ```
///
/// [`DscRepoSchema::schema_i18n`]: crate::dsc_repo::DscRepoSchema::schema_i18n
/// [`DscRepoSchema::SCHEMA_I18N_ROOT_KEY`]: crate::dsc_repo::DscRepoSchema::SCHEMA_I18N_ROOT_KEY
#[macro_export]
macro_rules! schema_i18n {
    // When the dotpath is just a literal, pass it through to `Self::schema_i18n` directly.
    ($dot_path: literal) => {
        Self::schema_i18n($dot_path).unwrap()
    };
    // When the dotpath is a root with one or more suffixes, concatenate them with dots.
    ($root: literal, $($suffix: literal),+ ) => {
        Self::schema_i18n(concat!($root, $( ".", $suffix ),+)).unwrap()
    };
    // When the dotpath is an arbitrary expression, pass it through to `Self::schema_i18n` directly.
    ($dot_path: expr) => {
        Self::schema_i18n($dot_path).unwrap()
    };
}
