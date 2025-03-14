// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::dscresources::resource_manifest::ArgKind;
use rust_i18n::t;
use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::{dscerror::DscError, schemas::DscRepoSchema};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExtensionManifest {
    /// The version of the resource manifest schema.
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "ExtensionManifest::recognized_schema_uris_subschema")]
    pub schema_version: String,
    /// The namespaced name of the extension.
    #[serde(rename = "type")]
    pub r#type: String,
    /// The version of the resource using semantic versioning.
    pub version: String,
    /// The description of the resource.
    pub description: Option<String>,
    /// Tags for the resource.
    pub tags: Option<Vec<String>>,
    /// Details how to call the Discover method of the resource.
    pub discover: Option<DiscoverMethod>,
    /// Mapping of exit codes to descriptions.  Zero is always success and non-zero is always failure.
    #[serde(rename = "exitCodes", skip_serializing_if = "Option::is_none")]
    pub exit_codes: Option<HashMap<i32, String>>,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct DiscoverMethod {
    /// The command to run to get the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Get.
    pub args: Option<Vec<ArgKind>>,
}

impl DscRepoSchema for ExtensionManifest {
    const SCHEMA_FILE_BASE_NAME: &'static str = "manifest";
    const SCHEMA_FOLDER_PATH: &'static str = "extension";
    const SCHEMA_SHOULD_BUNDLE: bool = true;

    fn schema_metadata() -> schemars::schema::Metadata {
        schemars::schema::Metadata {
            title: Some(t!("extensions.extension_manifest.extensionManifestSchemaTitle").into()),
            description: Some(t!("extensions.extension_manifest.extensioneManifestSchemaDescription").into()),
            ..Default::default()
        }
    }

    fn validate_schema_uri(&self) -> Result<(), DscError> {
        if Self::is_recognized_schema_uri(&self.schema_version) {
            Ok(())
        } else {
            Err(DscError::UnrecognizedSchemaUri(
                self.schema_version.clone(),
                Self::recognized_schema_uris(),
            ))
        }
    }
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
    use crate::{
        dscerror::DscError,
        dscresources::resource_manifest::ResourceManifest,
        schemas::DscRepoSchema
    };

    #[test]
    fn test_validate_schema_uri_with_invalid_uri() {
        let invalid_uri = "https://invalid.schema.uri".to_string();

        let manifest = ResourceManifest{
            schema_version: invalid_uri.clone(),
            resource_type: "Microsoft.Dsc.Test/InvalidSchemaUri".to_string(),
            version: "0.1.0".to_string(),
            ..Default::default()
        };

        let ref result = manifest.validate_schema_uri();

        assert!(result.as_ref().is_err());

        match result.as_ref().unwrap_err() {
            DscError::UnrecognizedSchemaUri(actual, recognized) => {
                assert_eq!(actual, &invalid_uri);
                assert_eq!(recognized, &ResourceManifest::recognized_schema_uris())
            },
            _ => {
                panic!("Expected validate_schema_uri() to error on unrecognized schema uri, but was {:?}", result.as_ref().unwrap_err())
            }
        }
    }

    #[test]
    fn test_validate_schema_uri_with_valid_uri() {
        let manifest = ResourceManifest{
            schema_version: ResourceManifest::default_schema_id_uri(),
            resource_type: "Microsoft.Dsc.Test/ValidSchemaUri".to_string(),
            version: "0.1.0".to_string(),
            ..Default::default()
        };

        let result = manifest.validate_schema_uri();

        assert!(result.is_ok());
    }
}
