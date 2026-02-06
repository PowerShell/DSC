// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    discovery::command_discovery::{
        load_manifest, ImportedManifest
    },
    dscerror::DscError,
    dscresources::{
        command_resource::{
            invoke_command, process_get_args, CommandResourceInfo
        },
        dscresource::DscResource,
        resource_manifest::GetArgKind,
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
use std::path::PathBuf;
use tracing::{info, trace};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.discover", folder_path = "extension")]
pub struct DiscoverMethod {
    /// The command to run to get the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Get.
    pub args: Option<Vec<GetArgKind>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "discover", folder_path = "extension/stdout")]
pub struct DiscoverResult {
    /// The path to the resource manifest, must be absolute.
    #[serde(rename = "manifestPath")]
    pub manifest_path: PathBuf,
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
            let command_resource_info = CommandResourceInfo {
                type_name: self.type_name.clone(),
                path: None,
            };
            let args = process_get_args(discover.args.as_ref(), "", &command_resource_info);
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
                    if !discover_result.manifest_path.is_absolute() {
                        return Err(DscError::Extension(t!("extensions.dscextension.discoverNotAbsolutePath", extension = self.type_name.clone(), path = discover_result.manifest_path.display()).to_string()));
                    }
                    // Currently we don't support extensions discovering other extensions
                    for imported_manifest in load_manifest(&discover_result.manifest_path)? {
                        if let ImportedManifest::Resource(resource) = imported_manifest {
                            resources.push(resource);
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
