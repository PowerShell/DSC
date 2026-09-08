// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActionInput {
    pub input_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActionOutput {
    pub output_text: String,
}

pub fn invoke_action(input: &str) -> Result<String, String> {
    // Parse the input JSON string into a serde_json::Value
    let input_value: ActionInput = serde_json::from_str(input)
        .map_err(|e| format!("Failed to parse input JSON: {}", e))?;

    let output = ActionOutput {
        output_text: input_value.input_text.clone(),
    };

    Ok(serde_json::to_string(&output).unwrap())
}
