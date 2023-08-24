// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::Serialize;

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
            name: String::new(),
            cmdline: String::new(),
        }
    }
}
