// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    discovery::command_discovery::{
        DSC_ADAPTED_RESOURCE_EXTENSIONS, DSC_EXTENSION_EXTENSIONS, DSC_MANIFEST_LIST_EXTENSIONS, DSC_RESOURCE_EXTENSIONS, ImportedManifest, load_manifest, load_manifest_content
    },
    dscerror::DscError,
    dscresources::{
        command_resource::invoke_command,
        dscresource::DscResource
    },
    extensions::{
        dscextension::{
            Capability,
            DscExtension,
        },
        extension_manifest::ExtensionManifest,
    },
    schemas::dsc_repo::DscRepoSchema
};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use tracing::{info, trace, warn};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.discover", folder_path = "extension")]
pub struct DiscoverMethod {
    /// The command to run to get the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Get.
    pub args: Option<Vec<DiscoverArgKind>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ManifestKind {
    /// The path to the resource manifest, must be absolute.
    ManifestPath(PathBuf),
    /// The content of the resource manifest.
    ManifestContent(Value),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "discover", folder_path = "extension/stdout")]
pub struct DiscoverResult {
    #[serde(flatten)]
    pub path_or_content: ManifestKind,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum DiscoverArgKind {
    String(String),
    Extensions {
        /// The argument that accepts the extensions list.  The extensions list will be passed as a comma separated list of extensions.
        #[serde(rename = "extensionsArg")]
        extensions_arg: String,
        /// Whether to include quotes around the extensions list.  If true, the extensions list will be passed as a quoted string.
        #[serde(rename = "includeQuotes", default)]
        include_quotes: bool,
    },
}

impl DscExtension {
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
                    return Err(DscError::Manifest(self.type_name.to_string(), err));
                }
            };
            let Some(discover) = extension.discover else {
                return Err(DscError::UnsupportedCapability(self.type_name.to_string(), Capability::Discover.to_string()));
            };
            let args = process_discover_args(discover.args.as_ref())?;
            if let Some(deprecation_message) = extension.deprecation_message.as_ref() {
                warn!("{}", t!("extensions.dscextension.deprecationMessage", extension = self.type_name, message = deprecation_message));
            }
            let (_exit_code, stdout, _stderr) = invoke_command(
                &discover.executable,
                args,
                None,
                Some(&self.directory),
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
                    match discover_result.path_or_content {
                        ManifestKind::ManifestContent(manifest_value) => {
                            let imported_manifests = load_manifest_content(&manifest_value)?;
                            info!("Manifest imported from extension {}", self.type_name);
                            for imported_manifest in imported_manifests {
                                if let ImportedManifest::Resource(resource) = imported_manifest {
                                    resources.push(resource);
                                }
                            }
                        }
                        ManifestKind::ManifestPath(manifest_path) => {
                            if !manifest_path.is_absolute() {
                                return Err(DscError::Extension(t!("extensions.dscextension.discoverNotAbsolutePath", extension = self.type_name.clone(), path = manifest_path.display()).to_string()));
                            }
                            // Currently we don't support extensions discovering other extensions
                            let manifests = match load_manifest(&manifest_path) {
                                Ok(manifests) => manifests,
                                Err(err) => {
                                    info!("{}", t!("extensions.dscextension.failedLoadManifest", extension = self.type_name, err = err));
                                    continue;
                                }
                            };
                            for imported_manifest in manifests {
                                if let ImportedManifest::Resource(resource) = imported_manifest {
                                    resources.push(resource);
                                }
                            }
                        }
                    }
                }
            }

            Ok(resources)
        } else {
            Err(DscError::UnsupportedCapability(
                self.type_name.to_string(),
                Capability::Discover.to_string()
            ))
        }
    }
}

fn process_discover_args(args: Option<&Vec<DiscoverArgKind>>) -> Result<Option<Vec<String>>, DscError> {
    let Some(arg_values) = args else {
        return Ok(None);
    };

    let mut processed_args = Vec::<String>::new();
    for arg in arg_values {
        match arg {
            DiscoverArgKind::String(s) => {
                processed_args.push(s.clone());
            }
            DiscoverArgKind::Extensions { extensions_arg, include_quotes } => {
                processed_args.push(extensions_arg.clone());
                let mut extensions = Vec::<String>::new();
                extensions.extend(DSC_ADAPTED_RESOURCE_EXTENSIONS.iter().map(|s| s.to_string()));
                extensions.extend(DSC_EXTENSION_EXTENSIONS.iter().map(|s| s.to_string()));
                extensions.extend(DSC_MANIFEST_LIST_EXTENSIONS.iter().map(|s| s.to_string()));
                extensions.extend(DSC_RESOURCE_EXTENSIONS.iter().map(|s| s.to_string()));
                let extensions_arg_value = extensions.join(",");
                if *include_quotes {
                    processed_args.push(format!("\"{}\"", extensions_arg_value));
                } else {
                    processed_args.push(extensions_arg_value);
                }
            }
        }
    }

    Ok(Some(processed_args))
}
