pub mod discovery;
pub mod dscresources;
pub mod dscerror;

use dscerror::DscError;
use discovery::ResourceIterator;
use dscresources::{dscresource::{DscResource, Invoke}, invoke_result::{GetResult, SetResult, TestResult}};

pub struct DscManager {
    discovery: discovery::Discovery,
}

impl DscManager {
    pub fn new() -> Result<Self, DscError> {
        Ok(Self {
            discovery: discovery::Discovery::new()?,
        })
    }

    pub fn find_resource(&self, name: &str) -> ResourceIterator {
        self.discovery.find_resource(name)
    }

    pub fn resource_get(&self, resource: &DscResource, input: &str) -> Result<GetResult, DscError> {
        resource.get(input)
    }

    pub fn resource_set(&self, resource: &DscResource, input: &str) -> Result<SetResult, DscError> {
        resource.set(input)
    }

    pub fn resource_test(&self, resource: &DscResource, input: &str) -> Result<TestResult, DscError> {
        resource.test(input)
    }
}

impl Default for DscManager {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
