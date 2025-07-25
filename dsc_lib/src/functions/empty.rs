// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Empty {}

impl Function for Empty {
    fn description(&self) -> String {
        t!("functions.empty.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Array
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> usize {
        1
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::Array, AcceptedArgKind::Object, AcceptedArgKind::String]
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
