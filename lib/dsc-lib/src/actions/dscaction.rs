// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    actions::action_manifest::{
        ActionManifest,
        ArgKind,
        InvokeMethod,
        SchemaArgKind,
        SchemaKind
    },
    discovery::command_discovery::verify_executable,
    dscerror::DscError,
    dscresources::{
        command_resource::{
            invoke_command,
            validate_security_context
        },
        dscresource::Operation
    },
    schemas::dsc_repo::DscRepoSchema,
    types::{
        ExitCodesMap,
        FullyQualifiedTypeName,
        SemanticVersion
    },
};
use jsonschema::Validator;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(base_name = "list", folder_path = "outputs/action")]
pub struct DscAction {
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

    pub(crate) fn invoke(&self, input: Option<&str>) -> Result<Value, DscError> {
        let manifest = serde_json::from_value::<ActionManifest>(self.manifest.clone())?;

        if let Some(required_context) = &manifest.invoke.require_security_context {
            validate_security_context(None, &Some(required_context.clone()), &manifest.type_name, &Operation::Invoke)?;
        }

        let Some(input_schema_kind) = manifest.invoke.input_schema.as_ref() else {
            return Err(DscError::SchemaNotAvailable(self.type_name.to_string()));
        };
        let input_schema = get_schema(input_schema_kind, &self.directory, manifest.exit_codes.as_ref())?;
        if let Some(input) = input {
            validate_json(&input, &input_schema)?;
        }
        let args = process_invoke_args(manifest.invoke.args.as_ref(), input.unwrap_or(""));
        let (_exit_code, stdout, _stderr) = invoke_command(&manifest.invoke.executable, args, input, Some(&self.directory), None, manifest.exit_codes.as_ref())?;
        let Some(output_schema_kind) = manifest.invoke.output_schema.as_ref() else {
            return Err(DscError::SchemaNotAvailable(self.type_name.to_string()));
        };
        let output_schema = get_schema(output_schema_kind, &self.directory, manifest.exit_codes.as_ref())?;
        validate_json(&stdout, &output_schema)?;

        let output = serde_json::from_str(&stdout)?;
        Ok(output)
    }
}

pub fn get_schema(schema_kind: &SchemaKind, directory: &Path, exit_codes: &ExitCodesMap) -> Result<Value, DscError> {
    match schema_kind {
        SchemaKind::Command(command) => {
            let args = process_schema_args(command.args.as_ref());
            let (_exit_code, stdout, _stderr) = invoke_command(&command.executable, args, None, Some(directory), None, exit_codes)?;
            let schema_value: Value = serde_json::from_str(&stdout)?;
            Ok(schema_value)
        },
        SchemaKind::Embedded(schema) => Ok(schema.clone()),
    }
}

fn validate_json(input: &str, schema: &Value) -> Result<(), DscError> {
    let compiled_schema = match Validator::new(&schema) {
        Ok(schema) => schema,
        Err(e) => {
            return Err(DscError::Schema(e.to_string()));
        },
    };
    let input_value = serde_json::from_str(input)?;
    if let Err(err) = compiled_schema.validate(&input_value) {
        return Err(DscError::Schema(err.to_string()));
    }
    Ok(())
}

fn process_invoke_args(args: Option<&Vec<ArgKind>>, input: &str) -> Option<Vec<String>> {
    let Some(arg_values) = args else {
        return None;
    };

    let mut processed_args = Vec::<String>::new();
    for arg in arg_values {
        match arg {
            ArgKind::String(s) => {
                processed_args.push(s.clone());
            },
            ArgKind::Json{ json_input_arg, mandatory } => {
                if input.is_empty() && *mandatory != Some(true) {
                    continue;
                }

                processed_args.push(json_input_arg.clone());
                processed_args.push(input.to_string());
            }
        }
    }

    Some(processed_args)
}

fn process_schema_args(args: Option<&Vec<SchemaArgKind>>) -> Option<Vec<String>> {
    let Some(arg_values) = args else {
        return None;
    };

    let mut processed_args = Vec::<String>::new();
    for arg in arg_values {
        match arg {
            SchemaArgKind::String(s) => {
                processed_args.push(s.clone());
            },
        }
    }

    Some(processed_args)
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
