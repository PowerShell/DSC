use dscerror::DscError;
use resource_manifest::ResourceManifest;
use serde::{Deserialize, Serialize};
use super::{*, invoke_result::{GetResult, SetResult, TestResult}};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DscResource {
    #[serde(rename="ResourceType")]
    pub resource_type: Option<String>,
    #[serde(rename="Name")]
    pub name: String,
    #[serde(rename="FriendlyName")]
    pub friendly_name: Option<String>,
    #[serde(rename="Module")]
    pub module: Option<String>,
    #[serde(rename="ModuleName")]
    pub module_name: Option<String>,
    #[serde(rename="Version")]
    pub version: String,
    #[serde(rename="Path")]
    pub path: String,
    #[serde(rename="ParentPath")]
    pub parent_path: String,
    #[serde(rename="ImplementedAs")]
    pub implemented_as: ImplementedAs,
    #[serde(rename="CompanyName")]
    pub company_name: Option<String>,
    #[serde(rename="Properties")]
    pub properties: Vec<String>,
    #[serde(rename="Manifest")]
    pub manifest: Option<ResourceManifest>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ImplementedAs {
    PowerShell,
    PowerShellScript,   // .ps1
    Command,
}

impl DscResource {
    pub fn new() -> Self {
        Self {
            resource_type: None,
            name: String::new(),
            friendly_name: None,
            module: None,
            module_name: None,
            version: String::new(),
            path: String::new(),
            parent_path: String::new(),
            implemented_as: ImplementedAs::PowerShell,
            company_name: None,
            properties: Vec::new(),
            manifest: None,
        }
    }
}

impl Default for DscResource {
    fn default() -> Self {
        DscResource::new()
    }
}

pub trait Invoke {
    // the strings are expected to be json
    fn get(&self, filter: &str) -> Result<GetResult, DscError>;
    fn set(&self, desired: &str) -> Result<SetResult, DscError>;
    fn test(&self, expected: &str) -> Result<TestResult, DscError>;
}

impl Invoke for DscResource {
    fn get(&self, filter: &str) -> Result<GetResult, DscError> {
        match self.implemented_as {
            ImplementedAs::PowerShell => {
                Err(DscError::NotImplemented)
            },
            ImplementedAs::PowerShellScript => {
                Err(DscError::NotImplemented)
            },
            ImplementedAs::Command => {
                let manifest = match &self.manifest {
                    None => {
                        return Err(DscError::MissingManifest(self.name.clone()));
                    },
                    Some(manifest) => manifest,
                };
                command_resource::invoke_get(manifest, filter)
            },
        }
    }
    fn set(&self, _desired: &str) -> Result<SetResult, DscError> {
        Err(DscError::NotImplemented)
    }
    fn test(&self, _expected: &str) -> Result<TestResult, DscError> {
        match self.implemented_as {
            ImplementedAs::PowerShell => {
                Err(DscError::NotImplemented)
            },
            ImplementedAs::PowerShellScript => {
                Err(DscError::NotImplemented)
            },
            ImplementedAs::Command => {
                let manifest = match &self.manifest {
                    None => {
                        return Err(DscError::MissingManifest(self.name.clone()));
                    },
                    Some(manifest) => manifest,
                };
                command_resource::invoke_test(manifest, _expected)
            },
        }
    }
}
