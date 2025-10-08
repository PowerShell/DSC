// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandInfo {
    /// Switch to include defaults in the output
    #[serde(rename = "_includeDefaults")]
    pub include_defaults: bool,
    /// input provided with the command
    pub input: Map<String, Value>,
    /// metadata provided with the command
    pub metadata: Metadata,
    /// additional arguments for the call to sshd -T
    pub sshd_args: Option<SshdCommandArgs>,
}

impl CommandInfo {
    /// Create a new `CommandInfo` instance.
    pub fn new(include_defaults: bool) -> Self {
        Self {
            include_defaults,
            input: Map::new(),
            metadata: Metadata::new(),
            sshd_args: None,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Metadata {
    /// Filepath for the `sshd_config` file to be processed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filepath: Option<String>,
}

impl Metadata {
    /// Create a new `Metadata` instance.
    pub fn new() -> Self {
        Self { filepath: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SshdCommandArgs {
    /// the path to the `sshd_config` file to be processed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filepath: Option<String>,
    /// additional arguments to pass to the sshd -T command
    #[serde(rename = "additionalArgs", skip_serializing_if = "Option::is_none")]
    pub additional_args: Option<Vec<String>>,
}
