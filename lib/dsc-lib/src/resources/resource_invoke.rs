// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    actions::dscaction::DscAction, configure::config_doc::ExecutionKind, discovery::ResourceKind, dscerror::DscError, dscresources::dscresource::DscResource, resources::{ManifestKind, invoke_result::{DeleteResultKind, ExportResult, GetResult, ResolveResult, SetResult, TestResult, ValidateResult}}, schemas::{dsc_repo::DscRepoSchema, transforms::idiomaticize_string_enum}, types::{FullyQualifiedTypeName, ResourceVersion},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt::Display;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "resourceCapabilities", folder_path = "definitions")]
#[schemars(
    transform = idiomaticize_string_enum,
    transform = Capability::transform_export_schema_uris,
    transform = Capability::transform_schema_docs
)]
pub enum Capability {
    /// The resource supports retrieving configuration.
    Get,
    /// The resource supports applying configuration.
    Set,
    /// The resource supports the `_exist` property directly.
    SetHandlesExist,
    /// The resource supports simulating configuration directly for `set`
    SetWhatIf,
    /// The resource supports validating configuration.
    Test,
    /// The resource supports deleting configuration.
    Delete,
    /// The resource supports simulating configuration directly for `delete`
    DeleteWhatIf,
    /// The resource supports exporting configuration.
    Export,
    /// The resource supports resolving imported configuration.
    Resolve,
    /// The resource supports invoking an action.
    Invoke,
}

impl Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::Get => write!(f, "get"),
            Capability::Set => write!(f, "set"),
            Capability::SetHandlesExist => write!(f, "setHandlesExist"),
            Capability::SetWhatIf => write!(f, "setWhatIf"),
            Capability::Test => write!(f, "test"),
            Capability::Delete => write!(f, "delete"),
            Capability::DeleteWhatIf => write!(f, "deleteWhatIf"),
            Capability::Export => write!(f, "export"),
            Capability::Resolve => write!(f, "resolve"),
            Capability::Invoke => write!(f, "invoke"),
        }
    }
}

/// The interface for a Resource.
pub trait ResourceInvoke: Send + Sync {
    /// Invoke the get operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter as JSON to apply to the resource.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn get(&self, filter: &str) -> Result<GetResult, DscError>;

    /// Invoke the set operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `desired` - The desired state as JSON to apply to the resource.
    /// * `skip_test` - Whether to skip the test operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn set(&self, desired: &str, skip_test: bool, execution_type: &ExecutionKind) -> Result<SetResult, DscError>;

    /// Invoke the test operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `expected` - The expected state as JSON to apply to the resource.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn test(&self, expected: &str) -> Result<TestResult, DscError>;

    /// Invoke the delete operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter as JSON to apply to the resource.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn delete(&self, filter: &str, execution_type: &ExecutionKind) -> Result<DeleteResultKind, DscError>;

    /// Invoke the invoke operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `input` - The input to the operation to be invoked.
    /// * `execution_type` - The execution type for the operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn invoke(&self, input: &str, execution_type: &ExecutionKind) -> Result<Value, DscError>;

    /// Invoke the validate operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration as JSON to have the resource validate.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource or validation fails.
    fn validate(&self, config: &str) -> Result<ValidateResult, DscError>;

    /// Get the schema for the resource.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn schema(&self) -> Result<String, DscError>;

    /// Invoke the export operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `input` - Input for export operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn export(&self, input: &str) -> Result<ExportResult, DscError>;

    /// Invoke the resolve operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `input` - The input to the operation to be resolved.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn resolve(&self, input: &str) -> Result<ResolveResult, DscError>;

    /// Get the kind of the resource (e.g., Action or Resource).
    ///
    /// # Returns
    ///
    /// The kind of the resource (e.g., Action or Resource).
    fn kind(&self) -> ResourceKind;

    /// Check if the resource is group type.
    ///
    /// # Returns
    ///
    /// `true` if the resource is grouped, `false` otherwise.
    fn is_group(&self) -> bool;

    /// Get the capabilities of the resource.
    ///
    /// # Returns
    ///
    /// The capabilities of the resource.
    fn capabilities(&self) -> Vec<Capability>;

    /// Get the manifest of the resource.
    ///
    /// # Returns
    ///
    /// The manifest of the resource, if available.
    fn manifest(&self) -> Option<ManifestKind>;

    /// Get the schema of the resource.
    ///
    /// # Returns
    ///
    /// The schema of the resource, if available.
    fn schema_as_map(&self) -> Option<Map<String, Value>>;

    /// Get the fully qualified type name of the resource.
    ///
    /// # Returns
    ///
    /// The fully qualified type name of the resource.
    fn type_name(&self) -> FullyQualifiedTypeName;

    /// Get the version of the resource.
    ///
    /// # Returns
    ///
    /// The version of the resource.
    fn version(&self) -> ResourceVersion;

    /// Set the target resource for this resource.
    ///
    /// # Arguments
    ///
    /// * `target` - The target resource to be set.
    ///
    /// # Errors
    ///
    /// This function will return an error if setting the target resource fails.
    fn set_target_resource(&mut self, target: Box<DscResource>) -> Result<(), DscError>;

    /// Get the underlying DSC resource, if available.
    ///
    /// # Returns
    ///
    /// The underlying DSC resource, if available.
    fn as_dsc_resource(&self) -> Option<&DscResource>;

    /// Get the underlying DSC action, if available.
    ///
    /// # Returns
    ///
    /// The underlying DSC action, if available.
    fn as_action(&self) -> Option<&DscAction>;

    /// Clone the resource invoke object into a boxed trait object.
    ///
    /// # Returns
    ///
    /// The cloned boxed trait object.
    fn clone_box(&self) -> Box<dyn ResourceInvoke>;
}

impl Clone for Box<dyn ResourceInvoke> {
    fn clone(&self) -> Box<dyn ResourceInvoke> {
        self.clone_box()
    }
}
