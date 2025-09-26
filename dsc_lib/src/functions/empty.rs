// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Empty {}

impl Function for Empty {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "empty".to_string(),
            description: t!("functions.empty.description").to_string(),
            category: vec![FunctionCategory::String, FunctionCategory::Array, FunctionCategory::Object],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::Array, FunctionArgKind::Object, FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.empty.invoked"));
        if let Some(array) = args[0].as_array() {
            return Ok(Value::Bool(array.is_empty()));
        }

        if let Some(object) = args[0].as_object() {
            return Ok(Value::Bool(object.is_empty()));
        }

        if let Some(string) = args[0].as_str() {
            return Ok(Value::Bool(string.is_empty()));
        }

        Err(DscError::Parser(t!("functions.empty.invalidArgType").to_string()))
    }
}


#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::Value;

    #[test]
    fn empty_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[empty('')]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn not_empty_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[empty('foo')]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }
}
