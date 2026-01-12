// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::discovery::{command_discovery::ImportedManifest, discovery_trait::DiscoveryFilter};
use crate::discovery::discovery_trait::DiscoveryKind;
use crate::progress::ProgressFormat;

use configure::config_doc::ExecutionKind;
use dscerror::DscError;
use dscresources::{dscresource::{DscResource, Invoke}, invoke_result::{GetResult, SetResult, TestResult}};
use rust_i18n::i18n;

pub mod configure;
pub mod discovery;
pub mod dscerror;
pub mod dscresources;
pub mod extensions;
pub mod functions;
pub mod parser;
pub mod progress;
pub mod types;
pub mod util;

// Re-export the dependency crate to minimize dependency management.
pub use dsc_lib_jsonschema as schemas;

i18n!("locales", fallback = "en-us");

pub struct DscManager {
    discovery: discovery::Discovery,
}

impl DscManager {
    /// Create a new `DscManager` instance.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying discovery fails.
    ///
    #[must_use]
    pub fn new() -> Self {
        Self {
            discovery: discovery::Discovery::new(),
        }
    }

    /// Find a resource by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the resource to find, can have wildcards.
    ///
    #[must_use]
    pub fn find_resource(&mut self, name: &str, version: Option<&str>) -> Option<&DscResource> {
        self.discovery.find_resource(name, version)
    }

    pub fn list_available(&mut self, kind: &DiscoveryKind, type_name_filter: &str, adapter_name_filter: &str, progress_format: ProgressFormat) -> Vec<ImportedManifest> {
        self.discovery.list_available(kind, type_name_filter, adapter_name_filter, progress_format)
    }

    pub fn find_resources(&mut self, required_resource_types: &[DiscoveryFilter], progress_format: ProgressFormat) {
        self.discovery.find_resources(required_resource_types, progress_format);
    }
    /// Invoke the get operation on a resource.
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to invoke the operation on.
    /// * `input` - The input to the operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    ///
    pub fn resource_get(&self, resource: &DscResource, input: &str) -> Result<GetResult, DscError> {
        resource.get(input)
    }

    /// Invoke the set operation on a resource.
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to invoke the operation on.
    /// * `input` - The input to the operation.
    /// * `skip_test` - Whether to skip the test operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    ///
    pub fn resource_set(&self, resource: &DscResource, input: &str, skip_test: bool) -> Result<SetResult, DscError> {
        resource.set(input, skip_test, &ExecutionKind::Actual)
    }

    /// Invoke the test operation on a resource.
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to invoke the operation on.
    /// * `input` - The input to the operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    ///
    pub fn resource_test(&self, resource: &DscResource, input: &str) -> Result<TestResult, DscError> {
        resource.test(input)
    }
}

impl Default for DscManager {
    fn default() -> Self {
        Self::new()
    }
}
