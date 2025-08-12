// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use path_absolutize::Absolutize;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use schemars::JsonSchema;
use std::{fmt::Display, path::Path};
use tracing::{debug, info, trace};

use crate::{
    discovery::command_discovery::{
        load_manifest, ImportedManifest
    },
    dscerror::DscError,
    dscresources::{
        command_resource::{
            invoke_command,
            process_args
        },
        dscresource::DscResource
    },
    extensions::import::ImportArgKind
};

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
    /// The extensions supported for importing.
    #[serde(rename = "importFileExtensions")]
    pub import_file_extensions: Option<Vec<String>>,
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
            type_name: String::new(),
            version: String::new(),
            capabilities: Vec::new(),
            import_file_extensions: None,
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

    /// Import a file based on the extension.
    ///
    /// # Arguments
    ///
    /// * `file` - The file to import.
    ///
    /// # Returns
    ///
    /// A result containing the imported file content or an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the import fails or if the extension does not support the import capability.
    pub fn import(&self, file: &str) -> Result<String, DscError> {
        if self.capabilities.contains(&Capability::Import) {
            let file_path = Path::new(file);
            let file_extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or_default().to_string();
            if self.import_file_extensions.as_ref().is_some_and(|exts| exts.contains(&file_extension)) {
                debug!("{}", t!("extensions.dscextension.importingFile", file = file, extension = self.type_name));
            } else {
                debug!("{}", t!("extensions.dscextension.importNotSupported", file = file, extension = self.type_name));
                return Err(DscError::NotSupported(
                    t!("extensions.dscextension.importNotSupported", file = file, extension = self.type_name).to_string(),
                ));
            }

            let extension = match serde_json::from_value::<ExtensionManifest>(self.manifest.clone()) {
                Ok(manifest) => manifest,
                Err(err) => {
                    return Err(DscError::Manifest(self.type_name.clone(), err));
                }
            };
            let Some(import) = extension.import else {
                return Err(DscError::UnsupportedCapability(self.type_name.clone(), Capability::Import.to_string()));
            };
            let args = process_import_args(import.args.as_ref(), file)?;
            let (_exit_code, stdout, _stderr) = invoke_command(
                &import.executable,
                args,
                None,
                Some(self.directory.as_str()),
                None,
                extension.exit_codes.as_ref(),
            )?;
            if stdout.is_empty() {
                info!("{}", t!("extensions.dscextension.importNoResults", extension = self.type_name));
            } else {
                return Ok(stdout);
            }
        }
        Err(DscError::UnsupportedCapability(
            self.type_name.clone(),
            Capability::Import.to_string()
        ))
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
                // see if multiple lines were returned
                let secret = if stdout.lines().count() > 1 {
                    return Err(DscError::NotSupported(t!("extensions.dscextension.secretMultipleLinesReturned", extension = self.type_name).to_string()));
                } else {
                    debug!("{}", t!("extensions.dscextension.extensionReturnedSecret", extension = self.type_name));
                    // remove any trailing newline characters
                    stdout.trim_end_matches('\n').to_string()
                };
                Ok(Some(secret))
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

fn process_import_args(args: Option<&Vec<ImportArgKind>>, file: &str) -> Result<Option<Vec<String>>, DscError> {
    let Some(arg_values) = args else {
        debug!("{}", t!("dscresources.commandResource.noArgs"));
        return Ok(None);
    };

    // make path absolute
    let path = Path::new(file);
    let Ok(full_path) = path.absolutize() else {
        return Err(DscError::Extension(t!("util.failedToAbsolutizePath", path = path : {:?}).to_string()));
    };

    let mut processed_args = Vec::<String>::new();
    for arg in arg_values {
        match arg {
            ImportArgKind::String(s) => {
                processed_args.push(s.clone());
            },
            ImportArgKind::File { file_arg } => {
                if !file_arg.is_empty() {
                    processed_args.push(file_arg.to_string());
                }
                processed_args.push(full_path.to_string_lossy().to_string());
            },
        }
    }

    Ok(Some(processed_args))
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
