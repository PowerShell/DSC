// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::util::ProgressFormat;
use configure::config_doc::ExecutionKind;
use dscerror::DscError;
use dscresources::{dscresource::{DscResource, Invoke}, invoke_result::{GetResult, SetResult, TestResult}};

pub mod configure;
pub mod discovery;
pub mod dscerror;
pub mod dscresources;
pub mod functions;
pub mod parser;
pub mod util;

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
    pub fn new() -> Result<Self, DscError> {
        Ok(Self {
            discovery: discovery::Discovery::new()?,
        })
    }

    /// Find a resource by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the resource to find, can have wildcards.
    ///
    #[must_use]
    pub fn find_resource(&self, name: &str) -> Option<&DscResource> {
        self.discovery.find_resource(name)
    }

    pub fn list_available_resources(&mut self, type_name_filter: &str, adapter_name_filter: &str, progress_format: ProgressFormat) -> Vec<DscResource> {
        self.discovery.list_available_resources(type_name_filter, adapter_name_filter, progress_format)
    }

    pub fn find_resources(&mut self, required_resource_types: &[String], progress_format: ProgressFormat) {
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
        Self::new().unwrap()
    }
}
