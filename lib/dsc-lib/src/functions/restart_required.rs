// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use crate::util::resource_id;
use rust_i18n::t;
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
            .map_err(|_| DscError::FunctionArgumentError {
                function_name: self.get_metadata().name,
                message: t!("functions.restartRequired.invalidKind", kind = args[0]).to_string(),
            })?;

        let name = if args.len() > 1 {
            Some(
                args[1].as_str()?.to_string()
            )
        } else {
            None
        };

        let restart_required = match kind {
            RestartKind::Process => {
                if name.is_none() {
                    return Err(DscError::FunctionArgumentError {
                        function_name: self.get_metadata().name,
                        message: t!("functions.restartRequired.nameRequired").to_string(),
                    });
                }
                context
                .restart_required
                .as_ref()
                .map(|rr| rr.iter().any(|r| r.kind == "process" && r.name == name))
                .unwrap_or(false)
            },
            RestartKind::Service => {
                if name.is_none() {
                    return Err(DscError::FunctionArgumentError {
                        function_name: self.get_metadata().name,
                        message: t!("functions.restartRequired.nameRequired").to_string(),
                    });
                }
                context
                .restart_required
                .as_ref()
                .map(|rr| rr.iter().any(|r| r.kind == "service" && r.name == name))
                .unwrap_or(false)
            },
            RestartKind::System => {
                if name.is_some() {
                    return Err(DscError::FunctionArgumentError {
                        function_name: self.get_metadata().name,
                        message: t!("functions.restartRequired.nameNotAllowed").to_string(),
                    });
                }
                context
                .restart_required
                .as_ref()
                .map(|rr| rr.iter().any(|r| r.kind == "system"))
                .unwrap_or(false)
            },
        };

        Ok(Value::Bool(restart_required))
    }
}
