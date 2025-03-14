// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Hash)]
#[serde(rename ="File", deny_unknown_fields)]
pub struct File {
    /// The path to the file.
    pub path: String,

    /// The file size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,

    /// The file hash.
    pub hash: Option<String>,

    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Hash)]
#[serde(rename ="Directory", deny_unknown_fields)]
pub struct Directory {
    /// The path to the directory.
    pub path: String,

    /// The directory size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,

    /// The files under the directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<File>>,

    /// Recurse into subdirectories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurse: Option<bool>,

    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Hash)]
#[serde(rename ="FileContent", deny_unknown_fields)]
pub struct FileContent
{
    /// The path to the file.
    pub path: String,

    /// The file hash. If not provided, the hash is calculated from the content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,

    /// The file encoding. UTF-8 is the default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,

    /// The file content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// If the file exists. True is the default.
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Hash)]
pub enum Encoding {
    Utf8,
    Utf16,
    Ascii,
    Binary,
}
