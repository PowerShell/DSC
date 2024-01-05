// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde_json::Value;
use std::collections::HashMap;

pub struct Context {
    pub parameters: HashMap<String, Value>,
    pub _variables: HashMap<String, Value>,
    pub _outputs: HashMap<String, Value>, // This is eventually for References function to get output from resources
}

impl Context {
    #[must_use]
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            _variables: HashMap::new(),
            _outputs: HashMap::new(),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
