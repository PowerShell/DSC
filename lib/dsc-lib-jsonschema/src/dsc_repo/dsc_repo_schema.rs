// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use schemars::{JsonSchema, Schema};
use thiserror::Error;

use crate::dsc_repo::{
    RecognizedSchemaVersion,
    SchemaForm,
    SchemaUriPrefix,
    get_default_schema_uri,
    get_recognized_schema_uri,
    get_recognized_schema_uris,
    get_recognized_uris_subschema
};

/// Defines a reusable trait to simplify managing multiple versions of JSON Schemas for DSC
/// structs and enums.
///
/// This trait is only intended for use by definitions in the DSC repository.
pub trait DscRepoSchema : JsonSchema {
    /// Defines the base name for the exported JSON Schema.
    ///
    /// For example, for the following `$id`, `document` is the base name:
    ///
    /// ```json
    /// { "$id": "https://aka.ms/dsc/schemas/v3/config/document.json" }
    /// ```
    const SCHEMA_FILE_BASE_NAME: &'static str;

    /// Defines the folder path for the schema relative to the published version folder.
    ///
    /// For example, for the following `$id`, `config` is the folder path:
    ///
    /// ```json
    /// { "$id": "https://aka.ms/dsc/schemas/v3/config/document.json" }
    /// ```
    const SCHEMA_FOLDER_PATH: &'static str;

    /// Indicates whether the schema should be published in its bundled form.
    ///
    /// All bundled schemas are also published with their VS Code form. Schemas that aren't bundled
    /// aren't published with the VS Code form.
    const SCHEMA_SHOULD_BUNDLE: bool;

    /// Defines the metadata for the `$schema` property of a struct that takes multiple schema
    /// versions.
    ///
    /// This simplifies providing metadata annotation keywords, since we generate the subschema for
    /// this property with the [`recognized_schema_uris_subschema`] method.
    ///
    /// [`recognized_schema_uris_subschema`]: DscRepoSchema::recognized_schema_uris_subschema
    fn schema_property_metadata() -> Schema;

    /// Returns the default URI for the schema.
    ///
    /// An object representing an instance of the schema can specify any valid URI, but the
    /// default when creating an instance is the latest major version of the schema with the
    /// `aka.ms` prefix. If the schema is published in the bundled form, the default is for the
    /// bundled schema. Otherwise, the default is for the canonical (non-bundled) schema.
    #[must_use]
    fn default_schema_id_uri() -> String {
        get_default_schema_uri(
            Self::SCHEMA_FILE_BASE_NAME,
            Self::SCHEMA_FOLDER_PATH,
            Self::SCHEMA_SHOULD_BUNDLE
        )
    }

    /// Returns the schema URI for a given version, form, and prefix.
    #[must_use]
    fn get_schema_id_uri(
        schema_version: RecognizedSchemaVersion,
        schema_form: SchemaForm,
        uri_prefix: SchemaUriPrefix
    ) -> String {
        get_recognized_schema_uri(
            Self::SCHEMA_FILE_BASE_NAME,
            Self::SCHEMA_FOLDER_PATH,
            schema_version,
            schema_form,
            uri_prefix
        )
    }

    /// Returns the URI for the VS Code form of the schema with the default prefix for a given
    /// version.
    ///
    /// If the type isn't published in bundled form, this function returns `None`.
    #[must_use]
    fn get_enhanced_schema_id_uri(schema_version: RecognizedSchemaVersion) -> Option<String> {
        if !Self::SCHEMA_SHOULD_BUNDLE {
            return None;
        }

        Some(get_recognized_schema_uri(
            Self::SCHEMA_FILE_BASE_NAME,
            Self::SCHEMA_FOLDER_PATH,
            schema_version,
            SchemaForm::VSCode,
            SchemaUriPrefix::default()
        ))
    }

    /// Returns the URI for the canonical (non-bundled) form of the schema with the default
    /// prefix for a given version.
    #[must_use]
    fn get_canonical_schema_id_uri(schema_version: RecognizedSchemaVersion) -> String {
        get_recognized_schema_uri(
            Self::SCHEMA_FILE_BASE_NAME,
            Self::SCHEMA_FOLDER_PATH,
            schema_version,
            SchemaForm::Canonical,
            SchemaUriPrefix::default()
        )
    }

    /// Returns the URI for the bundled form of the schema with the default prefix for a given
    /// version.
    #[must_use]
    fn get_bundled_schema_id_uri(schema_version: RecognizedSchemaVersion) -> Option<String> {
        if !Self::SCHEMA_SHOULD_BUNDLE {
            return None;
        }

        Some(get_recognized_schema_uri(
            Self::SCHEMA_FILE_BASE_NAME,
            Self::SCHEMA_FOLDER_PATH,
            schema_version,
            SchemaForm::Bundled,
            SchemaUriPrefix::default()
        ))
    }

    /// Returns the list of recognized schema URIs for the struct or enum.
    ///
    /// This convenience function generates a vector containing every recognized JSON Schema `$id`
    /// URI for a specific schema. It handles returning the schemas for every recognized prefix,
    /// version, and form.
    #[must_use]
    fn recognized_schema_uris() -> Vec<String> {
        get_recognized_schema_uris(
            Self::SCHEMA_FILE_BASE_NAME,
            Self::SCHEMA_FOLDER_PATH,
            Self::SCHEMA_SHOULD_BUNDLE
        )
    }

    /// Returns the subschema to validate a `$schema` keyword pointing to the type.
    ///
    /// Every schema has a canonical `$id`, but DSC needs to maintain compatibility with schemas
    /// within a major version and ensure that previous schema versions can be correctly
    /// recognized and validated. This method generates the appropriate subschema with every
    /// valid URI for the schema's `$id` without needing to regularly update an enum for each
    /// schema and release.
    #[must_use]
    fn recognized_schema_uris_subschema(_: &mut schemars::SchemaGenerator) -> Schema {
        get_recognized_uris_subschema(
            &Self::schema_property_metadata(),
            Self::SCHEMA_FILE_BASE_NAME,
            Self::SCHEMA_FOLDER_PATH,
            Self::SCHEMA_SHOULD_BUNDLE
        )
    }

    /// Indicates whether a given string is a recognized schema URI.
    #[must_use]
    fn is_recognized_schema_uri(uri: &String) -> bool {
        Self::recognized_schema_uris().contains(uri)
    }

    /// Validates the `$schema` keyword for deserializing instances.
    ///
    /// This method simplifies the validation of a type that has the `$schema` keyword and expects
    /// that instances of the type in data indicate which schema version DSC should use to validate
    /// them.
    ///
    /// This method includes a default implementation to avoid requiring the implementation for
    /// types that don't define the `$schema` keyword in their serialized form.
    ///
    /// Any DSC type that serializes with the `$schema` keyword **must** define this
    /// method to actually validate the instance.
    ///
    /// # Errors
    ///
    /// If the value for the schema field isn't a recognized schema, the method should raise the
    /// [`UnrecognizedSchemaUri`] error.
    fn validate_schema_uri(&self) -> Result<(), UnrecognizedSchemaUri> {
        Ok(())
    }
}

/// Defines the error when a user-defined JSON Schema references an unrecognized schema URI.
#[derive(Error, Debug)]
#[error(
    "{t}: {0}. {t2}: {1:?}",
    t = t!("dsc_repo.dsc_repo_schema.unrecognizedSchemaUri"),
    t2 = t!("dsc_repo.dsc_repo_schema.validSchemaUrisAre")
)]
pub struct UnrecognizedSchemaUri(pub String, pub Vec<String>);
