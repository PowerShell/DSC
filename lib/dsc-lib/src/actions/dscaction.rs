// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    actions::action_manifest::{ActionManifest, InvokeMethod, SchemaKind},
    discovery::command_discovery::verify_executable,
    dscerror::DscError,
    dscresources::{
        command_resource::process_schema_args,
    },
    schemas::dsc_repo::DscRepoSchema,
    types::{FullyQualifiedTypeName, SemanticVersion},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(base_name = "list", folder_path = "outputs/action")]
pub(crate) struct DscAction {
    /// The namespaced name of the extension.
    #[serde(rename="type")]
    pub type_name: FullyQualifiedTypeName,
    /// The version of the extension.
    pub version: SemanticVersion,
    /// The invoke specifics.
    pub invoke: InvokeMethod,
    /// The file path to the extension.
    pub path: PathBuf,
    /// The author of the extension.
    pub author: Option<String>,
    /// An optional message indicating the extension is deprecated.  If provided, the message will be shown when the extension is used.
    pub deprecation_message: Option<String>,
    /// The description of the extension.
    pub description: Option<String>,
    /// The directory path to the extension.
    pub directory: PathBuf,
    /// The manifest of the extension.
    pub manifest: Value,
}

impl DscAction {
    #[must_use]
    pub fn new() -> Self {
        Self {
            type_name: FullyQualifiedTypeName::default(),
            version: SemanticVersion::default(),
            invoke: InvokeMethod::default(),
            path: PathBuf::new(),
            author: None,
            deprecation_message: None,
            description: None,
            directory: PathBuf::new(),
            manifest: Value::Null,
        }
    }

    pub fn invoke(&self, input: Option<Value>) -> Result<Value, DscError> {
        // validate input against input_schema
        let manifest = serde_json::from_value::<ActionManifest>(self.manifest.clone())?;
        let Some(schema_kind) = manifest.invoke.input_schema.as_ref() else {
            return Err(DscError::SchemaNotAvailable(self.type_name.to_string()));
        };
        let schema_value = match schema_kind {
            SchemaKind::Command(command) => {
                let args = process_schema_args(command.args.as_ref(), target_resource);
                let (_exit_code, stdout, _stderr) = invoke_command(&command.executable, args, None, Some(&resource.directory), None, manifest.exit_codes.as_ref())?;
                let schema_value: Value = serde_json::from_str(&stdout)?;
                schema_value
            },
            SchemaKind::Embedded(schema) => {
                schema.clone()
            }
        };
        // validate input against schema_value
        if let Some(input) = input {
            validate_input(&input, &schema_value)?;
        }
        Ok(Value::Null)
    }
}

pub(crate) fn load_action_manifest(path: &Path, manifest: &ActionManifest) -> Result<DscAction, DscError> {
    verify_executable(&manifest.type_name, "invoke", &manifest.invoke.executable, path.parent().unwrap());
    let action = DscAction {
        type_name: manifest.type_name.clone(),
        version: manifest.version.clone(),
        invoke: manifest.invoke.clone(),
        path: path.to_path_buf(),
        author: manifest.author.clone(),
        deprecation_message: manifest.deprecation_message.clone(),
        description: manifest.description.clone(),
        directory: path.parent().unwrap().to_path_buf(),
        manifest: serde_json::to_value(manifest)?,
    };
    Ok(action)
}
