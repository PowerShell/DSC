use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct ResourceManifest {
    #[serde(rename = "manifestVersion")]
    pub manifest_version: String,
    pub name: String,
    pub version: String,
    pub get: GetMethod,
    pub set: SetMethod,
    pub test: TestMethod,
    #[serde(rename = "exitCodes")]
    pub exit_codes: Option<HashMap<i32, String>>,
    pub schema: SchemaKind,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum InputKind {
    #[serde(rename = "stdin")]
    Stdin,
    #[serde(rename = "args")]
    Args,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum SchemaKind {
    #[serde(rename = "command")]
    Command(SchemaCommand),
    #[serde(rename = "embedded")]
    Embedded(String),
    #[serde(rename = "url")]
    Url(String),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct SchemaCommand {
    pub executable: String,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum ReturnKind {
    #[serde(rename = "state")]
    State,
    #[serde(rename = "stateAndDiff")]
    StateAndDiff,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct GetMethod {
    pub executable: String,
    pub args: Option<Vec<String>>,
    pub input: InputKind,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct SetMethod {
    pub executable: String,
    pub args: Option<Vec<String>>,
    pub input: InputKind,
    #[serde(rename = "preTest", skip_serializing_if = "Option::is_none")]
    pub pre_test: Option<bool>,
    #[serde(rename = "return", skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct TestMethod {
    pub executable: String,
    pub args: Option<Vec<String>>,
    pub input: InputKind,
    #[serde(rename = "return", skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnKind>,
}
