// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::schema::{get_registry_key_path_pattern, get_schema_uri};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum RegistryValueData {
    #[schemars(
        title = t!("schema.valueData.String.title").to_string(),
        description = t!("schema.valueData.String.description").to_string(),
        extend(
            "markdownDescription" = t!("schema.valueData.String.markdownDescription").to_string()
        )
    )]
    String(String),
    #[schemars(
        title = t!("schema.valueData.ExpandString.title").to_string(),
        description = t!("schema.valueData.ExpandString.description").to_string(),
        extend(
            "markdownDescription" = t!("schema.valueData.ExpandString.markdownDescription").to_string()
        )
    )]
    ExpandString(String),
    #[schemars(
        title = t!("schema.valueData.Binary.title").to_string(),
        description = t!("schema.valueData.Binary.description").to_string(),
        extend(
            "markdownDescription" = t!("schema.valueData.Binary.markdownDescription").to_string()
        )
    )]
    Binary(Vec<u8>),
    #[schemars(
        title = t!("schema.valueData.DWord.title").to_string(),
        description = t!("schema.valueData.DWord.description").to_string(),
        extend(
            "markdownDescription" = t!("schema.valueData.DWord.markdownDescription").to_string()
        )
    )]
    DWord(u32),
    #[schemars(
        title = t!("schema.valueData.MultiString.title").to_string(),
        description = t!("schema.valueData.MultiString.description").to_string(),
        extend(
            "markdownDescription" = t!("schema.valueData.MultiString.markdownDescription").to_string()
        )
    )]
    MultiString(Vec<String>),
    #[schemars(
        title = t!("schema.valueData.QWord.title").to_string(),
        description = t!("schema.valueData.QWord.description").to_string(),
        extend(
            "markdownDescription" = t!("schema.valueData.QWord.markdownDescription").to_string()
        )
    )]
    QWord(u64),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "Registry", deny_unknown_fields)]
#[schemars(
    title=t!("schema.title").to_string(),
    description=t!("schema.description").to_string(),
    extend(
        "$id" = get_schema_uri(true),
        "dependentRequired" = { "valueData": ["valueName"] },
        "markdownDescription" = t!("schema.markdownDescription").to_string()
    )
)]
pub struct Registry {
    /// The path to the registry key.
    #[serde(rename = "keyPath")]
    #[schemars(
        title = t!("schema.keyPath.title").to_string(),
        description = t!("schema.keyPath.description").to_string(),
        required,
        regex(pattern = get_registry_key_path_pattern()),
        extend(
            "markdownDescription" = t!("schema.keyPath.markdownDescription").to_string(),
            "patternErrorMessage" = t!("schema.keyPath.patternErrorMessage").to_string(),
        )
    )]
    pub key_path: String,
    /// The information from a config set --what-if operation.
    #[serde(rename = "_metadata", skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = t!("schema.metadata.title").to_string(),
        description = t!("schema.metadata.description").to_string(),
        extend(
            "readOnly" = true,
            "markdownDescription" = t!("schema.metadata.markdownDescription").to_string()
        )
    )]
    pub metadata: Option<Metadata>,
    /// The name of the registry value.
    #[serde(rename = "valueName", skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = t!("schema.valueName.title").to_string(),
        description = t!("schema.valueName.description").to_string(),
        extend(
            "markdownDescription" = t!("schema.valueName.markdownDescription").to_string()
        )
    )]
    pub value_name: Option<String>,
    /// The data of the registry value.
    #[serde(rename = "valueData", skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = t!("schema.valueData.title").to_string(),
        description = t!("schema.valueData.description").to_string(),
        extend(
            "markdownDescription" = t!("schema.valueData.markdownDescription").to_string()
        )
    )]
    pub value_data: Option<RegistryValueData>,
    #[serde(rename = "_exist")]
    #[schemars(
        title = t!("schema.exist.title").to_string(),
        description = t!("schema.exist.description").to_string(),
        extend(
            "default" = true,
            "markdownDescription" = t!("schema.exist.markdownDescription").to_string()
        )
    )]
    pub exist: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    #[serde(rename = "whatIf", skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = t!("schema.metadata.whatIf.title").to_string(),
        description = t!("schema.metadata.whatIf.description").to_string(),
        extend(
            "readOnly" = true,
            "markdownDescription" = t!("schema.metadata.whatIf.markdownDescription").to_string()
        )
    )]
    pub what_if: Option<Vec<String>>
}
