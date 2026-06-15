// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::PathBuf;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandInfo {
    /// Switch to include defaults in the output
    #[serde(rename = "_includeDefaults")]
    pub include_defaults: bool,
    /// input provided with the command
    pub input: Map<String, Value>,
    /// Filepath for the `sshd_config` file to be processed
    pub filepath: Option<PathBuf>,
    #[serde(rename = "_purge")]
    pub purge: bool,
    /// additional arguments for the call to sshd -T
    pub sshd_args: Option<SshdCommandArgs>
}

impl CommandInfo {
    /// Create a new `CommandInfo` instance.
    pub fn new(
        include_defaults: bool,
        input: Map<String, Value>,
        filepath: Option<PathBuf>,
        purge: bool,
        sshd_args: Option<SshdCommandArgs>
    ) -> Self {
        // Lowercase keys for case-insensitive comparison
        let input = input.into_iter()
            .map(|(k, v)| (k.to_lowercase(), v))
            .collect();

        Self {
            include_defaults,
            input,
            filepath,
            purge,
            sshd_args
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SshdCommandArgs {
    /// the path to the `sshd_config` file to be processed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filepath: Option<PathBuf>,
    /// additional arguments to pass to the sshd -T command
    #[serde(rename = "additionalArgs", skip_serializing_if = "Option::is_none")]
    pub additional_args: Option<Vec<String>>,
}
