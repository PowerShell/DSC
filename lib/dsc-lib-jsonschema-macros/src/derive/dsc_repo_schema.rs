// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Path, parse_macro_input};

use crate::ast::StringOrExpr;

/// Defines the top-level input for the derive macro as `dsc_repo_schema`.
#[derive(Clone, FromDeriveInput)]
#[darling(attributes(dsc_repo_schema))]
struct DscRepoSchemaReceiver {
    /// Defines the base name for the schema file, like `exist`. Must be a literal string.
    pub base_name: String,
    /// Defines the folder path relative to the version folder for the schema file, like
    /// `resource/properties`. Must be a literal string.
    pub folder_path: String,
    /// Defines whether the schema should be bundled. Root schemas should always be bundled, but
    /// other schemas may not be.
    #[darling(default)]
    pub should_bundle: bool,
    /// Defines the field for the struct that is used as the `$schema` property. Typically only
    /// defined for root schemas.
    #[darling(default)]
    pub schema_field: Option<DscRepoSchemaField>,
}

/// Defines the suboptions for the `schema_field` option on the derive macro.
#[derive(FromMeta, Clone)]
#[darling(derive_syn_parse)]
struct DscRepoSchemaField {
    /// Defines the field that is used as the `$schema` property. This should be the name of field,
    /// like `schema_version`. It's used to generate the validation function.
    pub name: Path,
    /// Defines the `title` keyword for the `$schema` property.
    #[darling(default)]
    pub title: Option<StringOrExpr>,
    /// Defines the `description` keyword for the `$schema` property.
    #[darling(default)]
    pub description: Option<StringOrExpr>,
    /// Defines the `markdownDescription` keyword for the `$schema` property.
    #[darling(default)]
    pub markdown_description: Option<StringOrExpr>,
}

/// Implements the `DscRepoSchema` trait for a type with the derive macro.
pub(crate) fn dsc_repo_schema_impl(input: TokenStream) -> TokenStream {
    // Parse input token stream as `DeriveInput`
    let original = parse_macro_input!(input as DeriveInput);

    // Destructure the input to get the identity of the type the macro was used on.
    let DeriveInput { ident, .. } = original.clone();

    // Parse the attribute at the top level of the type to retrieve the necessary information.
    let args = match DscRepoSchemaReceiver::from_derive_input(&original) {
        Ok(v) => v,
        Err(e) => {
            // Return the error as a token stream for better diagnostics.
            return TokenStream::from(e.write_errors());
        }
    };

    let mut output = quote!();

    if let Some(schema_field) = args.schema_field {
        output.extend(generate_with_schema_field(
            ident,
            args.base_name,
            args.folder_path,
            args.should_bundle,
            schema_field
        ));
    } else {
        output.extend(generate_without_schema_field(
            ident,
            args.base_name,
            args.folder_path,
            args.should_bundle
        ));
    }

    output.into()
}


/// Generates the minimal implementation of the `DscRepoSchema` trait when the type doesn't define
/// the `schema_field` option in the macro attribute.
fn generate_without_schema_field(
    ident: Ident,
    base_name: String,
    folder_path: String,
    should_bundle: bool
) -> proc_macro2::TokenStream {
    quote!(
        #[automatically_derived]
        impl DscRepoSchema for #ident {
            const SCHEMA_FILE_BASE_NAME: &'static str = #base_name;
            const SCHEMA_FOLDER_PATH: &'static str = #folder_path;
            const SCHEMA_SHOULD_BUNDLE: bool = #should_bundle;

            fn schema_property_metadata() -> schemars::Schema {
                schemars::json_schema!({})
            }
        }
    )
}

/// Generates the implementation of the `DscRepoSchema` trait for a type that defines the
/// `schema_field` option in the macro attribute. This is typically used for root schemas, like the
/// configuration document or resource manifest.
/// 
/// It generates the trait implementation with the associated constants, the metadata for the field,
/// and the schema URI validation function.
fn generate_with_schema_field(
    ident: Ident,
    base_name: String,
    folder_path: String,
    should_bundle: bool,
    schema_field: DscRepoSchemaField
) -> proc_macro2::TokenStream {
    let schema_property_metadata = generate_schema_property_metadata_fn(&schema_field);
    let field = schema_field.name;
    quote!(
        #[automatically_derived]
        impl DscRepoSchema for #ident {
            const SCHEMA_FILE_BASE_NAME: &'static str = #base_name;
            const SCHEMA_FOLDER_PATH: &'static str = #folder_path;
            const SCHEMA_SHOULD_BUNDLE: bool = #should_bundle;

            #schema_property_metadata

            fn validate_schema_uri(&self) -> Result<(), dsc_lib_jsonschema::dsc_repo::UnrecognizedSchemaUri> {
                if Self::is_recognized_schema_uri(&self.#field) {
                    Ok(())
                } else {
                    Err(dsc_lib_jsonschema::dsc_repo::UnrecognizedSchemaUri(
                        self.#field.clone(),
                        Self::recognized_schema_uris(),
                    ))
                }
            }
        }
    )
}

/// Generates the implementation for the `schema_property_metadata` trait function, inserting the
/// defined keywords into the schema.
fn generate_schema_property_metadata_fn(schema_field: &DscRepoSchemaField) -> proc_macro2::TokenStream {
    let mut schema_body = quote!();
    let fields = schema_field.clone();

    if let Some(title) = fields.title {
        schema_body.extend(quote!{"title": #title,});
    }
    if let Some(description) = fields.description {
        schema_body.extend(quote!{"description": #description,});
    }
    if let Some(markdown_description) = fields.markdown_description {
        schema_body.extend(quote!{"markdownDescription": #markdown_description,});
    }

    quote!{
        fn schema_property_metadata() -> schemars::Schema {
            schemars::json_schema!({
                #schema_body
            })
        }
    }
}
