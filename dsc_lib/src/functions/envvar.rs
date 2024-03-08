// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::AcceptedArgKind;
use super::Function;
use serde_json::Value;
use std::env;

#[derive(Debug, Default)]
pub struct Envvar {}

impl Function for Envvar {
    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String]
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> usize {
        1
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        if let Ok(val) = env::var(args[0].as_str().unwrap_or_default()) {
            return Ok(Value::String(val));
        }

        Err(DscError::Function("envvar".to_string(), "Environment variable not found".to_string()))
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
