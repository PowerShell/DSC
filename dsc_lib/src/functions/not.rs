// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Not {}

impl Function for Not {
    fn description(&self) -> String {
        t!("functions.not.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Logical
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> usize {
        1
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::Boolean]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.not.invoked"));
        if let Some(arg1) = args[0].as_bool() {
            Ok(Value::Bool(!arg1))
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
    fn not_true() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[not(true)]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn not_false() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[not(false)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }
}
