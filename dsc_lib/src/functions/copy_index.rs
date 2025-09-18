// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::{Context, ProcessMode};
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct CopyIndex {}

impl Function for CopyIndex {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "copyIndex".to_string(),
            description: t!("functions.copyIndex.description").to_string(),
            category: FunctionCategory::Numeric,
            min_args: 0,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String, FunctionArgKind::Number],
                vec![FunctionArgKind::Number],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Number],
        }
    }

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.copyIndex.invoked"));
        if context.process_mode != ProcessMode::Copy {
            return Err(DscError::Parser(t!("functions.copyIndex.cannotUseOutsideCopy").to_string()));
        }
        match args.len() {
            // no args, we return the current index of the current loop
            0 => Ok(Value::Number(get_current_loop_index(context)?.into())),
            1 => {
                // if arg is a number, we return current index + offset
                // if arg is a string, we return the index of that loop
                if let Some(offset) = args[0].as_i64() {
                    if offset < 0 {
                        return Err(DscError::Parser(t!("functions.copyIndex.offsetNegative").to_string()));
                    }
                    Ok(Value::Number((get_current_loop_index(context)? + offset).into()))
                } else if let Some(loop_name) = args[0].as_str() {
                    if let Some(index) = context.copy.get(loop_name) {
                        Ok(Value::Number((*index).into()))
                    } else {
                        Err(DscError::Parser(t!("functions.copyIndex.loopNameNotFound", name = loop_name).to_string()))
                    }
                } else {
                    Err(DscError::Parser(t!("functions.invalidArguments").to_string()))
                }
            }
            // two args, first is loop name, second is offset
            2 => {
                if let Some(loop_name) = args[0].as_str() {
                    if let Some(index) = context.copy.get(loop_name) {
                        if let Some(offset) = args[1].as_i64() {
                            Ok(Value::Number(((*index) + offset).into()))
                        } else {
                            Err(DscError::Parser(t!("functions.invalidArguments").to_string()))
                        }
                    } else {
                        Err(DscError::Parser(t!("functions.copyIndex.loopNameNotFound", name = loop_name).to_string()))
                    }
                } else {
                    Err(DscError::Parser(t!("functions.invalidArguments").to_string()))
                }
            }
            _ => Err(DscError::Parser(t!("functions.invalidArguments").to_string())),
        }
    }
}

fn get_current_loop_index(context: &Context) -> Result<i64, DscError> {
    if let Some(index) = context.copy.get(&(context.copy_current_loop_name)) {
        Ok(*index)
    } else {
        Err(DscError::Parser(t!("functions.copyIndex.noCurrentLoop").to_string()))
    }
}
