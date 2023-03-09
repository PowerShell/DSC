use dscerror::DscError;
use resource_manifest::ResourceManifest;
use serde::Serialize;
use super::*;

#[derive(Clone, Debug, Serialize)]
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

#[derive(Clone, Debug, Serialize)]
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
    fn get(&self, filter: &str) -> Result<String, DscError>;
    fn set(&self, desired: &str) -> Result<String, DscError>;
    fn test(&self, expected: &str) -> Result<String, DscError>; // result json should include a `_inDesiredState` bool property and optional additional json for the diff
}

impl Invoke for DscResource {
    fn get(&self, filter: &str) -> Result<String, DscError> {
        Err(DscError::NotImplemented)
    }
    fn set(&self, desired: &str) -> Result<String, DscError> {
        Err(DscError::NotImplemented)
    }
    fn test(&self, expected: &str) -> Result<String, DscError> {
        Err(DscError::NotImplemented)
    }
}
