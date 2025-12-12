// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::{Schema, json_schema};

mod dsc_repo_schema;
pub use dsc_repo_schema::DscRepoSchema;
pub use dsc_repo_schema::UnrecognizedSchemaUri;

mod recognized_schema_version;
pub use recognized_schema_version::RecognizedSchemaVersion;

mod schema_form;
pub use schema_form::SchemaForm;

mod schema_uri_prefix;
pub use schema_uri_prefix::SchemaUriPrefix;

pub use dsc_lib_jsonschema_macros::DscRepoSchema;

/// Returns the constructed URI for a hosted DSC schema.
///
/// This convenience function simplifies constructing the URIs for the various published schemas
/// that DSC recognizes, instead of needing to maintain long lists of those recognized schemas.
/// This function should primarily be called by [`get_recognized_schema_uris`], not called
/// directly.
///
/// Parameters:
///
/// - `schema_file_base_name` - specify the base name for the schema file, like `document` for
///   the configuration document schema or `manifest` for the resource manifest schema.
/// - `schema_folder_path` - specify the folder path for the schema file relative to the version
///   folder, like `config` for the configuration document schema or `resource` for the resource
///   manifest schema.
/// - `schema_version` - specify the version of the schema.
/// - `schema_form` - specify whether the schema is bundled, for VS Code, or is the canonical
///   (non-bundled) schema.
/// - `uri_prefix` - Specify whether the URI should be prefixed for `aka.ms` or GitHub.
pub(crate) fn get_recognized_schema_uri(
    schema_file_base_name: &str,
    schema_folder_path: &str,
    schema_version: RecognizedSchemaVersion,
    schema_form: SchemaForm,
    schema_uri_prefix: SchemaUriPrefix
) -> String {
    format!(
        "{schema_uri_prefix}/{schema_version}/{}{schema_folder_path}/{schema_file_base_name}{}",
        schema_form.to_folder_prefix(),
        schema_form.to_extension()
    )
}

/// Returns the vector of recognized URIs for a given schema.
///
/// This convenience function generates a vector containing every recognized JSON Schema `$id` URI
/// for a specific schema. It handles returning the schemas for every recognized host, version,
/// and form.
///
/// Parameters:
///
/// - `schema_file_base_name` - specify the base name for the schema file, like `document` for
///   the configuration document schema or `manifest` for the resource manifest schema.
/// - `schema_folder_path` - specify the folder path for the schema file relative to the version
///   folder, like `config` for the configuration document schema or `resource` for the resource
///   manifest schema.
/// - `should_bundle` - specify whether the schema should be published in its bundled form. All
///   bundled schemas are also published with their VS Code form. Schemas that aren't bundled
///   aren't published with the VS Code form.
pub(crate) fn get_recognized_schema_uris(
    schema_file_base_name: &str,
    schema_folder_path: &str,
    should_bundle: bool
) -> Vec<String> {
    let mut uris: Vec<String> = Vec::new();
    let schema_forms = if should_bundle {
        SchemaForm::all()
    } else {
        vec![SchemaForm::Canonical]
    };
    for uri_prefix in SchemaUriPrefix::all() {
        for schema_form in schema_forms.iter().copied() {
            for schema_version in RecognizedSchemaVersion::all() {
                uris.push(
                    get_recognized_schema_uri(
                        schema_file_base_name,
                        schema_folder_path,
                        schema_version,
                        schema_form,
                        uri_prefix
                    )
                );
            }
        }
    }

    uris
}

/// Returns the JSON Schema to validate that a `$schema` keyword for a DSC type is one of the
/// recognized URIs.
///
/// This is a convenience function used by the [`DscRepoSchema`] trait. It's not intended for
/// direct use.
#[must_use]
pub(crate) fn get_recognized_uris_subschema(
    metadata: &Schema,
    schema_file_base_name: &str,
    schema_folder_path: &str,
    should_bundle: bool
) -> Schema {
    let enums: Vec<serde_json::Value> = get_recognized_schema_uris(
        schema_file_base_name,
        schema_folder_path,
        should_bundle
    ).iter().map(
        |schema_uri| serde_json::Value::String(schema_uri.clone())
    ).collect();

    let mut subschema = json_schema!({
        "type": "string",
        "format": Some("uri".to_string()),
        "enum": Some(enums),
    });
    
    let annotation_keywords = [
        "title",
        "description",
        "markdownDescription",
        "enumMarkdownDescriptions",
        "enumDescriptions",
        "completionDetail",
        "defaultSnippets",
        "enumDetails",
        "enumSortTexts",
        "suggestSortText",
        "deprecationMessage",
        "errorMessage",
    ];
    for annotation_keyword in annotation_keywords {
        if let Some(value) = metadata.get(annotation_keyword) {
            subschema.insert(annotation_keyword.to_string(), value.clone());
        }
    }

    subschema
}

/// Returns the recognized schema URI for the latest major version with the
/// `aka.ms` URI prefix.
///
/// If the schema is published in bundled form, this function returns the URI for that form.
/// Otherwise, it returns the URI for the canonical (non-bundled) form. The VS Code form of the
/// schema is never returned as the default.
///
/// Parameters:
///
/// - `schema_file_base_name` - specify the base name for the schema file, like `document` for
///   the configuration document schema or `manifest` for the resource manifest schema.
/// - `schema_folder_path` - specify the folder path for the schema file relative to the version
///   folder, like `config` for the configuration document schema or `resource` for the resource
///   manifest schema.
/// - `should_bundle` - specify whether the schema should be published in its bundled form. All
///   bundled schemas are also published with their VS Code form. Schemas that aren't bundled
///   aren't published with the VS Code form.
pub(crate) fn get_default_schema_uri(
    schema_file_base_name: &str,
    schema_folder_path: &str,
    should_bundle: bool
) -> String {
    get_recognized_schema_uri(
        schema_file_base_name,
        schema_folder_path,
        RecognizedSchemaVersion::default(),
        get_default_schema_form(should_bundle),
        SchemaUriPrefix::default()
    )
}

/// Returns the default form for a schema depending on whether it publishes with its references
/// bundled.
///
/// If a schema is published in bundled form, the bundled form is the default. Otherwise, the
/// default form is canonical (non-bundled).
pub(crate) fn get_default_schema_form(should_bundle: bool) -> SchemaForm {
    if should_bundle {
        SchemaForm::Bundled
    } else {
        SchemaForm::Canonical
    }
}
