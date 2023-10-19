// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod configure;
pub mod discovery;
pub mod dscresources;
pub mod dscerror;
pub mod functions;
pub mod parser;

use dscerror::DscError;
use discovery::ResourceIterator;
use dscresources::{dscresource::{DscResource, Invoke}, invoke_result::{GetResult, SetResult, TestResult}};

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

    /// Initialize the discovery process.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying discovery fails.
    ///
    pub fn initialize_discovery(&mut self) -> Result<(), DscError> {
        self.discovery.initialize()
    }

    /// Find a resource by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the resource to find, can have wildcards.
    ///
    #[must_use]
    pub fn find_resource(&self, name: &str) -> ResourceIterator {
        self.discovery.find_resource(name)
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
        resource.set(input, skip_test)
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
