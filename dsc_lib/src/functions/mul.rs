// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use serde_json::Value;
use tracing::trace;

#[derive(Debug, Default)]
pub struct Mul {}

impl Function for Mul {
    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        2
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::Number]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        trace!("mul function");
        let value = args[0].as_i64().unwrap_or_default() * args[1].as_i64().unwrap_or_default();
        Ok(Value::Number(value.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn numbers() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[mul(2, 3)]", &Context::new()).unwrap();
        assert_eq!(result, 6);
    }

    #[test]
    fn nested() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[mul(2, mul(3, 4))]", &Context::new()).unwrap();
        assert_eq!(result, 24);
    }

    #[test]
    fn invalid_one_parameter() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[mul(5)]", &Context::new());
        assert!(result.is_err());
    }
}
