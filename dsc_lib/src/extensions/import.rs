// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    dscerror::DscError,
    dscresources::command_resource::invoke_command,
    extensions::{
        dscextension::{Capability, DscExtension},
        extension_manifest::ExtensionManifest,
    },
};
use path_absolutize::Absolutize;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct ImportMethod {
    /// The extensions to import.
    #[serde(rename = "fileExtensions")]
    pub file_extensions: Vec<String>,
    /// The command to run to get the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform an Import.
    pub args: Option<Vec<ImportArgKind>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum ImportArgKind {
    /// The argument is a string.
    String(String),
    /// The argument accepts the file path.
    File {
        /// The argument that accepts the file path.
        #[serde(rename = "fileArg")]
        file_arg: String,
    },
}

impl DscExtension {
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
            let file_extension = file_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            if self
                .import_file_extensions
                .as_ref()
                .is_some_and(|exts| exts.contains(&file_extension))
            {
                debug!(
                    "{}",
                    t!(
                        "extensions.dscextension.importingFile",
                        file = file,
                        extension = self.type_name
                    )
                );
            } else {
                debug!(
                    "{}",
                    t!(
                        "extensions.dscextension.importNotSupported",
                        file = file,
                        extension = self.type_name
                    )
                );
                return Err(DscError::NotSupported(
                    t!(
                        "extensions.dscextension.importNotSupported",
                        file = file,
                        extension = self.type_name
                    )
                    .to_string(),
                ));
            }

            let extension = match serde_json::from_value::<ExtensionManifest>(self.manifest.clone()) {
                Ok(manifest) => manifest,
                Err(err) => {
                    return Err(DscError::Manifest(self.type_name.clone(), err));
                }
            };
            let Some(import) = extension.import else {
                return Err(DscError::UnsupportedCapability(
                    self.type_name.clone(),
                    Capability::Import.to_string(),
                ));
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
                info!(
                    "{}",
                    t!("extensions.dscextension.importNoResults", extension = self.type_name)
                );
            } else {
                return Ok(stdout);
            }
        }
        Err(DscError::UnsupportedCapability(
            self.type_name.clone(),
            Capability::Import.to_string(),
        ))
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
        return Err(DscError::Extension(
            t!("util.failedToAbsolutizePath", path = path : {:?}).to_string(),
        ));
    };

    let mut processed_args = Vec::<String>::new();
    for arg in arg_values {
        match arg {
            ImportArgKind::String(s) => {
                processed_args.push(s.clone());
            }
            ImportArgKind::File { file_arg } => {
                if !file_arg.is_empty() {
                    processed_args.push(file_arg.to_string());
                }
                processed_args.push(full_path.to_string_lossy().to_string());
            }
        }
    }

    Ok(Some(processed_args))
}
