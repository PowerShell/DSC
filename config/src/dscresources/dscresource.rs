use dscerror::DscError;
use serde::Serialize;
use super::*;

#[derive(Debug, Serialize)]
pub struct DscResource {
    #[serde(rename="ImplementationDetail")]
    pub implementation_detail: String,
    #[serde(rename="ResourceType")]
    pub resource_type: String,
    #[serde(rename="Name")]
    pub name: String,
    #[serde(rename="FriendlyName")]
    pub friendly_name: String,
    #[serde(rename="Module")]
    pub module: String,
    #[serde(rename="ModuleName")]
    pub module_name: String,
    #[serde(rename="Version")]
    pub version: String,
    #[serde(rename="Path")]
    pub path: String,
    #[serde(rename="ParentPath")]
    pub parent_path: String,
    #[serde(rename="ImplementedAs")]
    pub implemented_as: ImplementedAs,
    #[serde(rename="CompanyName")]
    pub company_name: String,
    #[serde(rename="Properties")]
    pub properties: Vec<String>,
    command_line: String,
}

#[derive(Debug, Serialize)]
pub enum ImplementedAs {
    PowerShell,
    PowerShellScript,   // .ps1
    Command,
}

impl DscResource {
    pub fn new() -> Self {
        Self {
            implementation_detail: String::new(),
            resource_type: String::new(),
            name: String::new(),
            friendly_name: String::new(),
            module: String::new(),
            module_name: String::new(),
            version: String::new(),
            path: String::new(),
            parent_path: String::new(),
            implemented_as: ImplementedAs::PowerShell,
            company_name: String::new(),
            properties: Vec::new(),
            command_line: String::new(),
        }
    }
}

impl Default for DscResource {
    fn default() -> Self {
        DscResource::new()
    }
}

pub trait Invoke {
    fn get(&self) -> Result<(), DscError>;  // TODO: does it return JSON or a struct?
    fn set(&self) -> Result<(), DscError>;
    fn test(&self, expected: &str) -> Result<(), DscError>;
}

impl Invoke for DscResource {
    fn get(&self) -> Result<(), DscError> {
        Err(DscError::NotImplemented)
    }
    fn set(&self) -> Result<(), DscError> {
        Err(DscError::NotImplemented)
    }
    fn test(&self, _expected: &str) -> Result<(), DscError> {
        Err(DscError::NotImplemented)
    }
}
