// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Or {}

impl Function for Or {
    fn description(&self) -> String {
        t!("functions.or.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Logical
    }

    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        usize::MAX
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::Boolean]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.or.invoked"));
        for arg in args {
            if let Some(value) = arg.as_bool() {
                if value {
                    return Ok(Value::Bool(true));
                }
            } else {
                return Err(DscError::Parser(t!("functions.invalidArguments").to_string()));
            }
        }
        Ok(Value::Bool(false))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn two_values() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[or(true, false)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn multiple_values() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[or(true, false, true)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn all_false() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[or(false, false)]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }
}
