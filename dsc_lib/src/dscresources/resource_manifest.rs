use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::dscerror::DscError;
use crate::dscresources::dscresource::Invoke;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ResourceManifest {
    #[serde(rename = "manifestVersion")]
    pub manifest_version: String,
    pub name: String,
    pub version: String,
    pub get: ResourceMethod,
    pub set: ResourceMethod,
    pub test: ResourceMethod,
    #[serde(rename = "exitCodes")]
    pub exit_codes: Option<HashMap<i32, String>>,
    pub schema: SchemaKind,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum InputKind {
    #[serde(rename = "stdin")]
    Stdin,
    #[serde(rename = "args")]
    Args,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum SchemaKind {
    #[serde(rename = "url")]
    Url(String),
    #[serde(rename = "command")]
    Command(SchemaCommand),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SchemaCommand {
    pub executable: String,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ResourceMethod {
    pub executable: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    pub input: InputKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_state: Option<bool>,
}

impl Invoke for ResourceManifest {
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
