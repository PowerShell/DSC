// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use schemars::JsonSchema;
use std::{fmt::Display, path::Path};
use tracing::{debug, info, trace};

use crate::{discovery::command_discovery::{load_manifest, ImportedManifest}, dscerror::DscError, dscresources::{command_resource::{invoke_command, process_args}, dscresource::DscResource}, extensions::secret::SecretResult};

use super::{discover::DiscoverResult, extension_manifest::ExtensionManifest, secret::SecretArgKind};

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
    /// The extension aids in retrieving secrets.
    Secret,
}

impl Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::Discover => write!(f, "Discover"),
            Capability::Secret => write!(f, "Secret"),
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
                    trace!("{}", t!("extensions.dscextension.extensionReturned", extension = self.type_name, line = line));
                    let discover_result: DiscoverResult = match serde_json::from_str(line) {
                        Ok(result) => result,
                        Err(err) => {
                            return Err(DscError::Json(err));
                        }
                    };
                    if !Path::new(&discover_result.manifest_path).is_absolute() {
                        return Err(DscError::Extension(t!("extensions.dscextension.discoverNotAbsolutePath", extension = self.type_name.clone(), path = discover_result.manifest_path.clone()).to_string()));
                    }
                    let manifest_path = Path::new(&discover_result.manifest_path);
                    // Currently we don't support extensions discovering other extensions
                    if let ImportedManifest::Resource(resource) = load_manifest(manifest_path)? {
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

    /// Retrieve a secret using the extension.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the secret to retrieve.
    /// * `vault` - An optional vault name to use for the secret.
    ///
    /// # Returns
    ///
    /// A result containing the secret as a string or an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the secret retrieval fails or if the extension does not support the secret capability.
    pub fn secret(&self, name: &str, vault: Option<&str>) -> Result<Option<String>, DscError> {
        if self.capabilities.contains(&Capability::Secret) {
            debug!("{}", t!("extensions.dscextension.retrievingSecretFromExtension", name = name, extension = self.type_name));
            let extension = match serde_json::from_value::<ExtensionManifest>(self.manifest.clone()) {
                Ok(manifest) => manifest,
                Err(err) => {
                    return Err(DscError::Manifest(self.type_name.clone(), err));
                }
            };
            let Some(secret) = extension.secret else {
                return Err(DscError::UnsupportedCapability(self.type_name.clone(), Capability::Secret.to_string()));
            };
            let args = process_secret_args(secret.args.as_ref(), name, vault);
            let (_exit_code, stdout, _stderr) = invoke_command(
                &secret.executable,
                args,
                vault,
                Some(self.directory.as_str()),
                None,
                extension.exit_codes.as_ref(),
            )?;
            if stdout.is_empty() {
                info!("{}", t!("extensions.dscextension.secretNoResults", extension = self.type_name));
                Ok(None)
            } else {
                let result: SecretResult = match serde_json::from_str(&stdout) {
                    Ok(value) => value,
                    Err(err) => {
                        return Err(DscError::Extension(t!("extensions.dscextension.secretExtensionReturnedInvalidJson", extension = self.type_name, error = err).to_string()));
                    }
                };
                if result.secure_string.is_some() {
                    debug!("{}", t!("extensions.dscextension.extensionReturnedSecret", extension = self.type_name));
                } else {
                    debug!("{}", t!("extensions.dscextension.extensionReturnedNoSecret", extension = self.type_name));
                }
                Ok(result.secure_string)
            }
        } else {
            Err(DscError::UnsupportedCapability(
                self.type_name.clone(),
                Capability::Secret.to_string()
            ))
        }
    }
}

impl Default for DscExtension {
    fn default() -> Self {
        DscExtension::new()
    }
}

fn process_secret_args(args: Option<&Vec<SecretArgKind>>, name: &str, vault: Option<&str>) -> Option<Vec<String>> {
    let Some(arg_values) = args else {
        debug!("{}", t!("dscresources.commandResource.noArgs"));
        return None;
    };

    let mut processed_args = Vec::<String>::new();
    for arg in arg_values {
        match arg {
            SecretArgKind::String(s) => {
                processed_args.push(s.clone());
            },
            SecretArgKind::Name { name_arg } => {
                processed_args.push(name_arg.to_string());
                processed_args.push(name.to_string());
            },
            SecretArgKind::Vault { vault_arg } => {
                if let Some(value) = vault {
                    processed_args.push(vault_arg.to_string());
                    processed_args.push(value.to_string());
                }
            },
        }
    }

    Some(processed_args)
}
