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

use crate::schema_utility_extensions::SchemaUtilityExtensions;

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

/// Retrieves the version segment from the `$id` keyword of a DSC repo schema.
pub(crate) fn get_schema_id_version(schema: &Schema) -> Option<String> {
    let Some(root_id) = schema.get_id() else {
        return None;
    };

    // Remove the URI prefix and leading slash to get the URI relative to the `schemas` folder
    let schema_folder_relative_id = root_id
        .trim_start_matches(&SchemaUriPrefix::AkaDotMs.to_string())
        .trim_start_matches(&SchemaUriPrefix::Github.to_string())
        .trim_start_matches("/");
    // The version segment is the first segment of the relative URI
    schema_folder_relative_id
        .split("/")
        .collect::<Vec<&str>>()
        .first()
        .map(std::string::ToString::to_string)
}

/// Updates the version of bundled schema resources to match the root schema version.
/// 
/// This transformer:
/// 
/// 1. Parses the `$id` of the root schema to find the current version. 
/// 1. Iterates over every bundled schema resource.
/// 1. If the bundled schema resource is for a DSC repo schema, the transformer updates the `$id`
///    of the bundled resource to use the same version as the root schema.
/// 1. After updating the ID for a bundled resource, the transformer updates all references to the
///    bundled schema resource.
pub(crate) fn sync_bundled_resource_id_versions(schema: &mut Schema) {
    // First get the root ID so we can update the bundled dsc repo schema resources.
    let lookup_schema = &schema.clone();
    let Some(schema_version_folder) = get_schema_id_version(lookup_schema) else {
        return;
    };
    let replacement_pattern = regex::Regex::new(r"schemas/v(Next|\d+(\.\d+){0,2})/")
        .expect("the regex is always valid");
    let replacement_value = &format!("schemas/{schema_version_folder}/");

    // Make sure we're working from canonicalized references and definitions:
    schema.canonicalize_refs_and_defs_for_bundled_resources();

    // Iterate over bundled schema resources, skipping bundled resources from outside of the
    // repository. Replace the existing version segment with the canonical one for the `$id`.
    for resource_id in lookup_schema.get_bundled_schema_resource_ids(true) {
        let is_dsc_repo_schema =
            resource_id.starts_with(&SchemaUriPrefix::Github.to_string()) ||
            resource_id.starts_with(&SchemaUriPrefix::AkaDotMs.to_string());
        if !is_dsc_repo_schema {
            continue;
        }

        let new_id = replacement_pattern.replace(
            resource_id,
            replacement_value
        );
        // Munge the `$id` keyword in the definition subschema with the correct version folder.
        let definition = schema.get_defs_subschema_from_id_mut(resource_id)
            .expect("a discovered resource ID should exist in `$defs`");
        definition.set_id(&new_id);
        schema.rename_defs_subschema_for_reference(&new_id, &new_id);
        // Replace all references to the old ID with the new ID.
        schema.replace_references(resource_id, &new_id);
    }

    // Re-canonicalize the definition keys and references now that the IDs are updated.
    schema.canonicalize_refs_and_defs_for_bundled_resources();
}
