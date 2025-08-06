// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use super::Function;
use rust_i18n::t;
use serde_json::Value;
use std::env;

#[derive(Debug, Default)]
pub struct Envvar {}

impl Function for Envvar {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "envvar".to_string(),
            description: t!("functions.envvar.description").to_string(),
            category: FunctionCategory::System,
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        if let Ok(val) = env::var(args[0].as_str().unwrap_or_default()) {
            return Ok(Value::String(val));
        }

        Err(DscError::Function("envvar".to_string(), t!("functions.envvar.notFound").to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn valid() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[envvar('PATH')]", &Context::new()).unwrap();
        assert_eq!(result, std::env::var("PATH").unwrap());
    }

    #[test]
    fn invalid() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[envvar('INVALID')]", &Context::new());
        assert!(result.is_err());
    }
}
