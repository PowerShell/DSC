use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceManifest {
    #[serde(rename = "manifestVersion")]
    pub manifest_version: String,
    pub name: String,
    pub version: String,
    pub get: GetMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set: Option<SetMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<TestMethod>,
    #[serde(rename = "exitCodes", skip_serializing_if = "Option::is_none")]
    pub exit_codes: Option<HashMap<i32, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum InputKind {
    #[serde(rename = "args")]
    Args,
    #[serde(rename = "stdin")]
    Stdin,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<InputKind>,
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
