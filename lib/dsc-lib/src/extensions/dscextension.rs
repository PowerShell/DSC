// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::extensions::import::ImportMethod;
use crate::schemas::{dsc_repo::DscRepoSchema, transforms::idiomaticize_string_enum};
use crate::types::FullyQualifiedTypeName;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use schemars::JsonSchema;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(base_name = "list", folder_path = "outputs/extension")]
pub struct DscExtension {
    /// The namespaced name of the extension.
    #[serde(rename="type")]
    pub type_name: FullyQualifiedTypeName,
    /// The version of the extension.
    pub version: String,
    /// The capabilities of the extension.
    pub capabilities: Vec<Capability>,
    /// The import specifics.
    pub import: Option<ImportMethod>,
    /// The file path to the extension.
    pub path: PathBuf,
    /// An optional message indicating the extension is deprecated.  If provided, the message will be shown when the extension is used.
    pub deprecation_message: Option<String>,
    /// The description of the extension.
    pub description: Option<String>,
    // The directory path to the extension.
    pub directory: PathBuf,
    /// The author of the extension.
    pub author: Option<String>,
    /// The manifest of the extension.
    pub manifest: Value,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(transform = idiomaticize_string_enum)]
#[dsc_repo_schema(base_name = "extensionCapabilities", folder_path = "definitions")]
pub enum Capability {
    /// The extension aids in discovering resources.
    Discover,
    /// The extension aids in retrieving secrets.
    Secret,
    /// The extension imports configuration from a different format.
    Import,
}

impl Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::Discover => write!(f, "Discover"),
            Capability::Secret => write!(f, "Secret"),
            Capability::Import => write!(f, "Import"),
        }
    }
}

impl DscExtension {
    #[must_use]
    pub fn new() -> Self {
        Self {
            type_name: FullyQualifiedTypeName::default(),
            version: String::new(),
            capabilities: Vec::new(),
            import: None,
            deprecation_message: None,
            description: None,
            path: PathBuf::new(),
            directory: PathBuf::new(),
            author: None,
            manifest: Value::Null,
        }
    }
}

impl Default for DscExtension {
    fn default() -> Self {
        DscExtension::new()
    }
}
