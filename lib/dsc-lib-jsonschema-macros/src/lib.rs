// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Provides procedural macros for dsc-lib-jsonschema.

extern crate proc_macro;

use proc_macro::TokenStream;

pub(crate) mod derive {
    pub(crate) mod dsc_repo_schema;
}

pub(crate) mod ast {
    mod string_or_expr;
    pub(crate) use string_or_expr::StringOrExpr;
}

#[cfg(test)] mod test {
    #[cfg(test)] mod ast {
        #[cfg(test)] mod string_or_expr;
    }
}

/// Derives the `DscRepoSchema` trait.
/// 
/// This is only intended for use in types defined within the `PowerShell/Dsc` repository. It
/// simplifies defining the schemas and extracting them for publishing.
/// 
/// You can use this derive macro on structs and enums.
/// 
/// # Required Attributes
/// 
/// - `base_name`: The base name for the schema file, like `exist`.
/// - `folder_path`: The folder path relative to the version folder, like `resource/properties`.
///
/// # Optional Attributes
/// 
/// - `should_bundle`: Whether the schema should be bundled. A bundled schema includes every
///   referenced schema in the `$defs` keyword to minimize network calls. This typically only
///   applies to root schemas, like for a configuration document.
/// 
///   When this attribute isn't specified, the default is `false`.
/// - `schema_field`: Settings for the `$schema` property of a root schema. This option has the
///   following fields:
/// 
///   - `name` (required) - Must be the literal name of the struct field that maps to the `$schema`
///     property, like `schema` or `schema_version`. The derive macro uses this to validate the
///     value against the recognized schema URIs for the type.
///   - `title` (optional) - Defines the `title` keyword for the `$schema` property subschema. You
///     can specify the value as a string literal or an expression that resolves to a string.
///   - `description` (optional) - Defines the `description` keyword for the `$schema` property
///     subschema. You can specify the value as a string literal or an expression that resolves to
///     a string.
///   - `markdown_description` (optional) - Defines the `markdownDescription` keyword for the
///     `$schema` property subschema. You can specify the value as a string literal or an
///     expression that resolves to a string.
///
/// # Examples
/// 
/// The following examples show how you can derive the `DscRepoSchema` trait for different types.
/// 
/// ## Without schema field or bundling
/// 
/// In this example, the `Resource` struct derives the `DscRepoSchema` trait without bundling or a
/// schema field. Given the values for `base_name` and `folder_path`, the canonical URI to this
/// schema for the `v3` version folder is `https://aka.ms/dsc/schemas/v3/config/document.resource.json`.
/// 
/// ```ignore
/// #[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
/// #[serde(deny_unknown_fields)]
/// #[dsc_repo_schema(base_name = "document.resource", folder_path = "config")]
/// struct Resource {
///     // ...
/// }
/// ```
/// 
/// This is typically all that is required for types that don't define a root schema.
/// 
/// ## With schema field and bundling
/// 
/// In this example, the `Configuration` struct is a root schema. The `dsc_repo_schema` attribute
/// defines information for the schema field as well as the root document.
/// 
/// ```ignore
/// #[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
/// #[serde(deny_unknown_fields)]
/// #[dsc_repo_schema(
///     base_name = "document",
///     folder_path = "config",
///     should_bundle = true,
///     schema_field(
///         name = schema,
///         title = t!("schemas.configuration.document.properties.$schema.title"),
///         description = t!("schemas.configuration.document.properties.$schema.description"),
///         markdown_description = t!("schemas.configuration.document.properties.$schema.markdownDescription")
///     )
/// )]
/// pub struct Configuration {
///     #[serde(rename = "$schema")]
///     #[schemars(schema_with = "Configuration::recognized_schema_uris_subschema")]
///     pub schema: String,
///     // ...
/// }
/// ```
#[proc_macro_derive(DscRepoSchema, attributes(dsc_repo_schema))]
pub fn derive_into_dsc_repo_schema(item: TokenStream) -> TokenStream {
    derive::dsc_repo_schema::dsc_repo_schema_impl(item)
}
