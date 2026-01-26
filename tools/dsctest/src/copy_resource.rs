// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CopyResource {
    pub source_file: String,
    pub type_name: String,
}

pub fn copy_the_resource(source_file: &str, type_name: &str) -> Result<(), String> {
    // open the source_file, derialize it from JSON, change the `type` property to type_name,
    // serialize it back to JSON and save it to a new file named "<name>.dsc.resource.json"
    let file_content = std::fs::read_to_string(source_file)
        .map_err(|e| format!("Failed to read source file: {}", e))?;
    let mut resource_json: serde_json::Value = serde_json::from_str(&file_content)
        .map_err(|e| format!("Failed to parse JSON from source file: {}", e))?;
    if let Some(obj) = resource_json.as_object_mut() {
        obj.insert("type".to_string(), serde_json::Value::String(type_name.to_string()));
    } else {
        return Err("Source file not a resource manifest".to_string());
    }
    let name_part = type_name.split('/').last().unwrap_or(type_name);
    let output_file = format!("{name_part}.dsc.resource.json");
    let output_content = serde_json::to_string_pretty(&resource_json)
        .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
    std::fs::write(&output_file, output_content)
        .map_err(|e| format!("Failed to write output file: {e}"))?;
    Ok(())
}
