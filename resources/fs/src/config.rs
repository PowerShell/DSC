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
    pub hash: String,

    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
}


// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Hash)]
// #[serde(rename ="Directory", deny_unknown_fields)]
// pub struct Directory {
//     /// The path to the directory.
//     pub path: String,

//     /// The directory size.
//     pub size: u64,

//     /// The files under the directory.
//     pub files: Vec<File>,

//     #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
//     pub exist: Option<bool>,
// }