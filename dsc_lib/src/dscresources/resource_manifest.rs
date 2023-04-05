// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceManifest {
    /// The version of the resource manifest schema.
    #[serde(rename = "manifestVersion")]
    pub manifest_version: String,
    /// The namespaced name of the resource.
    #[serde(rename = "type")]
    pub resource_type: String,
    /// The version of the resource.
    pub version: String,
    /// The description of the resource.
    pub description: Option<String>,
    /// Details how to call the Get method of the resource.
    pub get: GetMethod,
    /// Details how to call the Set method of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set: Option<SetMethod>,
    /// Details how to call the Test method of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<TestMethod>,
    /// Mapping of exit codes to descriptions.  Zero is always success and non-zero is always failure.
    #[serde(rename = "exitCodes", skip_serializing_if = "Option::is_none")]
    pub exit_codes: Option<HashMap<i32, String>>,
    /// Details how to get the schema of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum InputKind {
    /// The input is accepted as named parameters.
    #[serde(rename = "args")]
    Args,
    /// The input is accepted as a JSON object via STDIN.
    #[serde(rename = "stdin")]
    Stdin,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum SchemaKind {
    /// The schema is returned by running a command.
    #[serde(rename = "command")]
    Command(SchemaCommand),
    /// The schema is embedded in the manifest.
    #[serde(rename = "embedded")]
    Embedded(Value),
    /// The schema is retrieved from a URL.
    #[serde(rename = "url")]
    Url(String),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct SchemaCommand {
    /// The command to run to get the schema.
    pub executable: String,
    /// The arguments to pass to the command.
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum ReturnKind {
    /// The return JSON is the state of the resource.
    #[serde(rename = "state")]
    State,
    /// The return JSON is the state of the resource and the diff.
    #[serde(rename = "stateAndDiff")]
    StateAndDiff,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct GetMethod {
    /// The command to run to get the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Get.
    pub args: Option<Vec<String>>,
    /// How to pass optional input for a Get.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<InputKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct SetMethod {
    /// The command to run to set the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Set.
    pub args: Option<Vec<String>>,
    /// How to pass required input for a Set.
    pub input: InputKind,
    /// Whether to run the Test method before the Set method.  True means the resource will perform its own test before running the Set method.
    #[serde(rename = "preTest", skip_serializing_if = "Option::is_none")]
    pub pre_test: Option<bool>,
    /// The type of return value expected from the Set method.
    #[serde(rename = "return", skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct TestMethod {
    /// The command to run to test the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Test.
    pub args: Option<Vec<String>>,
    /// How to pass required input for a Test.
    pub input: InputKind,
    /// The type of return value expected from the Test method.
    #[serde(rename = "return", skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnKind>,
}
