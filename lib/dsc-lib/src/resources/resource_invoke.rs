// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    actions::{action_manifest::SupportedOperation, dscaction::DscAction},
    configure::config_doc::ExecutionKind,
    discovery::ResourceKind,
    dscerror::DscError,
    dscresources::dscresource::{DscResource, ResourceCapability},
    resources::{
        ManifestKind,
        invoke_result::{DeleteResultKind, ExportResult, GetResult, ResolveResult, SetResult, TestResult, ValidateResult}
    },
    types::{FullyQualifiedTypeName, ResourceVersion},
};

// use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
// use std::fmt::Display;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Capabilities {
    Action(Vec<SupportedOperation>),
    Resource(Vec<ResourceCapability>)
}

impl Capabilities {
    pub fn as_action(&self) -> Option<&Vec<SupportedOperation>> {
        match self {
            Capabilities::Action(ops) => Some(ops),
            _ => None,
        }
    }
    pub fn as_resource(&self) -> Option<&Vec<ResourceCapability>> {
        match self {
            Capabilities::Resource(caps) => Some(caps),
            _ => None,
        }
    }
}

// impl Display for Capability {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Capability::Get => write!(f, "get"),
//             Capability::Set => write!(f, "set"),
//             Capability::SetHandlesExist => write!(f, "setHandlesExist"),
//             Capability::SetWhatIf => write!(f, "setWhatIf"),
//             Capability::Test => write!(f, "test"),
//             Capability::Delete => write!(f, "delete"),
//             Capability::DeleteWhatIf => write!(f, "deleteWhatIf"),
//             Capability::Export => write!(f, "export"),
//             Capability::Resolve => write!(f, "resolve"),
//             Capability::Invoke => write!(f, "invoke"),
//         }
//     }
// }

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
    fn capabilities(&self) -> Capabilities;

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

    /// Checks whether the resource has the specified [`ResourceCapability`].
    ///
    /// # Arguments
    ///
    /// - `cap` - The resource capability to check.
    ///
    /// # Returns
    ///
    /// `true` if the resource has the specified capability, `false` otherwise.
    fn has_resource_capability(&self, cap: &ResourceCapability) -> bool {
        self.capabilities()
            .as_resource()
            .is_some_and(|caps| caps.contains(cap))
    }

    /// Checks whether the resource supports the specified [`SupportedOperation`].
    ///
    /// # Arguments
    ///
    /// - `op` - The action operation to check.
    ///
    /// # Returns
    ///
    /// `true` if the resource supports the specified operation, `false` otherwise.
    fn supports_action_operation(&self, op: &SupportedOperation) -> bool {
        self.capabilities()
            .as_action()
            .is_some_and(|ops| ops.contains(op))
    }
}

impl Clone for Box<dyn ResourceInvoke> {
    fn clone(&self) -> Box<dyn ResourceInvoke> {
        self.clone_box()
    }
}
