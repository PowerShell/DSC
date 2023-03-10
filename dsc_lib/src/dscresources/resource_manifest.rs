use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::dscerror::DscError;
use crate::dscresources::dscresource::Invoke;

use super::invoke_result::{GetResult, SetResult, TestResult};

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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ReturnKind {
    #[serde(rename = "state")]
    State,
    #[serde(rename = "diff")]
    Diff,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GetMethod {
    pub executable: String,
    pub args: Option<Vec<String>>,
    pub input: InputKind,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SetMethod {
    pub executable: String,
    pub args: Option<Vec<String>>,
    pub input: InputKind,
    #[serde(rename = "preTest", skip_serializing_if = "Option::is_none")]
    pub pre_test: Option<bool>,
    #[serde(rename = "return", skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TestMethod {
    pub executable: String,
    pub args: Option<Vec<String>>,
    pub input: InputKind,
    #[serde(rename = "return", skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnKind>,
}

impl Invoke for ResourceManifest {
    fn get(&self, _filter: &str) -> Result<GetResult, DscError> {
        Err(DscError::NotImplemented)
    }

    fn set(&self, _desired: &str) -> Result<SetResult, DscError> {
        Err(DscError::NotImplemented)
    }

    fn test(&self, _expected: &str) -> Result<TestResult, DscError> {
        Err(DscError::NotImplemented)
    }
}
