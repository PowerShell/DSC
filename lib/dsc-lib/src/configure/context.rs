// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use chrono::{DateTime, Local};
use crate::{configure::config_doc::{ExecutionKind, Operation, UserFunctionDefinition}, extensions::dscextension::DscExtension};
use crate::types::SemanticVersion;
use dsc_lib_security_context::{get_security_context, SecurityContext};
use serde_json::{Map, Value};
use std::{collections::HashMap, path::PathBuf, sync::OnceLock};

use super::config_doc::{DataType, RestartRequired, SecurityContextKind};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProcessMode {
    Copy,
    Lambda,
    Normal,
    NoExpressionEvaluation,
    ParametersDefault,
    UserFunction,
}

#[derive(Clone, Debug)]
pub struct Context {
    pub copy: HashMap<String, i64>,
    pub copy_current_loop_name: String,
    /// The semantic version of DSC that is processing the configuration.
    ///
    /// Defaults to the version of the `dsc-lib` crate. A host that knows its own version, like
    /// the `dsc` CLI, sets it with
    /// [`Configurator::new_with_dsc_version`](crate::configure::Configurator::new_with_dsc_version).
    pub dsc_version: SemanticVersion,
    pub execution_type: ExecutionKind,
    pub extensions: Vec<DscExtension>,
    pub lambda_raw_args: std::cell::RefCell<Option<Vec<crate::parser::functions::FunctionArg>>>,
    pub lambda_variables: HashMap<String, Value>,
    pub lambdas: std::cell::RefCell<HashMap<String, crate::parser::functions::Lambda>>,
    pub operation: Option<Operation>,
    pub outputs: Map<String, Value>,
    pub parameters: HashMap<String, (Value, DataType)>,
    pub process_expressions: bool,
    pub process_mode: ProcessMode,
    pub processing_parameter_defaults: bool,
    pub references: Map<String, Value>,
    pub restart_required: Option<Vec<RestartRequired>>,
    pub security_context: SecurityContextKind,
    pub start_datetime: DateTime<Local>,
    pub state_changed: HashMap<String, bool>,
    pub stdout: Option<String>,
    pub system_root: PathBuf,
    pub user_functions: HashMap<String, UserFunctionDefinition>,
    pub variables: Map<String, Value>,
}

impl Context {
    #[must_use]
    pub fn new() -> Self {
        Self {
            copy: HashMap::new(),
            copy_current_loop_name: String::new(),
            dsc_version: dsc_lib_version(),
            execution_type: ExecutionKind::Actual,
            extensions: Vec::new(),
            lambda_raw_args: std::cell::RefCell::new(None),
            lambda_variables: HashMap::new(),
            lambdas: std::cell::RefCell::new(HashMap::new()),
            operation: None,
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
            state_changed: HashMap::new(),
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

/// Returns the version of the `dsc-lib` crate as a semantic version.
///
/// This is the default DSC version for a [`Context`] when the host doesn't provide its own.
///
/// The version is parsed from the cargo manifest on the first call and cached in
/// [`DSC_LIB_VERSION`], so later calls only clone the cached value.
///
/// # Panics
///
/// Panics if the version in the cargo manifest isn't a valid semantic version. Cargo rejects
/// such a manifest at build time, so this can't happen for a built crate.
fn dsc_lib_version() -> SemanticVersion {
    DSC_LIB_VERSION.get_or_init(|| {
        let manifest_version = env!("CARGO_PKG_VERSION");
        match SemanticVersion::parse(manifest_version) {
            Ok(version) => version,
            Err(err) => unreachable!("unable to parse '{manifest_version}' as a semantic version: {err}"),
        }
    }).clone()
}

/// Cached value for the version of the `dsc-lib` crate as a [`SemanticVersion`].
///
/// Initialized by the first call to [`dsc_lib_version`].
static DSC_LIB_VERSION: OnceLock<SemanticVersion> = OnceLock::new();

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
