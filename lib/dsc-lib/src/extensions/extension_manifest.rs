// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

use crate::dscerror::DscError;
use crate::extensions::{discover::DiscoverMethod, import::ImportMethod, secret::SecretMethod};
use crate::schemas::dsc_repo::DscRepoSchema;
use crate::types::{FullyQualifiedTypeName, TagList};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(
    base_name = "manifest",
    folder_path = "extension",
    should_bundle = true,
    schema_field(
        name = schema_version,
        title = t!("extensions.extension_manifest.extensionManifestSchemaTitle"),
        description = t!("extensions.extension_manifest.extensionManifestSchemaDescription"),
    )
)]
pub struct ExtensionManifest {
    /// The version of the extension manifest schema.
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "ExtensionManifest::recognized_schema_uris_subschema")]
    pub schema_version: String,
    /// The namespaced name of the extension.
    #[serde(rename = "type")]
    pub r#type: FullyQualifiedTypeName,
    /// The version of the extension using semantic versioning.
    pub version: String,
    /// An optional condition for the extension to be active.  If the condition evaluates to false, the extension is skipped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// The description of the extension.
    pub description: Option<String>,
    /// Tags for the extension.
    #[serde(default, skip_serializing_if = "TagList::is_empty")]
    pub tags: TagList,
    /// Details how to call the Discover method of the extension.
    pub discover: Option<DiscoverMethod>,
    /// Details how to call the Import method of the extension.
    pub import: Option<ImportMethod>,
    /// Details how to call the ImportParameters method of the extension.
    #[serde(rename = "importParameters")]
    pub import_parameters: Option<ImportMethod>,
    /// Details how to call the Secret method of the extension.
    pub secret: Option<SecretMethod>,
    /// Mapping of exit codes to descriptions.  Zero is always success and non-zero is always failure.
    #[serde(rename = "exitCodes", skip_serializing_if = "Option::is_none")]
    pub exit_codes: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Map<String, Value>>,
}

/// Import a resource manifest from a JSON value.
///
/// # Arguments
///
/// * `manifest` - The JSON value to import.
///
/// # Returns
///
/// * `Result<ResourceManifest, DscError>` - The imported resource manifest.
///
/// # Errors
///
/// * `DscError` - The JSON value is invalid or the schema version is not supported.
pub fn import_manifest(manifest: Value) -> Result<ExtensionManifest, DscError> {
    // TODO: enable schema version validation, if not provided, use the latest
    let manifest = serde_json::from_value::<ExtensionManifest>(manifest)?;
    Ok(manifest)
}

/// Validate a semantic version string.
///
/// # Arguments
///
/// * `version` - The semantic version string to validate.
///
/// # Returns
///
/// * `Result<(), Error>` - The result of the validation.
///
/// # Errors
///
/// * `Error` - The version string is not a valid semantic version.
pub fn validate_semver(version: &str) -> Result<(), semver::Error> {
    Version::parse(version)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::schemas::dsc_repo::{DscRepoSchema, UnrecognizedSchemaUri};

    use crate::extensions::extension_manifest::ExtensionManifest;

    #[test]
    fn test_validate_schema_uri_with_invalid_uri() {
        let invalid_uri = "https://invalid.schema.uri".to_string();

        let manifest = ExtensionManifest{
            schema_version: invalid_uri.clone(),
            r#type: "Microsoft.Dsc.Test/InvalidSchemaUri".parse().unwrap(),
            version: "0.1.0".to_string(),
            ..Default::default()
        };

        let ref result = manifest.validate_schema_uri();

        assert!(result.as_ref().is_err());

        match result.as_ref().unwrap_err() {
            UnrecognizedSchemaUri(actual, recognized) => {
                assert_eq!(actual, &invalid_uri);
                assert_eq!(recognized, &ExtensionManifest::recognized_schema_uris())
            },
        }
    }

    #[test]
    fn test_validate_schema_uri_with_valid_uri() {
        let manifest = ExtensionManifest{
            schema_version: ExtensionManifest::default_schema_id_uri(),
            r#type: "Microsoft.Dsc.Test/ValidSchemaUri".parse().unwrap(),
            version: "0.1.0".to_string(),
            ..Default::default()
        };

        let result = manifest.validate_schema_uri();

        assert!(result.is_ok());
    }
}
