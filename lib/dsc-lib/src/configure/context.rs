// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use chrono::{DateTime, Local};
use crate::{configure::config_doc::{ExecutionKind, UserFunctionDefinition}, extensions::dscextension::DscExtension};
use dsc_lib_security_context::{get_security_context, SecurityContext};
use serde_json::{Map, Value};
use std::{collections::HashMap, path::PathBuf};

use super::config_doc::{DataType, RestartRequired, SecurityContextKind};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProcessMode {
    Copy,
    Normal,
    NoExpressionEvaluation,
    ParametersDefault,
    UserFunction,
}

#[derive(Clone, Debug)]
pub struct Context {
    pub copy: HashMap<String, i64>,
    pub copy_current_loop_name: String,
    pub dsc_version: Option<String>,
    pub environment_variables: HashMap<String, String>,
    pub execution_type: ExecutionKind,
    pub extensions: Vec<DscExtension>,
    pub outputs: Map<String, Value>,
    pub parameters: HashMap<String, (Value, DataType)>,
    pub process_expressions: bool,
    pub process_mode: ProcessMode,
    pub processing_parameter_defaults: bool,
    pub references: Map<String, Value>,
    pub restart_required: Option<Vec<RestartRequired>>,
    pub security_context: SecurityContextKind,
    pub start_datetime: DateTime<Local>,
    pub stdout: Option<String>,
    pub system_root: PathBuf,
    pub user_functions: HashMap<String, UserFunctionDefinition>,
    pub variables: Map<String, Value>,
}

impl Context {
    #[must_use]
    pub fn new() -> Self {
        let environment_variables = std::env::vars().map(|(k, v)| (k, v)).collect();
        Self {
            copy: HashMap::new(),
            copy_current_loop_name: String::new(),
            dsc_version: None,
            environment_variables,
            execution_type: ExecutionKind::Actual,
            extensions: Vec::new(),
            outputs: Map::new(),
            parameters: HashMap::new(),
            process_expressions: true,
            process_mode: ProcessMode::Normal,
            processing_parameter_defaults: false,
            references: Map::new(),
            restart_required: None,
            security_context: match get_security_context() {
                SecurityContext::Admin => SecurityContextKind::Elevated,
                SecurityContext::User => SecurityContextKind::Restricted,
            },
            start_datetime: chrono::Local::now(),
            stdout: None,
            system_root: get_default_os_system_root(),
            user_functions: HashMap::new(),
            variables: Map::new(),
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
