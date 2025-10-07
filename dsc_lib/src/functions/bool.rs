// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::context::Context;
use crate::functions::{Function, FunctionArgKind, FunctionCategory, FunctionMetadata};
use crate::DscError;
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Bool {}

impl Function for Bool {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "bool".to_string(),
            description: t!("functions.bool.description").to_string(),
            category: vec![FunctionCategory::Logical],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String, FunctionArgKind::Number]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.bool.invoked"));
        if let Some(arg) = args[0].as_str() {
            match arg.to_lowercase().as_str() {
                "true" => Ok(Value::Bool(true)),
                "false" => Ok(Value::Bool(false)),
                _ => Err(DscError::Parser(t!("functions.invalidArguments").to_string())),
            }
        } else if let Some(num) = args[0].as_i64() {
            Ok(Value::Bool(num != 0))
        } else {
            Err(DscError::Parser(t!("functions.invalidArguments").to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn true_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[bool('true')]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn false_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[bool('false')]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn number_1() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[bool(1)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn number_0() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[bool(0)]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }
}
