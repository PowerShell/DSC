// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    configure::context::Context, dscerror::DscError, dscresources::command_resource::invoke_command, extensions::{
        dscextension::{
            Capability,
            DscExtension,
        },
        extension_manifest::ExtensionManifest,
    },
    parser::Statement,
    schemas::dsc_repo::DscRepoSchema
};

use path_absolutize::Absolutize;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, warn};

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.import", folder_path = "extension")]
pub struct ImportMethod {
    /// The extensions to import.
    #[serde(rename = "fileExtensions")]
    pub file_extensions: Vec<String>,
    /// The command to run to get the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform an Import.
    pub args: Option<Vec<ImportArgKind>>,
    /// Enables modifying the resulting output from STDOUT after running the import command.
    pub output: Option<String>,
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
    pub fn import(&self, file: &Path) -> Result<String, DscError> {
        if let Some(import) = &self.import {
            let file_extension = file.extension().and_then(|s| s.to_str()).unwrap_or_default().to_string();
            if import.file_extensions.contains(&file_extension) {
                debug!("{}", t!("extensions.dscextension.importingFile", file = file.display(), extension = self.type_name));
            } else {
                return Err(DscError::NotSupported(
                    t!("extensions.dscextension.importNotSupported", file = file.display(), extension = self.type_name).to_string(),
                ));
            }

            let extension = match serde_json::from_value::<ExtensionManifest>(self.manifest.clone()) {
                Ok(manifest) => manifest,
                Err(err) => {
                    return Err(DscError::Manifest(self.type_name.to_string(), err));
                }
            };
            let Some(import) = extension.import else {
                return Err(DscError::UnsupportedCapability(self.type_name.to_string(), Capability::Import.to_string()));
            };
            let args = process_import_args(import.args.as_ref(), file)?;
            if let Some(deprecation_message) = extension.deprecation_message.as_ref() {
                warn!("{}", t!("extensions.dscextension.deprecationMessage", extension = self.type_name, message = deprecation_message));
            }
            let (_exit_code, stdout, _stderr) = invoke_command(
                &import.executable,
                args,
                None,
                Some(&self.directory),
                None,
                extension.exit_codes.as_ref(),
            )?;
            if stdout.is_empty() {
                info!("{}", t!("extensions.dscextension.importNoResults", extension = self.type_name));
            } else {
                if let Some(output) = &import.output {
                    debug!("{}", t!("extensions.dscextension.importProcessingOutput", extension = self.type_name));
                    let mut parser = Statement::new()?;
                    let mut context = Context::new();
                    context.stdout = Some(stdout);
                    let processed_output = parser.parse_and_execute(output, &context)?;
                    return Ok(processed_output.to_string());
                }
                return Ok(stdout);
            }
        }
        Err(DscError::UnsupportedCapability(
            self.type_name.to_string(),
            Capability::Import.to_string()
        ))
    }
}

fn process_import_args(args: Option<&Vec<ImportArgKind>>, file: &Path) -> Result<Option<Vec<String>>, DscError> {
    let Some(arg_values) = args else {
        debug!("{}", t!("dscresources.commandResource.noArgs"));
        return Ok(None);
    };

    // make path absolute
    let Ok(full_path) = file.absolutize() else {
        return Err(DscError::Extension(t!("util.failedToAbsolutizePath", path = file.display()).to_string()));
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
