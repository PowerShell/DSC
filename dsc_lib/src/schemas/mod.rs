// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Contains helpers for JSON schemas and DSC

use schemars::{schema::{Metadata, Schema}, JsonSchema};

use crate::dscerror::DscError;

/// Defines the URI prefix for the hosted schemas.
/// 
/// While the schemas are currently hosted in the GitHub repository, DSC provides the shortened
/// `aka.ms` link for convenience. Using this enum simplifies migrating to a new URI for schemas
/// in the future.
#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]
pub enum SchemaUriPrefix {
    #[default]
    AkaDotMs,
    Github,
}

impl std::fmt::Display for SchemaUriPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AkaDotMs => write!(f, "https://aka.ms/dsc/schemas"),
            Self::Github => write!(f, "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas"),
        }
    }
}

impl SchemaUriPrefix {
    /// Returns every known URI prefix for convenient iteration.
    #[must_use]
    pub fn all() -> Vec<SchemaUriPrefix> {
        vec![
            Self::AkaDotMs,
            Self::Github,
        ]
    }
}

/// Defines the different forms of JSON Schema that DSC publishes.
#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]
pub enum SchemaForm {
    /// Indicates that the schema is bundled using the 2020-12 schema bundling contract.
    /// 
    /// These schemas include all of their references in the `$defs` keyword where the key for
    /// each reference is the `$id` of that subschema and the value is the subschema.
    /// 
    /// The bundled schemas are preferred for offline usage or where network latency is a concern.
    #[default]
    Bundled,
    /// Indicates that the schema is enhanced for interactively viewing, authoring, and editing
    /// the data in VS Code.
    /// 
    /// These schemas include keywords not recognized by JSON Schema libraries and clients outside
    /// of VS Code, like `markdownDescription` and `defaultSnippets`. The schema references and
    /// definitions do not follow the canonical bundling for schema 2020-12, as the VS Code
    /// JSON language server doesn't correctly resolve canonically bundled schemas.
    VSCode,
    /// Indicates that the schema is canonical but not bundled. It may contain references to other
    /// JSON Schemas that require resolution by retrieving those schemas over the network. All
    /// DSC schemas are published in this form for easier review, reuse, and retrieval.
    Canonical,
}

impl SchemaForm {
    /// Returns the file extension for a given form of schema.
    /// 
    /// The extension for [`Bundled`] and [`Canonical`] schemas is `.json`
    /// 
    /// The extension for [`VSCode`] schemas is `.vscode.json`
    #[must_use]
    pub fn to_extension(&self) -> String {
        match self {
            Self::Bundled | Self::Canonical => ".json".to_string(),
            Self::VSCode => ".vscode.json".to_string(),
        }
    }

    /// Return the prefix for a schema's folder path.
    /// 
    /// The [`Bundled`] and [`VSCode`] schemas are always published in the `bundled` folder
    /// immediately beneath the version folder. The [`Canonical`] schemas use the folder path
    /// as defined for that schema.
    #[must_use]
    pub fn to_folder_prefix(&self) -> String {
        match self {
            Self::Bundled | Self::VSCode  => "bundled/".to_string(),
            Self::Canonical => String::new(),
        }
    }

    /// Returns every schema form for convenient iteration.
    #[must_use]
    pub fn all() -> Vec<SchemaForm> {
        vec![
            Self::Bundled,
            Self::VSCode,
            Self::Canonical,
        ]
    }
}

/// Defines the versions of DSC recognized for schema validation and handling.
/// 
/// The DSC schemas are published into three folders:
/// 
/// - `v<major>.<minor>.<patch>` always includes the exact JSON Schema that shipped in that release
///   of DSC.
/// - `v<major>.<minor>` always includes the latest JSON Schema compatible with that minor version
///   of DSC.
/// - `v<major>` always includes the latest JSON Schema compatible with that major version of DSC.
/// 
/// Pinning to `v<major>` requires the least-frequent updating of the `$schema` in configuration
/// documents and resource manifests, but also introduces changes that affect those schemas
/// (without breaking changes) regularly. Some of the added features may not be effective in the
/// version of DSC a user has installed.
/// 
/// Pinning to `v<major>.<minor>` ensures that users always have the latest schemas for the version
/// of DSC they're using without schema changes that they may not be able to take advantage of.
/// However, it requires updating the resource manifests and configuration documents with each
/// minor release of DSC.
/// 
/// Pinning to `v<major>.<minor>.<patch>` is the most specific option, but requires the most
/// frequent updating on the part of resource and configuration authors.
#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]
pub enum RecognizedSchemaVersion {
    // Before any relase is published, this enum must be updated with the new version variants.
    // Every release requires a patch version, like `V3_0_1` or `v3_1_0`. New minor releases also
    // require a new minor version, like `v3_1`.

    /// Represents `v3` schema folder.
    #[default]
    V3,
    /// Represents the `v3.0` schema folder.
    V3_0,
    /// Represents the `v3.0.0` schema folder.
    V3_0_0,
}

impl std::fmt::Display for RecognizedSchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V3 => write!(f, "v3"),
            Self::V3_0 => write!(f, "v3.0"),
            Self::V3_0_0 => write!(f, "v3.0.0"),
        }
    }
}

impl RecognizedSchemaVersion {
    /// Returns every recognized schema version for convenient iteration.
    #[must_use]
    pub fn all() -> Vec<RecognizedSchemaVersion> {
        vec![
            Self::V3,
            Self::V3_0,
            Self::V3_0_0,
        ]
    }

    //// Returns the latest version with major, minor, and patch segments, like `3.0.0`.
    #[must_use]
    pub fn latest() -> RecognizedSchemaVersion {
        Self::V3_0_0
    }

    /// Returns the latest minor version for the latest major version, like `3.0`.
    #[must_use]
    pub fn latest_minor() -> RecognizedSchemaVersion {
        Self::V3_0
    }

    /// Returns the latest major version, like `3`
    #[must_use]
    pub fn latest_major() -> RecognizedSchemaVersion {
        Self::V3
    }
}

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
///- `schema_file_base_name` - specify the base name for the schema file, like `document` for
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
    metadata: Metadata,
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

    schemars::schema::SchemaObject {
        instance_type: Some(schemars::schema::InstanceType::String.into()),
        format: Some("uri".to_string()),
        string: Some(Box::new(schemars::schema::StringValidation {
            max_length: None,
            min_length: None,
            pattern: None,
        })),
        enum_values: Some(enums),
        metadata: Some(Box::new(metadata)),
        ..Default::default()
    }.into()
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
///- `schema_file_base_name` - specify the base name for the schema file, like `document` for
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
fn get_default_schema_form(should_bundle: bool) -> SchemaForm {
    if should_bundle {
        SchemaForm::Bundled
    } else {
        SchemaForm::Canonical
    }
}

/// Defines a reusable trait to simplify managing multiple versions of JSON Schemas for DSC
/// structs and enums.
/// 
/// This trait is only intended for use by definitions in the DSC repository.
pub trait DscRepoSchema : JsonSchema {
    /// Defines the base name for the exported JSON Schema, like `document` for
    /// [`Configuration`].
    /// 
    /// [`Configuration`]: crate::configure::config_doc::Configuration
    const SCHEMA_FILE_BASE_NAME: &'static str;

    /// Defines the folder path for the schema relative to the published version folder, like
    /// `config` for [`Configuration`].
    /// 
    /// [`Configuration`]: crate::configure::config_doc::Configuration
    const SCHEMA_FOLDER_PATH: &'static str;

    /// Indicates whether the schema should be published in its bundled form. All bundled schemas
    /// are also published with their VS Code form. Schemas that aren't bundled aren't published
    /// with the VS Code form.
    const SCHEMA_SHOULD_BUNDLE: bool;
    fn schema_metadata() -> Metadata;

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
    fn recognized_schema_uris_subschema(_: &mut schemars::gen::SchemaGenerator) -> Schema {
        get_recognized_uris_subschema(
            Self::schema_metadata(),
            Self::SCHEMA_FILE_BASE_NAME,
            Self::SCHEMA_FOLDER_PATH,
            Self::SCHEMA_SHOULD_BUNDLE
        )
    }

    /// Indicates whether a given string is a recognized shema URI.
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
    /// [`DscError::UnrecognizedSchemaUri`] error.
    fn validate_schema_uri(&self) -> Result<(), DscError> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[test]
    fn test_get_recognized_schema_uri() {
        let expected = "https://aka.ms/dsc/schemas/v3/bundled/config/document.json".to_string();
        let actual = get_recognized_schema_uri(
            "document",
            "config",
            RecognizedSchemaVersion::V3,
            SchemaForm::Bundled,
            SchemaUriPrefix::AkaDotMs
        );
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_get_recognized_schema_uris() {
        let expected: Vec<String> = vec![
            "https://aka.ms/dsc/schemas/v3/bundled/config/document.json".to_string(),
            "https://aka.ms/dsc/schemas/v3.0/bundled/config/document.json".to_string(),
            "https://aka.ms/dsc/schemas/v3.0.0/bundled/config/document.json".to_string(),
            "https://aka.ms/dsc/schemas/v3/bundled/config/document.vscode.json".to_string(),
            "https://aka.ms/dsc/schemas/v3.0/bundled/config/document.vscode.json".to_string(),
            "https://aka.ms/dsc/schemas/v3.0.0/bundled/config/document.vscode.json".to_string(),
            "https://aka.ms/dsc/schemas/v3/config/document.json".to_string(),
            "https://aka.ms/dsc/schemas/v3.0/config/document.json".to_string(),
            "https://aka.ms/dsc/schemas/v3.0.0/config/document.json".to_string(),
            "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/config/document.json".to_string(),
            "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/config/document.json".to_string(),
            "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/config/document.json".to_string(),
            "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/config/document.vscode.json".to_string(),
            "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/config/document.vscode.json".to_string(),
            "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/config/document.vscode.json".to_string(),
            "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/config/document.json".to_string(),
            "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/config/document.json".to_string(),
            "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/config/document.json".to_string(),
        ];

        let actual = get_recognized_schema_uris(
            "document",
            "config",
            true
        );

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_get_default_schema_uri() {
        let expected_bundled = "https://aka.ms/dsc/schemas/v3/bundled/config/document.json".to_string();
        let expected_canonical = "https://aka.ms/dsc/schemas/v3/config/document.json".to_string();

        let schema_file_base_name = "document";
        let schema_folder_path = "config";
        
        assert_eq!(expected_bundled, get_default_schema_uri(schema_file_base_name, schema_folder_path, true));
        assert_eq!(expected_canonical, get_default_schema_uri(schema_file_base_name, schema_folder_path, false))
    }

    #[test]
    fn test_dsc_repo_schema_bundled() {
        #[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
        struct ExampleBundledSchema {
            pub schema_version: String,
        }

        impl DscRepoSchema for ExampleBundledSchema {
            const SCHEMA_FILE_BASE_NAME: &'static str = "schema";
            const SCHEMA_FOLDER_PATH: &'static str = "example";
            const SCHEMA_SHOULD_BUNDLE: bool = true;

            fn schema_metadata() -> Metadata {
                Metadata::default()
            }
        }

        let bundled_uri = "https://aka.ms/dsc/schemas/v3/bundled/example/schema.json".to_string();
        let vscode_uri = "https://aka.ms/dsc/schemas/v3/bundled/example/schema.vscode.json".to_string();
        let canonical_uri = "https://aka.ms/dsc/schemas/v3/example/schema.json".to_string();
        let schema_version = RecognizedSchemaVersion::V3;

        assert_eq!(
            bundled_uri,
            ExampleBundledSchema::default_schema_id_uri()
        );

        assert_eq!(
            Some(bundled_uri),
            ExampleBundledSchema::get_bundled_schema_id_uri(schema_version)
        );

        assert_eq!(
            Some(vscode_uri),
            ExampleBundledSchema::get_enhanced_schema_id_uri(schema_version)
        );

        assert_eq!(
            canonical_uri,
            ExampleBundledSchema::get_canonical_schema_id_uri(schema_version)
        )
    }

    #[test]
    fn test_dsc_repo_schema_not_bundled() {
        #[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
        struct ExampleNotBundledSchema {
            pub schema_version: String,
        }

        impl DscRepoSchema for ExampleNotBundledSchema {
            const SCHEMA_FILE_BASE_NAME: &'static str = "schema";
            const SCHEMA_FOLDER_PATH: &'static str = "example";
            const SCHEMA_SHOULD_BUNDLE: bool = false;

            fn schema_metadata() -> Metadata {
                Metadata::default()
            }
        }

        let canonical_uri = "https://aka.ms/dsc/schemas/v3/example/schema.json".to_string();
        let schema_version = RecognizedSchemaVersion::V3;
        assert_eq!(
            canonical_uri,
            ExampleNotBundledSchema::default_schema_id_uri()
        );

        assert_eq!(
            None,
            ExampleNotBundledSchema::get_bundled_schema_id_uri(schema_version)
        );

        assert_eq!(
            None,
            ExampleNotBundledSchema::get_enhanced_schema_id_uri(schema_version)
        );

        assert_eq!(
            canonical_uri,
            ExampleNotBundledSchema::get_canonical_schema_id_uri(schema_version)
        )
    }
}
