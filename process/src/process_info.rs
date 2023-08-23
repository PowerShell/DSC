// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::Serialize;
use std::string::ToString;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cmdline: String
}

impl ProcessInfo {
    pub fn new() -> Self {
        
        Self {
            pid: 0,
            name: "".to_string(),
            cmdline: "".to_string(),
        }
    }
}
