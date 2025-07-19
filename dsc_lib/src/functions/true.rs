// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct True {}

impl Function for True {
    fn description(&self) -> String {
        t!("functions.true.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Logical
    }

    fn min_args(&self) -> usize {
        0
    }

    fn max_args(&self) -> usize {
        0
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![]
    }

    fn invoke(&self, _args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.true.invoked"));
        Ok(Value::Bool(true))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn true_function() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[true()]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }
}
