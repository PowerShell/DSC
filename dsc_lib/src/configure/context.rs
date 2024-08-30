// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use chrono::{DateTime, Local};
use crate::configure::config_doc::ExecutionKind;
use security_context_lib::{get_security_context, SecurityContext};
use serde_json::Value;
use std::collections::HashMap;

use super::config_doc::{DataType, SecurityContextKind};

pub struct Context {
    pub execution_type: ExecutionKind,
    pub outputs: HashMap<String, Value>, // this is used by the `reference()` function to retrieve output
    pub parameters: HashMap<String, (Value, DataType)>,
    pub security_context: SecurityContextKind,
    pub variables: HashMap<String, Value>,
    pub start_datetime: DateTime<Local>,
}

impl Context {
    #[must_use]
    pub fn new() -> Self {
        Self {
            execution_type: ExecutionKind::Actual,
            outputs: HashMap::new(),
            parameters: HashMap::new(),
            security_context: match get_security_context() {
                SecurityContext::Admin => SecurityContextKind::Elevated,
                SecurityContext::User => SecurityContextKind::Restricted,
            },
            variables: HashMap::new(),
            start_datetime: chrono::Local::now(),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
