// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Length {}

impl Function for Length {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "length".to_string(),
            description: t!("functions.length.description").to_string(),
            category: FunctionCategory::Array,
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::Array, FunctionArgKind::Object, FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Number],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.length.invoked"));
        if let Some(array) = args[0].as_array() {
            return Ok(Value::Number(array.len().into()));
        }

        if let Some(object) = args[0].as_object() {
            return Ok(Value::Number(object.keys().len().into()));
        }

        if let Some(string) = args[0].as_str() {
            return Ok(Value::Number(string.len().into()));
        }

        Err(DscError::Parser(t!("functions.length.invalidArgType").to_string()))
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
        let result = parser.parse_and_execute("[length('')]", &Context::new()).unwrap();
        assert_eq!(result, Value::Number(0.into()));
    }

    #[test]
    fn not_empty_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[length('foo')]", &Context::new()).unwrap();
        assert_eq!(result, Value::Number(3.into()));
    }
}
