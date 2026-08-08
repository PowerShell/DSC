// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::config_doc::RestartRequired as RestartRequiredKind;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct RestartRequired {}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RestartKind {
    Process,
    Service,
    System,
}

impl Function for RestartRequired {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "restartRequired".to_string(),
            description: t!("functions.restartRequired.description").to_string(),
            syntax: t!("functions.restartRequired.syntax").to_string(),
            constraints: Some(t!("functions.restartRequired.constraints").to_string()),
            category: vec![FunctionCategory::System],
            min_args: 1,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String],
                vec![FunctionArgKind::String],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        let kind: RestartKind = serde_json::from_value(args[0].clone())
            .map_err(|_| DscError::Parser(t!("functions.restartRequired.invalidKind", kind = args[0].as_str().unwrap_or("unknown")).to_string()))?;

        let name = if args.len() > 1 {
            Some(
                args[1].as_str().unwrap().to_string()
            )
        } else {
            None
        };

        let restart_required = match kind {
            RestartKind::Process => {
                if let Some(name) = &name {
                    for restart_required in context.restart_required.as_ref().unwrap_or(&vec![]) {
                        if let RestartRequiredKind::Process(p) = restart_required && p.name == *name {
                            return Ok(Value::Bool(true));
                        }
                    }
                    false
                } else {
                    return Err(DscError::Parser(t!("functions.restartRequired.nameRequired", kind = "process").to_string()));
                }
            },
            RestartKind::Service => {
                if let Some(name) = &name {
                    for restart_required in context.restart_required.as_ref().unwrap_or(&vec![]) {
                        if let RestartRequiredKind::Service(service_name) = restart_required && service_name == name {
                            return Ok(Value::Bool(true));
                        }
                    }
                    false
                } else {
                    return Err(DscError::Parser(t!("functions.restartRequired.nameRequired", kind = "service").to_string()));
                }
            },
            RestartKind::System => {
                if name.is_some() {
                    return Err(DscError::Parser(t!("functions.restartRequired.nameNotAllowed", kind = "system").to_string()));
                }
                for restart_required in context.restart_required.as_ref().unwrap_or(&vec![]) {
                    if let RestartRequiredKind::System(_) = restart_required {
                        return Ok(Value::Bool(true));
                    }
                }
                false
            },
        };

        Ok(Value::Bool(restart_required))
    }
}
