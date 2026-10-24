// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    actions::action_manifest::{ActionManifest, ArgKind, InvokeMethod, SchemaArgKind, SchemaKind, SupportedOperations},
    configure::config_doc::{ExecutionKind, Operation},
    discovery::{command_discovery::verify_executable, ResourceKind},
    dscerror::DscError,
    dscresources::{command_resource::{invoke_command, validate_security_context}, dscresource::DscResource},
    resources::{
        invoke_result::{DeleteResultKind, GetResult, SetResult, ExportResult, ResolveResult, ResourceGetResponse, ResourceSetResponse, ResourceTestResponse, TestResult, ValidateResult},
        ManifestKind, resource_invoke::{Capability, ResourceInvoke},
    },
    schemas::dsc_repo::DscRepoSchema,
    types::{ExitCodesMap, FullyQualifiedTypeName, ResourceVersion,SemanticVersion}
};
use jsonschema::Validator;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use tracing::error;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "list", folder_path = "outputs/action")]
pub struct DscAction {
    /// The namespaced name of the extension.
    #[serde(rename="type")]
    pub type_name: FullyQualifiedTypeName,
    /// The version of the extension.
    pub version: SemanticVersion,
    /// The invoke specifics.
    pub invoke: InvokeMethod,
    /// The input schema for the action.  If not defined, then the action does not expect input.
    pub input_schema: Option<Value>,
    /// The output schema for the action.  If not defined, then the action does not produce output.
    pub output_schema: Option<Value>,
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
    /// The operations supported by the action.
    pub supported_operations: Vec<SupportedOperations>,
    /// The manifest of the extension.
    pub manifest: ActionManifest,
}

impl Default for DscAction {
    fn default() -> Self {
        Self::new()
    }
}

impl DscAction {
    #[must_use]
    pub fn new() -> Self {
        Self {
            type_name: FullyQualifiedTypeName::default(),
            version: SemanticVersion::default(),
            invoke: InvokeMethod::default(),
            input_schema: None,
            output_schema: None,
            path: PathBuf::new(),
            author: None,
            deprecation_message: None,
            description: None,
            directory: PathBuf::new(),
            supported_operations: Vec::new(),
            manifest: ActionManifest::default(),
        }
    }

    fn invoke_action(&self, input: &str, execution_type: &ExecutionKind) -> Result<Value, DscError> {
        if let Some(required_context) = &self.manifest.invoke.require_security_context {
            validate_security_context(None, &Some(required_context.clone()), &self.manifest.type_name, &Operation::Invoke)?;
        }

        if let Some(input_schema) = &self.input_schema {
            validate_json(if input.is_empty() { "null" } else { input }, input_schema)?;
        } else if !input.is_empty() {
            error!("{}", t!("actions.dscaction.inputSchemaNotAvailable", type_name = self.type_name.to_string()).to_string());
            return Err(DscError::SchemaNotAvailable(self.type_name.to_string()));
        }
        let (args, supports_whatif) = process_invoke_args(self.manifest.invoke.args.as_ref(), input, execution_type);
        if execution_type == &ExecutionKind::WhatIf && !supports_whatif {
            return Err(DscError::Operation(t!("actions.dscaction.whatIfNotSupported", type_name = self.type_name.to_string()).to_string()));
        }

        // when the input is passed via a JSON argument, don't also write it to stdin to avoid pipe deadlocks
        let passes_input_as_arg = self.manifest.invoke.args.as_ref().is_some_and(|args| args.iter().any(|arg| matches!(arg, ArgKind::Json { .. })));
        let input = if input.is_empty() || passes_input_as_arg {
            None
        } else {
            Some(input)
        };

        let (_exit_code, stdout, _stderr) = invoke_command(&self.manifest.invoke.executable, args, input, Some(&self.directory), None, self.manifest.exit_codes.as_ref())?;
        if stdout.trim().is_empty() {
            return Ok(Value::Null);
        }
        if let Some(output_schema) = self.output_schema.as_ref().or(self.input_schema.as_ref()) {
            validate_json(&stdout, output_schema)?;
        }

        let output = serde_json::from_str(&stdout)?;
        Ok(output)
    }
}

impl ResourceInvoke for DscAction {
    fn get(&self, filter: &str) -> Result<GetResult, DscError> {
        let result = if self.supported_operations.contains(&SupportedOperations::Get) {
            self.invoke_action(filter, &ExecutionKind::Actual)?
        } else {
            return Err(DscError::NotSupported(t!("actions.dscaction.actionUnsupportedOperation", operation = SupportedOperations::Get, action = &self.type_name).to_string()));
        };
        let get_result_response = ResourceGetResponse {
            actual_state: result.clone()
        };
        Ok(GetResult::Resource(get_result_response))
    }

    fn set(&self, desired: &str, _skip_test: bool, execution_type: &ExecutionKind) -> Result<SetResult, DscError> {
        let result = if self.supported_operations.contains(&SupportedOperations::Set) {
            self.invoke_action(desired, execution_type)?
        } else {
            return Err(DscError::NotSupported(t!("actions.dscaction.actionUnsupportedOperation", operation = SupportedOperations::Set, action = &self.type_name).to_string()));
        };
        let set_result_response = ResourceSetResponse {
            before_state: result.clone(),
            after_state: result.clone(),
            changed_properties: None,
        };
        Ok(SetResult::Resource(set_result_response))
    }

    fn test(&self, expected: &str) -> Result<TestResult, DscError> {
        let result = if self.supported_operations.contains(&SupportedOperations::Test) {
            self.invoke_action(expected, &ExecutionKind::Actual)?
        } else {
            return Err(DscError::NotSupported(t!("actions.dscaction.actionUnsupportedOperation", operation = SupportedOperations::Test, action = &self.type_name).to_string()));
        };
        let test_response = ResourceTestResponse {
            desired_state: result.clone(),
            actual_state: result.clone(),
            in_desired_state: true,
            diff_properties: Vec::new(),
        };
        Ok(TestResult::Resource(test_response))
    }

    fn export(&self, input: &str) -> Result<ExportResult, DscError> {
        let result = if self.supported_operations.contains(&SupportedOperations::Export) {
            self.invoke_action(input, &ExecutionKind::Actual)?
        } else {
            return Err(DscError::NotSupported(t!("actions.dscaction.actionUnsupportedOperation", operation = SupportedOperations::Export, action = &self.type_name).to_string()));
        };
        Ok(ExportResult {
            actual_state: vec![result.clone()]
        })
    }

    fn invoke(&self, input: &str, execution_type: &ExecutionKind) -> Result<Value, DscError> {
        self.invoke_action(input, execution_type)
    }

    fn delete(&self, _filter: &str, _execution_type: &ExecutionKind) -> Result<DeleteResultKind, DscError> {
        Err(DscError::NotSupported(t!("actions.dscaction.actionUnsupportedOperation", operation = "delete", action = &self.type_name).to_string()))
    }

    fn resolve(&self, _input: &str) -> Result<ResolveResult, DscError> {
        Err(DscError::NotSupported(t!("actions.dscaction.actionUnsupportedOperation", operation = "resolve", action = &self.type_name).to_string()))
    }

    fn schema(&self) -> Result<String, DscError> {
        Err(DscError::NotSupported(t!("actions.dscaction.actionUnsupportedOperation", operation = "schema", action = &self.type_name).to_string()))
    }

    fn validate(&self, _config: &str) -> Result<ValidateResult, DscError> {
        Err(DscError::NotSupported(t!("actions.dscaction.actionUnsupportedOperation", operation = "validate", action = &self.type_name).to_string()))
    }

    fn kind(&self) -> crate::discovery::ResourceKind {
        ResourceKind::Action
    }

    fn is_group(&self) -> bool {
        false
    }

    fn capabilities(&self) -> Vec<Capability> {
        // convert the supported_operations to capabilities
        let mut capabilties = vec![];
        for op in &self.supported_operations {
            match op {
                SupportedOperations::Get => capabilties.push(Capability::Get),
                SupportedOperations::Set => capabilties.push(Capability::Set),
                SupportedOperations::Test => capabilties.push(Capability::Test),
                SupportedOperations::Export => capabilties.push(Capability::Export),
            }
        }
        capabilties
    }


    fn type_name(&self) -> FullyQualifiedTypeName {
        self.type_name.clone()
    }

    fn version(&self) -> ResourceVersion {
        ResourceVersion::Semantic(self.version.clone())
    }

    fn manifest(&self) -> Option<ManifestKind> {
        Some(ManifestKind::Action(Box::new(self.manifest.clone())))
    }

    fn schema_as_map(&self) -> Option<Map<String, Value>> {
        // convert the input_schema as a Map<String, Value>
        match &self.input_schema {
            Some(value) => {
                let map: Map<String, Value> = serde_json::from_value(value.clone()).ok()?;
                Some(map)
            },
            None => None,
        }
    }

    fn set_target_resource(&mut self, _target: Box<DscResource>) -> Result<(), DscError> {
        Err(DscError::NotSupported(t!("actions.dscaction.actionUnsupportedOperation", operation = "set_target_resource", action = &self.type_name).to_string()))
    }

    fn as_action(&self) -> Option<&DscAction> {
        Some(self)
    }

    fn as_dsc_resource(&self) -> Option<&DscResource> {
        None
    }

    fn clone_box(&self) -> Box<dyn ResourceInvoke> {
        Box::new(self.clone())
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
    let compiled_schema = match Validator::new(schema) {
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

fn process_invoke_args(args: Option<&Vec<ArgKind>>, input: &str, execution_type: &ExecutionKind) -> (Option<Vec<String>>, bool) {
    let Some(arg_values) = args else {
        return (None, false);
    };

    let mut supports_whatif = false;
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
            },
            ArgKind::WhatIf { what_if_arg } => {
                supports_whatif = true;
                if execution_type == &ExecutionKind::WhatIf {
                    processed_args.push(what_if_arg.clone());
                }
            }
        }
    }

    (Some(processed_args), supports_whatif)
}

fn process_schema_args(args: Option<&Vec<SchemaArgKind>>) -> Option<Vec<String>> {
    let arg_values = args?;

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
    // inline manifests have no real file path, so fall back to the current directory
    let directory = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() && parent.is_dir() => parent.to_path_buf(),
        _ => std::env::current_dir()?,
    };
    verify_executable(&manifest.type_name, "invoke", &manifest.invoke.executable, &directory);
    let mut supported_operations = Vec::new();
    if let Some(operations) = &manifest.supported_operations {
        supported_operations.extend(operations.clone());
    } else {
        supported_operations.push(SupportedOperations::Set);
    }
    let input_schema = if let Some(input_schema_kind) = manifest.invoke.input_schema.as_ref() {
        Some(get_schema(input_schema_kind, &directory, manifest.exit_codes.as_ref())?)
    } else {
        None
    };
    let output_schema = if let Some(output_schema_kind) = manifest.invoke.output_schema.as_ref() {
        Some(get_schema(output_schema_kind, &directory, manifest.exit_codes.as_ref())?)
    } else {
        None
    };

    let action = DscAction {
        type_name: manifest.type_name.clone(),
        version: manifest.version.clone(),
        invoke: manifest.invoke.clone(),
        input_schema,
        output_schema,
        path: path.to_path_buf(),
        author: manifest.author.clone(),
        deprecation_message: manifest.deprecation_message.clone(),
        description: manifest.description.clone(),
        directory,
        supported_operations,
        manifest: manifest.clone(),
    };
    Ok(action)
}
