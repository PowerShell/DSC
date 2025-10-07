// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::{context::Context as ConfigContext, config_doc::SecurityContextKind};
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use dsc_lib_osinfo::OsInfo;
use rust_i18n::t;
use serde::Serialize;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Serialize)]
pub struct ContextInfo {
    os: OsInfo,
    security: SecurityContextKind,
}

pub struct Context {}

impl Function for Context {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "context".to_string(),
            description: t!("functions.context.description").to_string(),
            category: vec![FunctionCategory::Deployment],
            min_args: 0,
            max_args: 0,
            accepted_arg_ordered_types: vec![],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Object],
        }
    }

    fn invoke(&self, _args: &[Value], config_context: &ConfigContext) -> Result<Value, DscError> {
        debug!("{}", t!("functions.context.invoked"));
        let context = ContextInfo {
            os: OsInfo::new(false),
            security: config_context.security_context.clone(),
        };
        Ok(serde_json::to_value(context)?)
    }
}
