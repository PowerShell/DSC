// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::AcceptedArgKind;
use super::Function;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct Equals {}

impl Function for Equals {
    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::Number, AcceptedArgKind::String, AcceptedArgKind::Array, AcceptedArgKind::Object]
    }

    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        2
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        Ok(Value::Bool(args[0] == args[1]))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::Value;

    #[test]
    fn int_equal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[equals(1,1)]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn int_notequal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[equals(1,2]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn string_equal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[equals('test','test')]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn string_notequal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[equals('test','TEST')]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn different_types() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[equals(1,'string')]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    // TODO: Add tests for arrays once `createArray()` is implemented
    // TODO: Add tests for objects once `createObject()` is implemented
}
