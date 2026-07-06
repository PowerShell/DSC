// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct StateChanged {}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RestartKind {
    Process,
    Service,
    System,
}

impl Function for StateChanged {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "stateChanged".to_string(),
            description: t!("functions.stateChanged.description").to_string(),
            syntax: t!("functions.stateChanged.syntax").to_string(),
            constraints: None,
            category: vec![FunctionCategory::System],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        let name = args[0].as_str().unwrap();
        if let Some(changed) = context.state_changed.get(name) {
            return Ok(Value::Bool(*changed));
        }
        Err(DscError::Parser(t!("functions.stateChanged.noStateChangeInformation", name = name).to_string()))
    }
}
