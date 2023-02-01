// reuse code from https://github.com/PowerShell/MSH/blob/main/config/src/main.rs

use crate::dscerror::DscError;
use crate::dscresources::dscresource::Invoke;

pub struct NativeResource {
    pub resource: String,
    // TODO: need members to represent path to the command and how to call it for get, set, test
}

impl Invoke for NativeResource {
    fn get(&self, _filter: &str) -> Result<String, DscError> {
        Err(DscError::NotImplemented)
    }

    fn set(&self, _desired: &str) -> Result<String, DscError> {
        Err(DscError::NotImplemented)
    }

    fn test(&self, _expected: &str) -> Result<String, DscError> {
        Err(DscError::NotImplemented)
    }
}
