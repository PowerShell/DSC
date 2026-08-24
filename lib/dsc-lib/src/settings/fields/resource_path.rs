// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::path::PathBuf;

use crate::settings::{DscSettingsResolvedField, DscSettingsScope};
use crate::schemas::{dsc_repo::DscRepoSchema, schema_i18n};

use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, DscRepoSchema)]
#[serde(rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "resourcePath", folder_path = "settings/fields")]
pub struct ResourcePathFileData {
    /// Directories that DSC should search for executables and manifests.
    pub directories: Option<Vec<PathBuf>>,
    /// Whether to append the `PATH` environment variable to the list of directories.
    pub append_env_path: Option<bool>,
    /// Whether DSC should allow invoking binaries outside of those listed in [`directories`].
    /// 
    /// [`directories`]: Self::directories
    pub restricted: Option<bool>,
}

impl JsonSchema for ResourcePathFileData {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        Self::default_schema_id_uri().into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "title": schema_i18n!("title"),
            "description": schema_i18n!("description"),
            "markdownDescription": schema_i18n!("markdownDescription"),
            "type": "object",
            "properties": {
                "appendEnvPath": {
                    "type": "boolean",
                    "title": schema_i18n!("appendEnvPath.title"),
                    "description": schema_i18n!("appendEnvPath.description"),
                    "markdownDescription": schema_i18n!("appendEnvPath.markdownDescription")
                },
                "directories": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "title": schema_i18n!("directories.title"),
                    "description": schema_i18n!("directories.description"),
                    "markdownDescription": schema_i18n!("directories.markdownDescription")
                },
                "restrictPath": {
                    "type": "boolean",
                    "title": schema_i18n!("restrictPath.title"),
                    "description": schema_i18n!("restrictPath.description"),
                    "markdownDescription": schema_i18n!("restrictPath.markdownDescription")
                }
            },
            "anyOf": [
                {
                    "if": {
                        "properties": { "restrictPath": { "const": true } }
                    },
                    "then": {
                        "anyOf": [
                            { "not": { "required": ["appendEnvPath"] } },
                            { "properties": { "appendEnvPath": { "const": false } } }
                        ]
                    }
                },
                {
                    "if": {
                        "properties": { "appendEnvPath": { "const": true } }
                    },
                    "then": {
                        "anyOf": [
                            { "not": { "required": ["restrictPath"] } },
                            { "properties": { "restrictPath": { "const": false } } }
                        ]
                    }
                }
            ]
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResourcePathCodeDefaults {
    pub directories: Vec<PathBuf>,
    pub append_env_path: bool,
    pub restricted: bool,
}

impl Default for ResourcePathCodeDefaults {
    fn default() -> Self {
        CODE_DEFAULT_RESOURCE_PATH
    }
}

/// Defines the default values for the resource path configuration in DSC.
/// 
/// The following snippet shows the effective code defaults as YAML data:
/// 
/// ```yaml
/// resource_path:
///   directories: []
///   append_env_path: true
///   restricted: false
/// ```
pub const CODE_DEFAULT_RESOURCE_PATH: ResourcePathCodeDefaults = ResourcePathCodeDefaults {
    directories: vec![],
    append_env_path: true,
    restricted: false,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePathResolvedSettings {
    pub directories: DscSettingsResolvedField<Vec<PathBuf>>,
    pub append_env_path: DscSettingsResolvedField<bool>,
    pub restricted: DscSettingsResolvedField<bool>,
}

impl Default for ResourcePathResolvedSettings {
    fn default() -> Self {
        let scope = DscSettingsScope::Default;
        Self {
            directories: DscSettingsResolvedField::new(CODE_DEFAULT_RESOURCE_PATH.directories.clone(), scope),
            append_env_path: DscSettingsResolvedField::new(CODE_DEFAULT_RESOURCE_PATH.append_env_path, scope),
            restricted: DscSettingsResolvedField::new(CODE_DEFAULT_RESOURCE_PATH.restricted, scope),
        }
    }
}
