// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    dscerror::DscError,
    dscresources::{
        command_resource::invoke_command,
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
use tracing::{debug, warn};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum SecretArgKind {
    /// The argument is a string.
    String(String),
    /// The argument accepts the secret name.
    Name {
        /// The argument that accepts the secret name.
        #[serde(rename = "nameArg")]
        name_arg: String,
    },
    /// The argument accepts the vault name.
    Vault {
        /// The argument that accepts the vault name.
        #[serde(rename = "vaultArg")]
        vault_arg: String,
    },
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.secret", folder_path = "extension")]
pub struct SecretMethod {
    /// The command to run to get the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Get.
    pub args: Option<Vec<SecretArgKind>>,
}

impl DscExtension {
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
                    return Err(DscError::Manifest(self.type_name.to_string(), err));
                }
            };
            let Some(secret) = extension.secret else {
                return Err(DscError::UnsupportedCapability(self.type_name.to_string(), Capability::Secret.to_string()));
            };
            let args = process_secret_args(secret.args.as_ref(), name, vault);
            if let Some(deprecation_message) = extension.deprecation_message.as_ref() {
                warn!("{}", t!("extensions.dscextension.deprecationMessage", extension = self.type_name, message = deprecation_message));
            }
            let (_exit_code, stdout, _stderr) = invoke_command(
                &secret.executable,
                args,
                vault,
                Some(&self.directory),
                None,
                extension.exit_codes.as_ref(),
            )?;
            if stdout.is_empty() {
                debug!("{}", t!("extensions.dscextension.extensionReturnedNoSecret", extension = self.type_name));
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
                self.type_name.to_string(),
                Capability::Secret.to_string()
            ))
        }
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
