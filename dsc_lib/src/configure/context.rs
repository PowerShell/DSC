// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use chrono::{DateTime, Local};
use crate::{configure::config_doc::ExecutionKind, extensions::dscextension::DscExtension};
use security_context_lib::{get_security_context, SecurityContext};
use serde_json::{Map, Value};
use std::{collections::HashMap, path::PathBuf};

use super::config_doc::{DataType, RestartRequired, SecurityContextKind};

pub struct Context {
    pub execution_type: ExecutionKind,
    pub extensions: Vec<DscExtension>,
    pub references: Map<String, Value>,
    pub system_root: PathBuf,
    pub parameters: HashMap<String, (Value, DataType)>,
    pub security_context: SecurityContextKind,
    pub variables: Map<String, Value>,
    pub start_datetime: DateTime<Local>,
    pub restart_required: Option<Vec<RestartRequired>>,
    pub process_expressions: bool,
}

impl Context {
    #[must_use]
    pub fn new() -> Self {
        Self {
            execution_type: ExecutionKind::Actual,
            extensions: Vec::new(),
            references: Map::new(),
            system_root: get_default_os_system_root(),
            parameters: HashMap::new(),
            security_context: match get_security_context() {
                SecurityContext::Admin => SecurityContextKind::Elevated,
                SecurityContext::User => SecurityContextKind::Restricted,
            },
            variables: Map::new(),
            start_datetime: chrono::Local::now(),
            restart_required: None,
            process_expressions: true,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
fn get_default_os_system_root() -> PathBuf {
    // use SYSTEMDRIVE env var to get the default target path, append trailing separator
    let system_drive = std::env::var("SYSTEMDRIVE").unwrap_or_else(|_| "C:".to_string());
    PathBuf::from(system_drive + "\\")
}

#[cfg(not(target_os = "windows"))]
fn get_default_os_system_root() -> PathBuf {
    PathBuf::from("/")
}
