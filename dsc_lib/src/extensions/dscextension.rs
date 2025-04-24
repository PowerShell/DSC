// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use schemars::JsonSchema;
use std::{fmt::Display, path::Path};
use tracing::info;

use crate::{discovery::command_discovery::{load_manifest, ManifestResource}, dscerror::DscError, dscresources::{command_resource::{invoke_command, process_args}, dscresource::DscResource}};

use super::{discover::DiscoverResult, extension_manifest::ExtensionManifest};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DscExtension {
    /// The namespaced name of the resource.
    #[serde(rename="type")]
    pub type_name: String,
    /// The version of the resource.
    pub version: String,
    /// The capabilities of the resource.
    pub capabilities: Vec<Capability>,
    /// The file path to the resource.
    pub path: String,
    /// The description of the resource.
    pub description: Option<String>,
    // The directory path to the resource.
    pub directory: String,
    /// The author of the resource.
    pub author: Option<String>,
    /// The manifest of the resource.
    pub manifest: Value,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Capability {
    /// The extension aids in discovering resources.
    Discover,
}

impl Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::Discover => write!(f, "Discover"),
        }
    }
}

impl DscExtension {
    #[must_use]
    pub fn new() -> Self {
        Self {
            type_name: String::new(),
            version: String::new(),
            capabilities: Vec::new(),
            description: None,
            path: String::new(),
            directory: String::new(),
            author: None,
            manifest: Value::Null,
        }
    }

    /// Perform discovery of resources using the extension.
    ///
    /// # Returns
    ///
    /// A result containing a vector of discovered resources or an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the discovery fails.
    pub fn discover(&self) -> Result<Vec<DscResource>, DscError> {
        let mut resources: Vec<DscResource> = Vec::new();

        if self.capabilities.contains(&Capability::Discover) {
            let extension = match serde_json::from_value::<ExtensionManifest>(self.manifest.clone()) {
                Ok(manifest) => manifest,
                Err(err) => {
                    return Err(DscError::Manifest(self.type_name.clone(), err));
                }
            };
            let Some(discover) = extension.discover else {
                return Err(DscError::UnsupportedCapability(self.type_name.clone(), Capability::Discover.to_string()));
            };
            let args = process_args(discover.args.as_ref(), "");
            let (_exit_code, stdout, _stderr) = invoke_command(
                &discover.executable,
                args,
                None,
                Some(self.directory.as_str()),
                None,
                extension.exit_codes.as_ref(),
            )?;
            if stdout.is_empty() {
                info!("{}", t!("extensions.dscextension.discoverNoResults", extension = self.type_name));
            } else {
                for line in stdout.lines() {
                    let discover_result: DiscoverResult = match serde_json::from_str(line) {
                        Ok(result) => result,
                        Err(err) => {
                            return Err(DscError::Json(err));
                        }
                    };
                    let manifest_path = Path::new(&discover_result.resource_manifest_path);
                    // Currently we don't support extensions discovering other extensions
                    if let ManifestResource::Resource(resource) = load_manifest(manifest_path)? {
                        resources.push(resource);
                    }
                }
            }

            Ok(resources)
        } else {
            Err(DscError::UnsupportedCapability(
                self.type_name.clone(),
                Capability::Discover.to_string()
            ))
        }
    }
}

impl Default for DscExtension {
    fn default() -> Self {
        DscExtension::new()
    }
}
